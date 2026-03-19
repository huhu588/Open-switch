// Codex 相关 Tauri 命令

use crate::modules::opencode_config::codex_manager::{
    CodexConfigManager, CodexMcpServer, CodexModelProvider, CodexProvider,
};
use serde::Serialize;
use std::collections::HashMap;

/// Codex 配置状态
#[derive(Debug, Serialize)]
pub struct CodexStatus {
    pub is_configured: bool,
    pub has_auth: bool,
    pub provider_count: usize,
    pub mcp_server_count: usize,
    pub credentials_store: Option<String>,
}

/// 获取 Codex 配置状态
#[tauri::command]
pub async fn get_codex_status() -> Result<CodexStatus, String> {
    let manager = CodexConfigManager::new()?;
    
    let auth = manager.read_auth()?;
    let has_auth = auth.access_token.is_some();
    
    let config = manager.read_config()?;
    
    Ok(CodexStatus {
        is_configured: manager.is_configured(),
        has_auth,
        provider_count: config.model_providers.len(),
        mcp_server_count: config.mcp_servers.len(),
        credentials_store: config.cli_auth_credentials_store,
    })
}

/// 获取 Codex 模型提供商列表
#[tauri::command]
pub async fn get_codex_providers() -> Result<HashMap<String, CodexModelProvider>, String> {
    let manager = CodexConfigManager::new()?;
    manager.get_model_providers()
}

/// 添加 Codex 模型提供商
#[tauri::command]
pub async fn add_codex_provider(name: String, provider: CodexModelProvider) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.add_model_provider(&name, provider)
}

/// 删除 Codex 模型提供商
#[tauri::command]
pub async fn remove_codex_provider(name: String) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.remove_model_provider(&name)
}

/// 应用 Provider 到 Codex
#[tauri::command]
pub async fn apply_provider_to_codex(provider: CodexProvider) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.apply_provider(&provider)
}

/// 获取 Codex MCP 服务器列表
#[tauri::command]
pub async fn get_codex_mcp_servers() -> Result<HashMap<String, CodexMcpServer>, String> {
    let manager = CodexConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Codex MCP 服务器
#[tauri::command]
pub async fn add_codex_mcp_server(name: String, server: CodexMcpServer) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Codex MCP 服务器
#[tauri::command]
pub async fn remove_codex_mcp_server(name: String) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Codex
#[tauri::command]
pub async fn sync_mcp_to_codex(servers: HashMap<String, CodexMcpServer>) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 获取 AGENTS.md 内容
#[tauri::command]
pub async fn get_agents_md() -> Result<Option<String>, String> {
    let manager = CodexConfigManager::new()?;
    manager.read_agents_md()
}

/// 保存 AGENTS.md 内容
#[tauri::command]
pub async fn save_agents_md(content: String) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.write_agents_md(&content)
}

/// 设置 Codex API Key 并跳过 OAuth 登录
/// 直接配置 API Key 和 model_provider，避免 OAuth 登录流程
#[tauri::command]
pub async fn set_codex_api_key_skip_oauth(
    api_key: String,
    base_url: String,
    provider_name: String,
) -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.set_api_key_skip_oauth(&api_key, &base_url, &provider_name)
}

/// 清除 Codex API Key 配置（恢复 OAuth 登录）
#[tauri::command]
pub async fn clear_codex_api_key() -> Result<(), String> {
    let manager = CodexConfigManager::new()?;
    manager.clear_api_key()
}

/// 获取 Codex 当前 API Key
#[tauri::command]
pub async fn get_codex_api_key() -> Result<Option<String>, String> {
    let manager = CodexConfigManager::new()?;
    manager.get_api_key()
}
