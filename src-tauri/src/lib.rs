// Open Switch Tauri 库入口

pub mod commands;
pub mod config;
pub mod error;

use std::sync::Mutex;
use config::ConfigManager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// 托盘图标状态包装器（用于在应用生命周期内保持托盘图标存活）
pub struct TrayState(pub TrayIcon);

/// 运行 Tauri 应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化配置管理器
    let config_manager = ConfigManager::new()
        .expect("初始化配置管理器失败");
    
    tauri::Builder::default()
        // 单实例插件必须首先注册，以便在第二个实例启动时能够正确拦截
        // 这对于 Windows/Linux 上的深链接功能至关重要
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            // 单实例模式：当第二个实例启动时，将参数传递给第一个实例
            // 这对于深链接很重要，因为 Windows/Linux 上深链接会启动新进程
            if let Some(url) = argv.iter().find(|arg| arg.starts_with("openswitch://")) {
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
        .manage(Mutex::new(config_manager))
        .setup(|app| {
            // 创建托盘菜单（开发和生产模式都需要托盘图标以支持最小化到托盘功能）
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;
            
            // 创建托盘图标（安全处理图标不存在的情况）
            let icon = app.default_window_icon()
                .ok_or_else(|| Box::<dyn std::error::Error>::from("未找到应用图标，请检查 tauri.conf.json 中的 icon 配置"))?
                .clone();
            let tray = TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Open Switch")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
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
            app.manage(TrayState(tray));
            
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
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::toggle_provider,
            commands::check_provider_applied,
            commands::apply_config,
            commands::get_deployed_providers,
            commands::remove_deployed_provider,
            commands::import_deployed_provider,
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
            // skills commands
            commands::get_installed_skills,
            commands::get_recommended_skills,
            commands::install_skills,
            commands::delete_skills,
            commands::read_skills_content,
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
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
