// Ai Switch Tauri 库入口
// v1.5.0 - 添加 Cursor 对话统计功能

pub mod commands;
pub mod config;
pub mod database;
pub mod error;
pub mod proxy;

use std::sync::{Arc, Mutex};
use config::ConfigManager;
use database::Database;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::sync::RwLock;

/// 托盘图标状态包装器（用于在应用生命周期内保持托盘图标存活）
pub struct TrayState(pub Mutex<TrayIcon>);

/// 构建包含 Provider 列表的托盘菜单
fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    // 获取 Provider 列表
    let providers = {
        let config_state = app.state::<Mutex<ConfigManager>>();
        let manager = config_state.lock().map_err(|e| format!("Lock error: {}", e))?;
        manager.opencode().get_all_providers().unwrap_or_default()
    };
    
    let mut items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    
    // 为每个启用的 Provider 创建菜单项
    for (name, provider) in &providers {
        if provider.enabled {
            let check_mark = "✓ ";
            let label = format!("{}{}", check_mark, name);
            let item = MenuItem::with_id(
                app,
                format!("provider_{}", name),
                &label,
                true,
                None::<&str>
            )?;
            items.push(item);
        }
    }
    
    // 为未启用的 Provider 也创建菜单项（无勾选）
    for (name, provider) in &providers {
        if !provider.enabled {
            let label = format!("  {}", name);
            let item = MenuItem::with_id(
                app,
                format!("provider_{}", name),
                &label,
                true,
                None::<&str>
            )?;
            items.push(item);
        }
    }
    
    // 如果有 Provider，添加分隔线
    let separator = if !providers.is_empty() {
        Some(PredefinedMenuItem::separator(app)?)
    } else {
        None
    };
    
    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    
    // 构建菜单项引用列表
    let mut menu_items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = Vec::new();
    for item in &items {
        menu_items.push(item);
    }
    if let Some(ref sep) = separator {
        menu_items.push(sep);
    }
    menu_items.push(&show_item);
    menu_items.push(&quit_item);
    
    Menu::with_items(app, &menu_items).map_err(|e| e.into())
}

/// 处理 Provider 点击事件
fn handle_provider_click(app: &AppHandle, provider_name: &str) {
    // 先切换 Provider 启用状态
    {
        let config_state = app.state::<Mutex<ConfigManager>>();
        if let Ok(mut manager) = config_state.lock() {
            // 获取当前状态
            let current_enabled = manager.opencode()
                .get_provider(provider_name)
                .ok()
                .flatten()
                .map(|p| p.enabled)
                .unwrap_or(false);
            
            // 切换到相反状态
            let _ = manager.opencode_mut().toggle_provider(provider_name, !current_enabled);
        };
    }
    
    // 刷新托盘菜单
    refresh_tray_menu(app);
    
    // 通知前端刷新
    let _ = app.emit("providers-changed", ());
}

/// 刷新托盘菜单
pub fn refresh_tray_menu(app: &AppHandle) {
    if let Some(tray_state) = app.try_state::<TrayState>() {
        if let Ok(tray) = tray_state.0.lock() {
            if let Ok(menu) = build_tray_menu(app) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }
}

/// 运行 Tauri 应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置管理器
    let config_manager = ConfigManager::new()?;
    
    // 初始化数据库
    let database = Database::open()
        .map_err(|e| format!("数据库初始化失败: {e}"))?;
    let db_arc = Arc::new(database);
    
    // 代理服务状态
    let proxy_service_state = commands::ProxyServiceState(Arc::new(RwLock::new(None)));
    
    tauri::Builder::default()
        // 单实例插件必须首先注册，以便在第二个实例启动时能够正确拦截
        // 这对于 Windows/Linux 上的深链接功能至关重要
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // 单实例模式：当第二个实例启动时，将参数传递给第一个实例
            // 这对于深链接很重要，因为 Windows/Linux 上深链接会启动新进程
            if let Some(url) = argv.iter().find(|arg| arg.starts_with("aiswitch://")) {
                let _ = app.emit("deep-link-received", url.clone());
            }
            // 聚焦主窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]) // 启动参数，最小化启动
        ))
        .manage(Mutex::new(config_manager))
        .manage(db_arc)
        .manage(proxy_service_state)
        .setup(|app| {
            // 创建动态托盘菜单（包含 Provider 列表）
            let menu = build_tray_menu(app.handle())?;
            
            // 创建托盘图标（安全处理图标不存在的情况）
            let icon = app.default_window_icon()
                .ok_or_else(|| Box::<dyn std::error::Error>::from("未找到应用图标，请检查 tauri.conf.json 中的 icon 配置"))?
                .clone();
            let tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Ai Switch")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    let id = event.id.as_ref();
                    match id {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {
                            // 处理 Provider 点击事件
                            if id.starts_with("provider_") {
                                let provider_name = id.strip_prefix("provider_").unwrap_or("");
                                if !provider_name.is_empty() {
                                    handle_provider_click(app, provider_name);
                                }
                            }
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    // 左键点击托盘图标时显示窗口
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            
            // 将托盘图标存储到应用状态中，防止被释放
            app.manage(TrayState(Mutex::new(tray)));
            
            // 深链接处理说明：
            // 初始深链接和后续深链接都由前端通过 @tauri-apps/plugin-deep-link 直接处理
            // - 初始深链接：前端使用 getCurrent() API 获取
            // - 后续深链接：前端使用 onOpenUrl() API 监听
            // 这样可以避免在 setup 阶段发送事件时前端监听器尚未注册的问题
            
            Ok(())
        })
        // 注意：窗口关闭事件由前端通过 onCloseRequested API 处理
        // 前端会调用 get_close_action 获取设置，然后调用 handle_close_choice 执行操作
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::get_providers,
            commands::get_provider,
            commands::get_provider_for_apply,
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::toggle_provider,
            commands::check_provider_applied,
            commands::apply_config,
            commands::get_deployed_providers,
            commands::remove_deployed_provider,
            commands::import_deployed_provider,
            // Multi-URL management commands
            commands::add_provider_base_url,
            commands::remove_provider_base_url,
            commands::set_active_base_url,
            commands::update_url_latency,
            commands::auto_select_fastest_base_url,
            // Model commands
            commands::get_models,
            commands::get_model,
            commands::add_model,
            commands::update_model,
            commands::delete_model,
            commands::fetch_site_models,
            commands::add_models_batch,
            commands::add_models_batch_detailed,
            // MCP commands
            commands::get_mcp_servers,
            commands::get_mcp_server,
            commands::add_mcp_server,
            commands::update_mcp_server,
            commands::delete_mcp_server,
            commands::toggle_mcp_server,
            commands::sync_mcp_config,
            commands::get_recommended_mcp_servers,
            commands::add_recommended_mcp_servers,
            commands::check_mcp_server_health,
            commands::sync_mcp_to_apps,
            commands::get_apps_mcp_status,
            commands::import_mcp_from_apps,
            commands::get_managed_mcps,
            commands::get_mcp_stats,
            commands::toggle_mcp_app,
            commands::delete_mcp_from_all,
            // skills commands
            commands::get_installed_skills,
            commands::get_recommended_skills,
            commands::install_skills,
            commands::delete_skills,
            commands::read_skills_content,
            commands::get_managed_skills,
            commands::toggle_skill_tool,
            commands::get_skills_stats,
            // skills repository commands
            commands::get_skills_repos,
            commands::add_skills_repo,
            commands::delete_skills_repo,
            commands::toggle_skills_repo,
            commands::toggle_skills_repo_enabled,
            commands::fetch_skills_from_repo,
            commands::discover_skills,
            // Rule commands
            commands::get_installed_rules,
            commands::get_recommended_rules,
            commands::install_rule,
            commands::delete_rule,
            commands::read_rule_content,
            commands::save_rule_content,
            commands::toggle_rule_enabled,
            commands::get_managed_rules,
            commands::get_rule_stats,
            commands::toggle_rule_app,
            commands::delete_rule_from_all,
            // Status commands
            commands::get_status,
            commands::get_version,
            commands::get_local_ip,
            // Backup commands
            commands::create_backup,
            commands::export_backup,
            commands::preview_backup,
            commands::import_backup,
            // Settings commands
            commands::get_app_settings,
            commands::save_app_settings,
            commands::get_close_action,
            commands::set_close_action,
            commands::handle_close_choice,
            // Autostart commands
            commands::get_autostart_enabled,
            commands::set_autostart_enabled,
            // Environment conflict detection
            commands::detect_env_conflicts,
            // Tray menu refresh
            commands::refresh_tray_menu,
            // 外部配置读取
            commands::get_cc_switch_providers,
            commands::delete_cc_switch_provider,
            commands::delete_open_switch_provider,
            // oh-my-opencode commands
            commands::check_ohmy_status,
            commands::get_available_models,
            commands::get_agent_infos,
            commands::install_bun,
            commands::install_ohmy,
            commands::save_ohmy_config,
            commands::install_and_configure,
            commands::uninstall_ohmy,
            // Deep link commands
            commands::parse_deep_link,
            commands::generate_deep_link,
            // Claude Code commands
            commands::get_claude_code_status,
            commands::get_claude_code_settings,
            commands::save_claude_code_settings,
            commands::set_claude_code_api_key,
            commands::set_claude_code_base_url,
            commands::set_claude_code_model,
            commands::apply_provider_to_claude_code,
            commands::get_claude_code_mcp_servers,
            commands::add_claude_code_mcp_server,
            commands::remove_claude_code_mcp_server,
            commands::sync_mcp_to_claude_code,
            commands::get_claude_md,
            commands::save_claude_md,
            commands::clear_claude_code_config,
            // Codex commands
            commands::get_codex_status,
            commands::get_codex_providers,
            commands::add_codex_provider,
            commands::remove_codex_provider,
            commands::apply_provider_to_codex,
            commands::get_codex_mcp_servers,
            commands::add_codex_mcp_server,
            commands::remove_codex_mcp_server,
            commands::sync_mcp_to_codex,
            commands::get_agents_md,
            commands::save_agents_md,
            // Gemini commands
            commands::get_gemini_status,
            commands::get_gemini_settings,
            commands::save_gemini_settings,
            commands::set_gemini_api_key,
            commands::set_gemini_base_url,
            commands::set_gemini_model,
            commands::set_gemini_auth_mode,
            commands::apply_provider_to_gemini,
            commands::get_gemini_mcp_servers,
            commands::add_gemini_mcp_server,
            commands::remove_gemini_mcp_server,
            commands::sync_mcp_to_gemini,
            commands::get_gemini_md,
            commands::save_gemini_md,
            commands::clear_gemini_config,
            // Prompts commands
            commands::get_prompts_status,
            commands::get_prompt,
            commands::save_prompt,
            commands::sync_prompt,
            commands::delete_prompt,
            commands::get_prompt_presets,
            // Speed test commands
            commands::test_endpoint_latency,
            commands::batch_test_endpoint,
            commands::test_multiple_providers,
            commands::test_provider_urls,
            commands::test_and_auto_select_fastest,
            // Usage statistics commands
            commands::get_usage_summary,
            commands::get_usage_trend,
            commands::add_usage_record,
            commands::clear_usage_stats,
            commands::get_usage_by_provider,
            // Model pricing commands
            commands::get_model_pricing_list,
            commands::update_model_pricing,
            commands::add_model_pricing,
            commands::delete_model_pricing,
            commands::reset_model_pricing,
            // Provider model pricing commands
            commands::get_provider_model_pricing,
            commands::get_all_provider_pricing,
            commands::set_provider_model_pricing,
            commands::delete_provider_model_pricing,
            commands::get_pricing_providers,
            // Diagnostics commands
            commands::diagnose_usage_data,
            // Proxy commands
            commands::init_proxy_service,
            commands::start_proxy,
            commands::stop_proxy,
            commands::get_proxy_status,
            commands::is_proxy_running,
            commands::start_proxy_with_takeover,
            commands::stop_proxy_with_restore,
            commands::get_takeover_status,
            commands::set_takeover_for_app,
            commands::get_proxy_config,
            commands::update_proxy_config,
            commands::get_proxy_usage_summary,
            commands::get_proxy_usage_trend,
            commands::get_proxy_usage_trend_by_model,
            commands::get_provider_stats,
            commands::clear_proxy_usage_stats,
            // Ai Switch unified config commands
            commands::get_open_switch_providers,
            commands::get_open_switch_provider,
            commands::add_open_switch_provider,
            commands::update_open_switch_provider,
            commands::remove_open_switch_provider,
            commands::apply_open_switch_provider,
            commands::import_to_open_switch,
            commands::get_open_switch_config_path,
            // Local logs import commands
            commands::debug_cursor_db_keys,
            commands::debug_cursor_message_fields,
            commands::get_cursor_conversation_stats,
            commands::scan_local_logs,
            commands::import_local_logs,
            commands::clear_local_logs,
            commands::auto_import_local_logs,
            // Log retention commands
            commands::get_log_retention,
            commands::set_log_retention,
            commands::cleanup_old_logs,
            // Session stats commands
            commands::get_session_stats_summary,
            commands::get_tool_call_stats,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
