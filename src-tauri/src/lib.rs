// Open Switch Tauri 库入口

pub mod commands;
pub mod config;
pub mod error;

use std::sync::Mutex;
use config::ConfigManager;

/// 运行 Tauri 应用
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化配置管理器
    let config_manager = ConfigManager::new()
        .expect("初始化配置管理器失败");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(config_manager))
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::get_providers,
            commands::get_provider,
            commands::add_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::check_provider_applied,
            commands::apply_config,
            // Model commands
            commands::get_models,
            commands::add_model,
            commands::delete_model,
            commands::fetch_site_models,
            commands::add_models_batch,
            // MCP commands
            commands::get_mcp_servers,
            commands::get_mcp_server,
            commands::add_mcp_server,
            commands::update_mcp_server,
            commands::delete_mcp_server,
            commands::toggle_mcp_server,
            commands::sync_mcp_config,
            // Status commands
            commands::get_status,
            commands::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
