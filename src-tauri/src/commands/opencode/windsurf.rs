// Windsurf 相关 Tauri 命令

use crate::modules::opencode_config::windsurf_manager::{
    WindsurfConfigManager, WindsurfMcpServer,
};
use serde::Serialize;
use std::collections::HashMap;

/// Windsurf 配置状态
#[derive(Debug, Serialize)]
pub struct WindsurfStatus {
    pub is_installed: bool,
    pub is_mcp_configured: bool,
    pub mcp_server_count: usize,
    pub config_dir: String,
    pub rules_dir: String,
    pub skills_dir: String,
}

/// 获取 Windsurf 配置状态
#[tauri::command]
pub async fn get_windsurf_status() -> Result<WindsurfStatus, String> {
    let manager = WindsurfConfigManager::new()?;

    let mcp_count = if manager.is_mcp_configured() {
        manager.get_mcp_count().unwrap_or(0)
    } else {
        0
    };

    Ok(WindsurfStatus {
        is_installed: manager.is_installed(),
        is_mcp_configured: manager.is_mcp_configured(),
        mcp_server_count: mcp_count,
        config_dir: manager.get_config_dir().to_string_lossy().to_string(),
        rules_dir: manager.get_rules_dir().to_string_lossy().to_string(),
        skills_dir: manager.get_skills_dir().to_string_lossy().to_string(),
    })
}

/// 获取 Windsurf MCP 服务器列表
#[tauri::command]
pub async fn get_windsurf_mcp_servers() -> Result<HashMap<String, WindsurfMcpServer>, String> {
    let manager = WindsurfConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Windsurf MCP 服务器
#[tauri::command]
pub async fn add_windsurf_mcp_server(name: String, server: WindsurfMcpServer) -> Result<(), String> {
    let manager = WindsurfConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Windsurf MCP 服务器
#[tauri::command]
pub async fn remove_windsurf_mcp_server(name: String) -> Result<(), String> {
    let manager = WindsurfConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Windsurf
#[tauri::command]
pub async fn sync_mcp_to_windsurf(servers: HashMap<String, WindsurfMcpServer>) -> Result<(), String> {
    let manager = WindsurfConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 从 Windsurf 导入 MCP 配置
#[tauri::command]
pub async fn import_mcp_from_windsurf() -> Result<HashMap<String, WindsurfMcpServer>, String> {
    let manager = WindsurfConfigManager::new()?;
    manager.get_mcp_servers()
}
