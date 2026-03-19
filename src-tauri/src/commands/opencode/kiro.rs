// Kiro 相关 Tauri 命令

use crate::modules::opencode_config::kiro_manager::{
    KiroConfigManager, KiroMcpServer,
};
use serde::Serialize;
use std::collections::HashMap;

/// Kiro 配置状态
#[derive(Debug, Serialize)]
pub struct KiroStatus {
    pub is_installed: bool,
    pub is_mcp_configured: bool,
    pub mcp_server_count: usize,
    pub config_dir: String,
    pub rules_dir: String,
    pub rules_count: usize,
}

/// 获取 Kiro 配置状态
#[tauri::command]
pub async fn get_kiro_status() -> Result<KiroStatus, String> {
    let manager = KiroConfigManager::new()?;

    let mcp_count = if manager.is_mcp_configured() {
        manager.get_mcp_count().unwrap_or(0)
    } else {
        0
    };

    Ok(KiroStatus {
        is_installed: manager.is_installed(),
        is_mcp_configured: manager.is_mcp_configured(),
        mcp_server_count: mcp_count,
        config_dir: manager.get_config_dir().to_string_lossy().to_string(),
        rules_dir: manager.get_rules_dir().to_string_lossy().to_string(),
        rules_count: manager.get_rules_count(),
    })
}

/// 获取 Kiro MCP 服务器列表
#[tauri::command]
pub async fn get_kiro_mcp_servers() -> Result<HashMap<String, KiroMcpServer>, String> {
    let manager = KiroConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Kiro MCP 服务器
#[tauri::command]
pub async fn add_kiro_mcp_server(name: String, server: KiroMcpServer) -> Result<(), String> {
    let manager = KiroConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Kiro MCP 服务器
#[tauri::command]
pub async fn remove_kiro_mcp_server(name: String) -> Result<(), String> {
    let manager = KiroConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Kiro
#[tauri::command]
pub async fn sync_mcp_to_kiro(servers: HashMap<String, KiroMcpServer>) -> Result<(), String> {
    let manager = KiroConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 从 Kiro 导入 MCP 配置
#[tauri::command]
pub async fn import_mcp_from_kiro() -> Result<HashMap<String, KiroMcpServer>, String> {
    let manager = KiroConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 确保 Kiro steering/rules 目录存在
#[tauri::command]
pub async fn ensure_kiro_rules_dir() -> Result<String, String> {
    let manager = KiroConfigManager::new()?;
    manager.ensure_rules_dir()?;
    Ok(manager.get_rules_dir().to_string_lossy().to_string())
}
