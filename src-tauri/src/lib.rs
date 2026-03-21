mod commands;
pub mod error;
pub mod opencode_error;
mod models;
mod modules;
mod utils;

use modules::config::CloseWindowBehavior;
use modules::logger;
use modules::opencode_config::ConfigManager;
use modules::opencode_db::Database;
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
#[cfg(target_os = "macos")]
use tauri::RunEvent;
use tauri::WindowEvent;
use tauri::{Emitter, Manager};
use tokio::sync::RwLock;
use tracing::info;

pub struct ProxyServiceState(pub Arc<RwLock<Option<modules::proxy::ProxyService>>>);

/// 全局 AppHandle 存储
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// 获取全局 AppHandle
pub fn get_app_handle() -> Option<&'static tauri::AppHandle> {
    APP_HANDLE.get()
}

#[cfg(target_os = "macos")]
fn apply_macos_activation_policy(app: &tauri::AppHandle) {
    let config = modules::config::get_user_config();
    let (policy, dock_visible, policy_label) = if config.hide_dock_icon {
        (ActivationPolicy::Accessory, false, "hidden")
    } else {
        (ActivationPolicy::Regular, true, "visible")
    };

    if let Err(err) = app.set_activation_policy(policy) {
        logger::log_warn(&format!("[Window] 设置 macOS 激活策略失败: {}", err));
        return;
    }

    if let Err(err) = app.set_dock_visibility(dock_visible) {
        logger::log_warn(&format!("[Window] 设置 macOS Dock 可见性失败: {}", err));
    }

    if dock_visible {
        let _ = app.show();
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
        }
    }

    info!("[Window] 已应用 macOS Dock 图标策略: {}", policy_label);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init_logger();

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            logger::log_info("[Linux] 设置 WEBKIT_DISABLE_DMABUF_RENDERER=1");
        }
    }

    let config_manager = ConfigManager::new().unwrap_or_else(|e| {
        logger::log_error(&format!("[OpenCode] ConfigManager 初始化失败: {}", e));
        panic!("ConfigManager init failed: {}", e);
    });

    let database = Database::open().unwrap_or_else(|e| {
        logger::log_error(&format!("[OpenCode] Database 初始化失败: {}", e));
        panic!("Database init failed: {}", e);
    });
    let db_arc = Arc::new(database);
    let proxy_service_state = ProxyServiceState(Arc::new(RwLock::new(None)));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = argv.iter().find(|arg| arg.starts_with("aiswitch://")) {
                let _ = app.emit("deep-link-received", url.clone());
            }
            let _ = app.get_webview_window("main").map(|window| {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            });
        }))
        .manage(Mutex::new(config_manager))
        .manage(db_arc)
        .manage(proxy_service_state)
        .setup(|app| {
            info!("ai switch 启动...");

            // 存储全局 AppHandle
            let _ = APP_HANDLE.set(app.handle().clone());

            // 初始化 Updater 插件
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
                app.handle().plugin(tauri_plugin_process::init())?;
                info!("[Updater] Tauri Updater + Process 插件已初始化");
            }

            // 启动时同步设置合并（移至后台线程，不阻塞窗口显示）
            std::thread::spawn(|| {
                let current_config = modules::config::get_user_config();
                if let Some(merged_language) = modules::sync_settings::merge_setting_on_startup(
                    "language",
                    &current_config.language,
                    None,
                ) {
                    info!(
                        "[SyncSettings] 启动时合并语言设置: {} -> {}",
                        current_config.language, merged_language
                    );
                    let new_config = modules::config::UserConfig {
                        language: merged_language,
                        ..current_config
                    };
                    if let Err(e) = modules::config::save_user_config(&new_config) {
                        logger::log_error(&format!("[SyncSettings] 保存合并后的配置失败: {}", e));
                    }
                }
            });

            // 启动 WebSocket 服务（使用 Tauri 的 async runtime）
            tauri::async_runtime::spawn(async {
                modules::websocket::start_server().await;
            });

            // 启动网关服务（如果配置了自动启动）
            tauri::async_runtime::spawn(async {
                let gw_config = modules::gateway::config::get_gateway_config();
                if gw_config.auto_start && gw_config.enabled {
                    info!("[Gateway] 自动启动网关服务...");
                    if let Err(e) = modules::gateway::start_gateway(gw_config.port).await {
                        logger::log_error(&format!("[Gateway] 自动启动失败: {}", e));
                    }
                }
            });

            // 启动 Sub2api 子进程（如果配置了自动启动）
            tauri::async_runtime::spawn(async {
                let sub2api_status = modules::subprocess::get_sub2api_status();
                if !sub2api_status.running {
                    let config_path = dirs::data_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("com.jlcodes.ai-switch")
                        .join("sub2api_config.json");
                    if config_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&config_path) {
                            if let Ok(config) = serde_json::from_str::<modules::subprocess::sub2api::Sub2apiConfig>(&content) {
                                if config.auto_start {
                                    info!("[Sub2api] 自动启动子进程...");
                                    if let Err(e) = modules::subprocess::start_sub2api().await {
                                        logger::log_error(&format!("[Sub2api] 自动启动失败: {}", e));
                                    }
                                }
                            }
                        }
                    }
                }
            });

            // 启动网页查询服务（网络服务配置中的独立模块）
            tauri::async_runtime::spawn(async {
                modules::web_report::start_server().await;
            });

            {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    modules::codex_oauth::restore_pending_oauth_listener(app_handle);
                    modules::windsurf_oauth::restore_pending_oauth_listener();
                    modules::kiro_oauth::restore_pending_oauth_listener();
                    modules::trae_oauth::restore_pending_oauth_listener();
                    modules::gemini_oauth::restore_pending_oauth_state();
                });
            }

            #[cfg(target_os = "macos")]
            apply_macos_activation_policy(&app.handle());

            // 创建骨架托盘（无账号文件 I/O，秒出）
            if let Err(e) = modules::tray::create_tray_skeleton(app.handle()) {
                logger::log_error(&format!("[Tray] 创建骨架托盘失败: {}", e));
            }

            // 后台线程加载完整托盘菜单（含账号数据）
            let tray_app_handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(e) = modules::tray::update_tray_menu(&tray_app_handle) {
                    logger::log_error(&format!("[Tray] 后台更新托盘菜单失败: {}", e));
                }
            });
            
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if window.label() != "main" {
                    return;
                }
                let config = modules::config::get_user_config();

                match config.close_behavior {
                    CloseWindowBehavior::Minimize => {
                        api.prevent_close();
                        let _ = window.hide();
                        info!("[Window] 窗口已最小化到托盘");
                    }
                    CloseWindowBehavior::Quit => {
                        info!("[Window] 用户选择退出应用");
                    }
                    CloseWindowBehavior::Ask => {
                        api.prevent_close();
                        let _ = window.emit("window:close_requested", ());
                        info!("[Window] 等待用户选择关闭行为");
                    }
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            // Account Commands
            commands::account::list_accounts,
            commands::account::add_account,
            commands::account::delete_account,
            commands::account::delete_accounts,
            commands::account::reorder_accounts,
            commands::account::get_current_account,
            commands::account::set_current_account,
            commands::account::fetch_account_quota,
            commands::account::refresh_all_quotas,
            commands::account::refresh_current_quota,
            commands::account::switch_account,
            commands::account::bind_account_fingerprint,
            commands::account::get_bound_accounts,
            commands::account::update_account_tags,
            commands::account::sync_current_from_client,
            commands::account::sync_from_extension,
            // Device Commands
            commands::device::get_device_profiles,
            commands::device::bind_device_profile,
            commands::device::bind_device_profile_with_profile,
            commands::device::list_device_versions,
            commands::device::restore_device_version,
            commands::device::delete_device_version,
            commands::device::restore_original_device,
            commands::device::open_device_folder,
            commands::device::preview_generate_profile,
            commands::device::preview_current_profile,
            // Fingerprint Commands
            commands::device::list_fingerprints,
            commands::device::get_fingerprint,
            commands::device::generate_new_fingerprint,
            commands::device::capture_current_fingerprint,
            commands::device::create_fingerprint_with_profile,
            commands::device::apply_fingerprint,
            commands::device::delete_fingerprint,
            commands::device::delete_unbound_fingerprints,
            commands::device::rename_fingerprint,
            commands::device::get_current_fingerprint_id,
            // OAuth Commands
            commands::oauth::start_oauth_login,
            commands::oauth::prepare_oauth_url,
            commands::oauth::complete_oauth_login,
            commands::oauth::submit_oauth_callback_url,
            commands::oauth::cancel_oauth_login,
            // Import/Export Commands
            commands::import::import_from_old_tools,
            commands::import::import_fingerprints_from_old_tools,
            commands::import::import_fingerprints_from_json,
            commands::import::import_from_local,
            commands::import::import_from_json,
            commands::import::import_from_files,
            commands::import::export_accounts,
            // System Commands
            commands::system::open_data_folder,
            commands::system::save_text_file,
            commands::system::get_downloads_dir,
            commands::system::get_network_config,
            commands::system::save_network_config,
            commands::system::get_general_config,
            commands::system::save_general_config,
            commands::system::save_tray_platform_layout,
            commands::system::set_app_path,
            commands::system::detect_app_path,
            commands::system::set_wakeup_override,
            commands::system::handle_window_close,
            commands::system::open_folder,
            commands::system::delete_corrupted_file,
            // Wakeup Commands
            commands::wakeup::wakeup_ensure_runtime_ready,
            commands::wakeup::trigger_wakeup,
            commands::wakeup::fetch_available_models,
            commands::wakeup::wakeup_sync_state,
            commands::wakeup::wakeup_load_history,
            commands::wakeup::wakeup_add_history,
            commands::wakeup::wakeup_clear_history,
            commands::wakeup::wakeup_verification_load_state,
            commands::wakeup::wakeup_verification_load_history,
            commands::wakeup::wakeup_verification_delete_history,
            commands::wakeup::wakeup_verification_run_batch,
            // Update Commands
            commands::update::should_check_updates,
            commands::update::update_last_check_time,
            commands::update::get_update_settings,
            commands::update::save_update_settings,
            commands::update::save_pending_update_notes,
            commands::update::check_version_jump,
            commands::update::update_log,
            commands::update::get_update_runtime_info,
            commands::update::install_linux_update,
            // Announcement Commands
            commands::announcement::announcement_get_state,
            commands::announcement::announcement_mark_as_read,
            commands::announcement::announcement_mark_all_as_read,
            commands::announcement::announcement_force_refresh,
            commands::announcement::announcement_dismiss,
            commands::announcement::announcement_dismiss_all,
            // Group Commands
            commands::group::get_group_settings,
            commands::group::save_group_settings,
            commands::group::set_model_group,
            commands::group::remove_model_group,
            commands::group::set_group_name,
            commands::group::delete_group,
            commands::group::update_group_order,
            commands::group::get_display_groups,
            // Codex Commands
            commands::codex::list_codex_accounts,
            commands::codex::get_current_codex_account,
            commands::codex::refresh_codex_account_profile,
            commands::codex::switch_codex_account,
            commands::codex::delete_codex_account,
            commands::codex::delete_codex_accounts,
            commands::codex::import_codex_from_local,
            commands::codex::import_codex_from_json,
            commands::codex::export_codex_accounts,
            commands::codex::import_codex_from_files,
            commands::codex::refresh_codex_quota,
            commands::codex::refresh_all_codex_quotas,
            commands::codex::refresh_current_codex_quota,
            commands::codex::codex_oauth_login_start,
            commands::codex::codex_oauth_login_completed,
            commands::codex::codex_oauth_submit_callback_url,
            commands::codex::codex_oauth_login_cancel,
            commands::codex::add_codex_account_with_token,
            commands::codex::add_codex_account_with_api_key,
            commands::codex::update_codex_account_name,
            commands::codex::update_codex_api_key_credentials,
            commands::codex::is_codex_oauth_port_in_use,
            commands::codex::close_codex_oauth_port,
            commands::codex::update_codex_account_tags,
            // GitHub Copilot Commands
            commands::github_copilot::list_github_copilot_accounts,
            commands::github_copilot::delete_github_copilot_account,
            commands::github_copilot::delete_github_copilot_accounts,
            commands::github_copilot::import_github_copilot_from_json,
            commands::github_copilot::export_github_copilot_accounts,
            commands::github_copilot::refresh_github_copilot_token,
            commands::github_copilot::refresh_all_github_copilot_tokens,
            commands::github_copilot::github_copilot_oauth_login_start,
            commands::github_copilot::github_copilot_oauth_login_complete,
            commands::github_copilot::github_copilot_oauth_login_cancel,
            commands::github_copilot::add_github_copilot_account_with_token,
            commands::github_copilot::update_github_copilot_account_tags,
            commands::github_copilot::get_github_copilot_accounts_index_path,
            commands::github_copilot::inject_github_copilot_to_vscode,
            // GitHub Copilot Instance Commands
            commands::github_copilot_instance::github_copilot_get_instance_defaults,
            commands::github_copilot_instance::github_copilot_list_instances,
            commands::github_copilot_instance::github_copilot_create_instance,
            commands::github_copilot_instance::github_copilot_update_instance,
            commands::github_copilot_instance::github_copilot_delete_instance,
            commands::github_copilot_instance::github_copilot_start_instance,
            commands::github_copilot_instance::github_copilot_stop_instance,
            commands::github_copilot_instance::github_copilot_open_instance_window,
            commands::github_copilot_instance::github_copilot_close_all_instances,
            // Windsurf Commands
            commands::windsurf::list_windsurf_accounts,
            commands::windsurf::delete_windsurf_account,
            commands::windsurf::delete_windsurf_accounts,
            commands::windsurf::import_windsurf_from_json,
            commands::windsurf::import_windsurf_from_local,
            commands::windsurf::export_windsurf_accounts,
            commands::windsurf::refresh_windsurf_token,
            commands::windsurf::refresh_all_windsurf_tokens,
            commands::windsurf::windsurf_oauth_login_start,
            commands::windsurf::windsurf_oauth_login_complete,
            commands::windsurf::windsurf_oauth_submit_callback_url,
            commands::windsurf::windsurf_oauth_login_cancel,
            commands::windsurf::add_windsurf_account_with_token,
            commands::windsurf::add_windsurf_account_with_password,
            commands::windsurf::update_windsurf_account_tags,
            commands::windsurf::get_windsurf_accounts_index_path,
            commands::windsurf::inject_windsurf_to_vscode,
            // Kiro Commands
            commands::kiro::list_kiro_accounts,
            commands::kiro::delete_kiro_account,
            commands::kiro::delete_kiro_accounts,
            commands::kiro::import_kiro_from_json,
            commands::kiro::import_kiro_from_local,
            commands::kiro::export_kiro_accounts,
            commands::kiro::refresh_kiro_token,
            commands::kiro::refresh_all_kiro_tokens,
            commands::kiro::kiro_oauth_login_start,
            commands::kiro::kiro_oauth_login_complete,
            commands::kiro::kiro_oauth_submit_callback_url,
            commands::kiro::kiro_oauth_login_cancel,
            commands::kiro::add_kiro_account_with_token,
            commands::kiro::update_kiro_account_tags,
            commands::kiro::get_kiro_accounts_index_path,
            commands::kiro::inject_kiro_to_vscode,
            // CodeBuddy Commands
            commands::codebuddy::list_codebuddy_accounts,
            commands::codebuddy::delete_codebuddy_account,
            commands::codebuddy::delete_codebuddy_accounts,
            commands::codebuddy::import_codebuddy_from_json,
            commands::codebuddy::import_codebuddy_from_local,
            commands::codebuddy::export_codebuddy_accounts,
            commands::codebuddy::refresh_codebuddy_token,
            commands::codebuddy::refresh_all_codebuddy_tokens,
            commands::codebuddy::codebuddy_oauth_login_start,
            commands::codebuddy::codebuddy_oauth_login_complete,
            commands::codebuddy::codebuddy_oauth_login_cancel,
            commands::codebuddy::add_codebuddy_account_with_token,
            commands::codebuddy::update_codebuddy_account_tags,
            commands::codebuddy::get_codebuddy_accounts_index_path,
            commands::codebuddy::inject_codebuddy_to_vscode,
            // CodeBuddy CN Commands
            commands::codebuddy_cn::list_codebuddy_cn_accounts,
            commands::codebuddy_cn::delete_codebuddy_cn_account,
            commands::codebuddy_cn::delete_codebuddy_cn_accounts,
            commands::codebuddy_cn::import_codebuddy_cn_from_json,
            commands::codebuddy_cn::import_codebuddy_cn_from_local,
            commands::codebuddy_cn::export_codebuddy_cn_accounts,
            commands::codebuddy_cn::refresh_codebuddy_cn_token,
            commands::codebuddy_cn::refresh_all_codebuddy_cn_tokens,
            commands::codebuddy_cn::codebuddy_cn_oauth_login_start,
            commands::codebuddy_cn::codebuddy_cn_oauth_login_complete,
            commands::codebuddy_cn::codebuddy_cn_oauth_login_cancel,
            commands::codebuddy_cn::add_codebuddy_cn_account_with_token,
            commands::codebuddy_cn::update_codebuddy_cn_account_tags,
            commands::codebuddy_cn::get_codebuddy_cn_accounts_index_path,
            commands::codebuddy_cn::inject_codebuddy_cn_to_vscode,
            commands::codebuddy_cn::sync_codebuddy_cn_to_workbuddy,
            // WorkBuddy Commands
            commands::workbuddy::list_workbuddy_accounts,
            commands::workbuddy::delete_workbuddy_account,
            commands::workbuddy::delete_workbuddy_accounts,
            commands::workbuddy::import_workbuddy_from_json,
            commands::workbuddy::import_workbuddy_from_local,
            commands::workbuddy::export_workbuddy_accounts,
            commands::workbuddy::refresh_workbuddy_token,
            commands::workbuddy::refresh_all_workbuddy_tokens,
            commands::workbuddy::workbuddy_oauth_login_start,
            commands::workbuddy::workbuddy_oauth_login_complete,
            commands::workbuddy::workbuddy_oauth_login_cancel,
            commands::workbuddy::add_workbuddy_account_with_token,
            commands::workbuddy::update_workbuddy_account_tags,
            commands::workbuddy::get_workbuddy_accounts_index_path,
            commands::workbuddy::inject_workbuddy_to_vscode,
            commands::workbuddy::sync_workbuddy_to_codebuddy_cn,
            // WorkBuddy Instance Commands
            commands::workbuddy_instance::workbuddy_get_instance_defaults,
            commands::workbuddy_instance::workbuddy_list_instances,
            commands::workbuddy_instance::workbuddy_create_instance,
            commands::workbuddy_instance::workbuddy_update_instance,
            commands::workbuddy_instance::workbuddy_delete_instance,
            commands::workbuddy_instance::workbuddy_start_instance,
            commands::workbuddy_instance::workbuddy_stop_instance,
            commands::workbuddy_instance::workbuddy_open_instance_window,
            commands::workbuddy_instance::workbuddy_close_all_instances,
            // CodeBuddy Instance Commands
            commands::codebuddy_instance::codebuddy_get_instance_defaults,
            commands::codebuddy_instance::codebuddy_list_instances,
            commands::codebuddy_instance::codebuddy_create_instance,
            commands::codebuddy_instance::codebuddy_update_instance,
            commands::codebuddy_instance::codebuddy_delete_instance,
            commands::codebuddy_instance::codebuddy_start_instance,
            commands::codebuddy_instance::codebuddy_stop_instance,
            commands::codebuddy_instance::codebuddy_open_instance_window,
            commands::codebuddy_instance::codebuddy_close_all_instances,
            // CodeBuddy CN Instance Commands
            commands::codebuddy_cn_instance::codebuddy_cn_get_instance_defaults,
            commands::codebuddy_cn_instance::codebuddy_cn_list_instances,
            commands::codebuddy_cn_instance::codebuddy_cn_create_instance,
            commands::codebuddy_cn_instance::codebuddy_cn_update_instance,
            commands::codebuddy_cn_instance::codebuddy_cn_delete_instance,
            commands::codebuddy_cn_instance::codebuddy_cn_start_instance,
            commands::codebuddy_cn_instance::codebuddy_cn_stop_instance,
            commands::codebuddy_cn_instance::codebuddy_cn_open_instance_window,
            commands::codebuddy_cn_instance::codebuddy_cn_close_all_instances,
            // Qoder Commands
            commands::qoder::list_qoder_accounts,
            commands::qoder::delete_qoder_account,
            commands::qoder::delete_qoder_accounts,
            commands::qoder::import_qoder_from_json,
            commands::qoder::import_qoder_from_local,
            commands::qoder::qoder_oauth_login_start,
            commands::qoder::qoder_oauth_login_peek,
            commands::qoder::qoder_oauth_login_complete,
            commands::qoder::qoder_oauth_login_cancel,
            commands::qoder::export_qoder_accounts,
            commands::qoder::refresh_qoder_token,
            commands::qoder::refresh_all_qoder_tokens,
            commands::qoder::inject_qoder_account,
            commands::qoder::update_qoder_account_tags,
            commands::qoder::get_qoder_accounts_index_path,
            // Qoder Instance Commands
            commands::qoder_instance::qoder_get_instance_defaults,
            commands::qoder_instance::qoder_list_instances,
            commands::qoder_instance::qoder_create_instance,
            commands::qoder_instance::qoder_update_instance,
            commands::qoder_instance::qoder_delete_instance,
            commands::qoder_instance::qoder_start_instance,
            commands::qoder_instance::qoder_stop_instance,
            commands::qoder_instance::qoder_open_instance_window,
            commands::qoder_instance::qoder_close_all_instances,
            // Trae Commands
            commands::trae::list_trae_accounts,
            commands::trae::delete_trae_account,
            commands::trae::delete_trae_accounts,
            commands::trae::import_trae_from_json,
            commands::trae::import_trae_from_local,
            commands::trae::trae_oauth_login_start,
            commands::trae::trae_oauth_login_complete,
            commands::trae::trae_oauth_submit_callback_url,
            commands::trae::trae_oauth_login_cancel,
            commands::trae::export_trae_accounts,
            commands::trae::refresh_trae_token,
            commands::trae::refresh_all_trae_tokens,
            commands::trae::add_trae_account_with_token,
            commands::trae::update_trae_account_tags,
            commands::trae::get_trae_accounts_index_path,
            commands::trae::inject_trae_account,
            // Trae Instance Commands
            commands::trae_instance::trae_get_instance_defaults,
            commands::trae_instance::trae_list_instances,
            commands::trae_instance::trae_create_instance,
            commands::trae_instance::trae_update_instance,
            commands::trae_instance::trae_delete_instance,
            commands::trae_instance::trae_start_instance,
            commands::trae_instance::trae_stop_instance,
            commands::trae_instance::trae_open_instance_window,
            commands::trae_instance::trae_close_all_instances,
            // Cursor Commands
            commands::cursor::list_cursor_accounts,
            commands::cursor::delete_cursor_account,
            commands::cursor::delete_cursor_accounts,
            commands::cursor::import_cursor_from_json,
            commands::cursor::import_cursor_from_local,
            commands::cursor::export_cursor_accounts,
            commands::cursor::refresh_cursor_token,
            commands::cursor::refresh_all_cursor_tokens,
            commands::cursor::add_cursor_account_with_token,
            commands::cursor::update_cursor_account_tags,
            commands::cursor::get_cursor_accounts_index_path,
            commands::cursor::cursor_oauth_login_start,
            commands::cursor::cursor_oauth_login_complete,
            commands::cursor::cursor_oauth_login_cancel,
            commands::cursor::inject_cursor_account,
            // Gemini Commands
            commands::gemini::list_gemini_accounts,
            commands::gemini::delete_gemini_account,
            commands::gemini::delete_gemini_accounts,
            commands::gemini::import_gemini_from_json,
            commands::gemini::import_gemini_from_local,
            commands::gemini::export_gemini_accounts,
            commands::gemini::refresh_gemini_token,
            commands::gemini::refresh_all_gemini_tokens,
            commands::gemini::gemini_oauth_login_start,
            commands::gemini::gemini_oauth_login_complete,
            commands::gemini::gemini_oauth_submit_callback_url,
            commands::gemini::gemini_oauth_login_cancel,
            commands::gemini::add_gemini_account_with_token,
            commands::gemini::update_gemini_account_tags,
            commands::gemini::get_gemini_accounts_index_path,
            commands::gemini::inject_gemini_account,
            // Gemini Instance Commands
            commands::gemini_instance::gemini_get_instance_defaults,
            commands::gemini_instance::gemini_list_instances,
            commands::gemini_instance::gemini_create_instance,
            commands::gemini_instance::gemini_update_instance,
            commands::gemini_instance::gemini_delete_instance,
            commands::gemini_instance::gemini_start_instance,
            commands::gemini_instance::gemini_stop_instance,
            commands::gemini_instance::gemini_open_instance_window,
            commands::gemini_instance::gemini_close_all_instances,
            commands::gemini_instance::gemini_get_instance_launch_command,
            commands::gemini_instance::gemini_execute_instance_launch_command,
            // Cursor Instance Commands
            commands::cursor_instance::cursor_get_instance_defaults,
            commands::cursor_instance::cursor_list_instances,
            commands::cursor_instance::cursor_create_instance,
            commands::cursor_instance::cursor_update_instance,
            commands::cursor_instance::cursor_delete_instance,
            commands::cursor_instance::cursor_start_instance,
            commands::cursor_instance::cursor_stop_instance,
            commands::cursor_instance::cursor_open_instance_window,
            commands::cursor_instance::cursor_close_all_instances,
            // Windsurf Instance Commands
            commands::windsurf_instance::windsurf_get_instance_defaults,
            commands::windsurf_instance::windsurf_list_instances,
            commands::windsurf_instance::windsurf_create_instance,
            commands::windsurf_instance::windsurf_update_instance,
            commands::windsurf_instance::windsurf_delete_instance,
            commands::windsurf_instance::windsurf_start_instance,
            commands::windsurf_instance::windsurf_stop_instance,
            commands::windsurf_instance::windsurf_open_instance_window,
            commands::windsurf_instance::windsurf_close_all_instances,
            // Kiro Instance Commands
            commands::kiro_instance::kiro_get_instance_defaults,
            commands::kiro_instance::kiro_list_instances,
            commands::kiro_instance::kiro_create_instance,
            commands::kiro_instance::kiro_update_instance,
            commands::kiro_instance::kiro_delete_instance,
            commands::kiro_instance::kiro_start_instance,
            commands::kiro_instance::kiro_stop_instance,
            commands::kiro_instance::kiro_open_instance_window,
            commands::kiro_instance::kiro_close_all_instances,
            // Codex Instance Commands
            commands::codex_instance::codex_get_instance_defaults,
            commands::codex_instance::codex_list_instances,
            commands::codex_instance::codex_create_instance,
            commands::codex_instance::codex_update_instance,
            commands::codex_instance::codex_delete_instance,
            commands::codex_instance::codex_start_instance,
            commands::codex_instance::codex_stop_instance,
            commands::codex_instance::codex_open_instance_window,
            commands::codex_instance::codex_close_all_instances,
            // Instance Commands
            commands::instance::get_instance_defaults,
            commands::instance::list_instances,
            commands::instance::create_instance,
            commands::instance::update_instance,
            commands::instance::delete_instance,
            commands::instance::start_instance,
            commands::instance::stop_instance,
            commands::instance::open_instance_window,
            commands::instance::close_all_instances,
            // Gateway Commands
            commands::gateway::start_gateway,
            commands::gateway::stop_gateway,
            commands::gateway::get_gateway_status,
            commands::gateway::get_gateway_config,
            commands::gateway::save_gateway_config,
            commands::gateway::list_gateway_accounts,
            commands::gateway::add_gateway_account,
            commands::gateway::delete_gateway_account,
            commands::gateway::import_gateway_accounts,
            commands::gateway::export_gateway_accounts,
            commands::gateway::list_api_keys,
            commands::gateway::create_api_key,
            commands::gateway::delete_api_key,
            commands::gateway::toggle_api_key,
            commands::gateway::list_request_logs,
            commands::gateway::get_request_log_summary,
            commands::gateway::clear_request_logs,
            commands::gateway::sync_accounts_to_gateway,
            commands::gateway::get_platform_account_stats,
            commands::gateway::sync_accounts_to_sub2api,
            commands::gateway::get_unified_account_pool,
            // Subprocess Commands
            commands::subprocess::start_sub2api,
            commands::subprocess::stop_sub2api,
            commands::subprocess::get_sub2api_status,
            commands::subprocess::get_sub2api_port,
            commands::subprocess::save_sub2api_config,
            // Cursor Welfare Commands
            commands::subprocess::start_cursor_welfare,
            commands::subprocess::stop_cursor_welfare,
            commands::subprocess::get_cursor_welfare_status,
            commands::subprocess::get_cursor_welfare_port,
            commands::subprocess::get_cursor_welfare_config,
            commands::subprocess::save_cursor_welfare_config,
            commands::subprocess::check_cursor_welfare_binary,
            commands::subprocess::share_cursor_welfare_to_sub2api,
            // Sub2api Proxy Commands
            commands::sub2api_proxy::sub2api_proxy,
            commands::sub2api_proxy::sub2api_login,
            commands::sub2api_proxy::sub2api_clear_auth,
            // === OpenCode Provider Commands ===
            commands::opencode::get_providers,
            commands::opencode::get_provider,
            commands::opencode::get_provider_for_apply,
            commands::opencode::add_provider,
            commands::opencode::update_provider,
            commands::opencode::delete_provider,
            commands::opencode::toggle_provider,
            commands::opencode::check_provider_applied,
            commands::opencode::apply_config,
            commands::opencode::import_local_provider_configs,
            commands::opencode::get_deployed_providers,
            commands::opencode::remove_deployed_provider,
            commands::opencode::import_deployed_provider,
            commands::opencode::add_provider_base_url,
            commands::opencode::remove_provider_base_url,
            commands::opencode::set_active_base_url,
            commands::opencode::update_url_latency,
            commands::opencode::auto_select_fastest_base_url,
            // === OpenCode Model Commands ===
            commands::opencode::get_models,
            commands::opencode::get_model,
            commands::opencode::add_model,
            commands::opencode::update_model,
            commands::opencode::delete_model,
            commands::opencode::fetch_site_models,
            commands::opencode::add_models_batch,
            commands::opencode::add_models_batch_detailed,
            // === OpenCode MCP Commands ===
            commands::opencode::get_mcp_servers,
            commands::opencode::get_mcp_server,
            commands::opencode::add_mcp_server,
            commands::opencode::update_mcp_server,
            commands::opencode::delete_mcp_server,
            commands::opencode::toggle_mcp_server,
            commands::opencode::sync_mcp_config,
            commands::opencode::get_recommended_mcp_servers,
            commands::opencode::add_recommended_mcp_servers,
            commands::opencode::check_mcp_server_health,
            commands::opencode::sync_mcp_to_apps,
            commands::opencode::get_apps_mcp_status,
            commands::opencode::import_mcp_from_apps,
            commands::opencode::get_managed_mcps,
            commands::opencode::get_mcp_stats,
            commands::opencode::toggle_mcp_app,
            commands::opencode::delete_mcp_from_all,
            // === OpenCode Skills Commands ===
            commands::opencode::get_installed_skills,
            commands::opencode::get_recommended_skills,
            commands::opencode::install_skills,
            commands::opencode::delete_skills,
            commands::opencode::read_skills_content,
            commands::opencode::get_managed_skills,
            commands::opencode::toggle_skill_tool,
            commands::opencode::get_skills_stats,
            commands::opencode::get_skills_repos,
            commands::opencode::add_skills_repo,
            commands::opencode::delete_skills_repo,
            commands::opencode::toggle_skills_repo,
            commands::opencode::toggle_skills_repo_enabled,
            commands::opencode::fetch_skills_from_repo,
            commands::opencode::discover_skills,
            // === OpenCode Rule Commands ===
            commands::opencode::get_installed_rules,
            commands::opencode::get_recommended_rules,
            commands::opencode::install_rule,
            commands::opencode::delete_rule,
            commands::opencode::read_rule_content,
            commands::opencode::save_rule_content,
            commands::opencode::toggle_rule_enabled,
            commands::opencode::get_managed_rules,
            commands::opencode::get_rule_stats,
            commands::opencode::toggle_rule_app,
            commands::opencode::delete_rule_from_all,
            // === OpenCode Status Commands ===
            commands::opencode::get_status,
            commands::opencode::get_version,
            commands::opencode::get_local_ip,
            commands::opencode::detect_cli_tools,
            commands::opencode::check_cli_latest_version,
            commands::opencode::update_cli_tool,
            // === OpenCode Backup Commands ===
            commands::opencode::create_backup,
            commands::opencode::export_backup,
            commands::opencode::export_backup_filtered,
            commands::opencode::preview_backup,
            commands::opencode::import_backup,
            // === OpenCode Settings Commands ===
            commands::opencode::get_app_settings,
            commands::opencode::save_app_settings,
            commands::opencode::get_close_action,
            commands::opencode::set_close_action,
            commands::opencode::handle_close_choice,
            commands::opencode::get_autostart_enabled,
            commands::opencode::set_autostart_enabled,
            commands::opencode::detect_env_conflicts,
            commands::opencode::refresh_tray_menu,
            commands::opencode::get_cc_switch_providers,
            commands::opencode::delete_cc_switch_provider,
            commands::opencode::delete_open_switch_provider,
            // === OpenCode OhMy Commands ===
            commands::opencode::check_ohmy_status,
            commands::opencode::get_ohmy_version_info,
            commands::opencode::get_available_models,
            commands::opencode::get_agent_infos,
            commands::opencode::install_bun,
            commands::opencode::install_ohmy,
            commands::opencode::save_ohmy_config,
            commands::opencode::install_and_configure,
            commands::opencode::uninstall_ohmy,
            commands::opencode::update_ohmy,
            // === OpenCode DeepLink Commands ===
            commands::opencode::parse_deep_link,
            commands::opencode::generate_deep_link,
            // === OpenCode Claude Code Commands ===
            commands::opencode::get_claude_code_status,
            commands::opencode::get_claude_code_settings,
            commands::opencode::save_claude_code_settings,
            commands::opencode::set_claude_code_api_key,
            commands::opencode::set_claude_code_base_url,
            commands::opencode::set_claude_code_model,
            commands::opencode::apply_provider_to_claude_code,
            commands::opencode::get_claude_code_mcp_servers,
            commands::opencode::add_claude_code_mcp_server,
            commands::opencode::remove_claude_code_mcp_server,
            commands::opencode::sync_mcp_to_claude_code,
            commands::opencode::get_claude_md,
            commands::opencode::save_claude_md,
            commands::opencode::clear_claude_code_config,
            commands::opencode::set_claude_code_skip_onboarding,
            commands::opencode::clear_claude_code_skip_onboarding,
            commands::opencode::get_claude_code_skip_onboarding,
            // === OpenCode Codex Config Commands ===
            commands::opencode::get_codex_status,
            commands::opencode::get_codex_providers,
            commands::opencode::add_codex_provider,
            commands::opencode::remove_codex_provider,
            commands::opencode::apply_provider_to_codex,
            commands::opencode::get_codex_mcp_servers,
            commands::opencode::add_codex_mcp_server,
            commands::opencode::remove_codex_mcp_server,
            commands::opencode::sync_mcp_to_codex,
            commands::opencode::get_agents_md,
            commands::opencode::save_agents_md,
            commands::opencode::set_codex_api_key_skip_oauth,
            commands::opencode::clear_codex_api_key,
            commands::opencode::get_codex_api_key,
            // === OpenCode Gemini Config Commands ===
            commands::opencode::get_gemini_status,
            commands::opencode::get_gemini_settings,
            commands::opencode::save_gemini_settings,
            commands::opencode::set_gemini_api_key,
            commands::opencode::set_gemini_base_url,
            commands::opencode::set_gemini_model,
            commands::opencode::set_gemini_auth_mode,
            commands::opencode::apply_provider_to_gemini,
            commands::opencode::get_gemini_mcp_servers,
            commands::opencode::add_gemini_mcp_server,
            commands::opencode::remove_gemini_mcp_server,
            commands::opencode::sync_mcp_to_gemini,
            commands::opencode::get_gemini_md,
            commands::opencode::save_gemini_md,
            commands::opencode::clear_gemini_config,
            commands::opencode::set_gemini_api_key_auth_mode,
            commands::opencode::set_gemini_oauth_auth_mode,
            commands::opencode::get_gemini_auth_selected_type,
            commands::opencode::clear_gemini_auth_selected_type,
            // === OpenCode Prompts Commands ===
            commands::opencode::get_prompts_status,
            commands::opencode::get_prompt,
            commands::opencode::save_prompt,
            commands::opencode::sync_prompt,
            commands::opencode::delete_prompt,
            commands::opencode::get_prompt_presets,
            // === OpenCode Speed Test Commands ===
            commands::opencode::test_endpoint_latency,
            commands::opencode::batch_test_endpoint,
            commands::opencode::test_multiple_providers,
            commands::opencode::test_provider_urls,
            commands::opencode::test_and_auto_select_fastest,
            // === OpenCode Usage Commands ===
            commands::opencode::get_usage_summary,
            commands::opencode::get_usage_trend,
            commands::opencode::add_usage_record,
            commands::opencode::clear_usage_stats,
            commands::opencode::get_usage_by_provider,
            commands::opencode::get_model_pricing_list,
            commands::opencode::update_model_pricing,
            commands::opencode::add_model_pricing,
            commands::opencode::delete_model_pricing,
            commands::opencode::reset_model_pricing,
            commands::opencode::get_provider_model_pricing,
            commands::opencode::get_all_provider_pricing,
            commands::opencode::set_provider_model_pricing,
            commands::opencode::delete_provider_model_pricing,
            commands::opencode::get_pricing_providers,
            commands::opencode::diagnose_usage_data,
            // === OpenCode Proxy Commands ===
            commands::opencode::init_proxy_service,
            commands::opencode::start_proxy,
            commands::opencode::stop_proxy,
            commands::opencode::get_proxy_status,
            commands::opencode::is_proxy_running,
            commands::opencode::start_proxy_with_takeover,
            commands::opencode::stop_proxy_with_restore,
            commands::opencode::get_takeover_status,
            commands::opencode::set_takeover_for_app,
            commands::opencode::get_proxy_config,
            commands::opencode::update_proxy_config,
            commands::opencode::get_proxy_usage_summary,
            commands::opencode::get_proxy_usage_trend,
            commands::opencode::get_proxy_usage_trend_by_model,
            commands::opencode::get_provider_stats,
            commands::opencode::get_project_stats,
            commands::opencode::clear_proxy_usage_stats,
            // === OpenCode Unified Config Commands ===
            commands::opencode::get_open_switch_providers,
            commands::opencode::get_open_switch_provider,
            commands::opencode::add_open_switch_provider,
            commands::opencode::update_open_switch_provider,
            commands::opencode::remove_open_switch_provider,
            commands::opencode::apply_open_switch_provider,
            commands::opencode::import_to_open_switch,
            commands::opencode::get_open_switch_config_path,
            // === OpenCode Local Logs Commands ===
            commands::opencode::debug_cursor_db_keys,
            commands::opencode::debug_cursor_message_fields,
            commands::opencode::get_cursor_conversation_stats,
            commands::opencode::scan_local_logs,
            commands::opencode::import_local_logs,
            commands::opencode::clear_local_logs,
            commands::opencode::auto_import_local_logs,
            commands::opencode::get_log_retention,
            commands::opencode::set_log_retention,
            commands::opencode::cleanup_old_logs,
            commands::opencode::get_session_stats_summary,
            commands::opencode::get_tool_call_stats,
            // === OpenCode DevEnv Commands ===
            commands::opencode::detect_all_dev_envs,
            commands::opencode::detect_single_dev_env,
            commands::opencode::get_installed_versions,
            commands::opencode::switch_env_version,
            commands::opencode::install_env_version,
            commands::opencode::install_version_manager,
            commands::opencode::uninstall_env_version,
            commands::opencode::uninstall_version_manager,
            // === OpenCode Chat Migration Commands ===
            commands::opencode::scan_chat_sources,
            commands::opencode::extract_conversations,
            commands::opencode::export_conversations,
            commands::opencode::import_migration_file,
            commands::opencode::get_migrated_conversations,
            commands::opencode::clear_migrated_conversations,
            // === OpenCode Windsurf Config Commands ===
            commands::opencode::get_windsurf_status,
            commands::opencode::get_windsurf_mcp_servers,
            commands::opencode::add_windsurf_mcp_server,
            commands::opencode::remove_windsurf_mcp_server,
            commands::opencode::sync_mcp_to_windsurf,
            commands::opencode::import_mcp_from_windsurf,
            // === OpenCode Augment Commands ===
            commands::opencode::get_augment_status,
            commands::opencode::ensure_augment_rules_dir,
            // === OpenCode Warp Commands ===
            commands::opencode::get_warp_status,
            commands::opencode::get_warp_usage_from_local_db,
            commands::opencode::ensure_warp_rules_dir,
            // === OpenCode Kiro Config Commands ===
            commands::opencode::get_kiro_status,
            commands::opencode::get_kiro_mcp_servers,
            commands::opencode::add_kiro_mcp_server,
            commands::opencode::remove_kiro_mcp_server,
            commands::opencode::sync_mcp_to_kiro,
            commands::opencode::import_mcp_from_kiro,
            commands::opencode::ensure_kiro_rules_dir,
            // === OpenCode Antigravity Config Commands ===
            commands::opencode::get_antigravity_status,
            commands::opencode::get_antigravity_mcp_servers,
            commands::opencode::add_antigravity_mcp_server,
            commands::opencode::remove_antigravity_mcp_server,
            commands::opencode::sync_mcp_to_antigravity,
            commands::opencode::import_mcp_from_antigravity,
            commands::opencode::ensure_antigravity_rules_dir,
            // === OpenCode OpenClaw Config Commands ===
            commands::opencode::get_openclaw_status,
            commands::opencode::get_openclaw_config_path,
            commands::opencode::get_openclaw_agents_content,
            commands::opencode::save_openclaw_agents_content,
            commands::opencode::get_openclaw_soul_content,
            commands::opencode::save_openclaw_soul_content,
            commands::opencode::apply_provider_to_openclaw,
            commands::opencode::get_claude_config_path,
            // === Cursor Welfare Provider Fan-out ===
            commands::opencode::apply_cursor_welfare_to_tools,
            // === Session Manager Commands ===
            commands::session::list_sessions,
            commands::session::get_session_messages,
            commands::session::search_sessions,
            commands::session::delete_session,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        #[cfg(target_os = "macos")]
        {
            if let RunEvent::Reopen { .. } = event {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app_handle, event);
        }
    });
}
