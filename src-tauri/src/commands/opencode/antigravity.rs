// Google Antigravity (反重力) 相关 Tauri 命令

use crate::modules::opencode_config::antigravity_manager::{
    AntigravityConfigManager, AntigravityMcpServer,
};
use serde::Serialize;
use std::collections::HashMap;

/// Antigravity 配置状态
#[derive(Debug, Serialize)]
pub struct AntigravityStatus {
    pub is_installed: bool,
    pub is_mcp_configured: bool,
    pub mcp_server_count: usize,
    pub config_dir: String,
    pub rules_dir: String,
    pub rules_count: usize,
}

/// 获取 Antigravity 配置状态
#[tauri::command]
pub async fn get_antigravity_status() -> Result<AntigravityStatus, String> {
    let manager = AntigravityConfigManager::new()?;

    let mcp_count = if manager.is_mcp_configured() {
        manager.get_mcp_count().unwrap_or(0)
    } else {
        0
    };

    Ok(AntigravityStatus {
        is_installed: manager.is_installed(),
        is_mcp_configured: manager.is_mcp_configured(),
        mcp_server_count: mcp_count,
        config_dir: manager.get_config_dir().to_string_lossy().to_string(),
        rules_dir: manager.get_rules_dir().to_string_lossy().to_string(),
        rules_count: manager.get_rules_count(),
    })
}

/// 获取 Antigravity MCP 服务器列表
#[tauri::command]
pub async fn get_antigravity_mcp_servers() -> Result<HashMap<String, AntigravityMcpServer>, String> {
    let manager = AntigravityConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Antigravity MCP 服务器
#[tauri::command]
pub async fn add_antigravity_mcp_server(name: String, server: AntigravityMcpServer) -> Result<(), String> {
    let manager = AntigravityConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Antigravity MCP 服务器
#[tauri::command]
pub async fn remove_antigravity_mcp_server(name: String) -> Result<(), String> {
    let manager = AntigravityConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Antigravity
#[tauri::command]
pub async fn sync_mcp_to_antigravity(servers: HashMap<String, AntigravityMcpServer>) -> Result<(), String> {
    let manager = AntigravityConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 从 Antigravity 导入 MCP 配置
#[tauri::command]
pub async fn import_mcp_from_antigravity() -> Result<HashMap<String, AntigravityMcpServer>, String> {
    let manager = AntigravityConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 确保 Antigravity rules 目录存在
#[tauri::command]
pub async fn ensure_antigravity_rules_dir() -> Result<String, String> {
    let manager = AntigravityConfigManager::new()?;
    manager.ensure_rules_dir()?;
    Ok(manager.get_rules_dir().to_string_lossy().to_string())
}
