// Gemini CLI 相关 Tauri 命令

use crate::modules::opencode_config::gemini_manager::{
    GeminiConfigManager, GeminiMcpServer, GeminiProvider, GeminiSettings,
};
use serde::Serialize;
use std::collections::HashMap;

/// Gemini 配置状态
#[derive(Debug, Serialize)]
pub struct GeminiStatus {
    pub is_configured: bool,
    pub has_api_key: bool,
    pub api_key_masked: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub auth_mode: Option<String>,
    pub mcp_server_count: usize,
}

/// 获取 Gemini 配置状态
#[tauri::command]
pub async fn get_gemini_status() -> Result<GeminiStatus, String> {
    let manager = GeminiConfigManager::new()?;
    
    let api_key = manager.get_api_key()?;
    let has_api_key = api_key.is_some();
    let api_key_masked = api_key.map(|k| {
        if k.len() > 8 {
            format!("{}...{}", &k[..4], &k[k.len()-4..])
        } else {
            "****".to_string()
        }
    });
    
    let base_url = manager.get_base_url()?;
    let model = manager.get_model()?;
    let auth_mode = manager.get_auth_mode()?;
    let mcp_servers = manager.get_mcp_servers()?;
    
    Ok(GeminiStatus {
        is_configured: manager.is_configured(),
        has_api_key,
        api_key_masked,
        base_url,
        model,
        auth_mode,
        mcp_server_count: mcp_servers.len(),
    })
}

/// 获取 Gemini 设置
#[tauri::command]
pub async fn get_gemini_settings() -> Result<GeminiSettings, String> {
    let manager = GeminiConfigManager::new()?;
    manager.read_settings()
}

/// 保存 Gemini 设置
#[tauri::command]
pub async fn save_gemini_settings(settings: GeminiSettings) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.write_settings(&settings)
}

/// 设置 Gemini API Key
#[tauri::command]
pub async fn set_gemini_api_key(api_key: String) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_api_key(&api_key)
}

/// 设置 Gemini Base URL
#[tauri::command]
pub async fn set_gemini_base_url(base_url: Option<String>) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_base_url(base_url)
}

/// 设置 Gemini 模型
#[tauri::command]
pub async fn set_gemini_model(model: Option<String>) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_model(model)
}

/// 设置 Gemini 认证模式
#[tauri::command]
pub async fn set_gemini_auth_mode(auth_mode: String) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_auth_mode(&auth_mode)
}

/// 应用 Provider 到 Gemini
#[tauri::command]
pub async fn apply_provider_to_gemini(provider: GeminiProvider) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.apply_provider(&provider)
}

/// 获取 Gemini MCP 服务器列表
#[tauri::command]
pub async fn get_gemini_mcp_servers() -> Result<HashMap<String, GeminiMcpServer>, String> {
    let manager = GeminiConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Gemini MCP 服务器
#[tauri::command]
pub async fn add_gemini_mcp_server(name: String, server: GeminiMcpServer) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Gemini MCP 服务器
#[tauri::command]
pub async fn remove_gemini_mcp_server(name: String) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Gemini
#[tauri::command]
pub async fn sync_mcp_to_gemini(servers: HashMap<String, GeminiMcpServer>) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 获取 GEMINI.md 内容
#[tauri::command]
pub async fn get_gemini_md() -> Result<Option<String>, String> {
    let manager = GeminiConfigManager::new()?;
    manager.read_gemini_md()
}

/// 保存 GEMINI.md 内容
#[tauri::command]
pub async fn save_gemini_md(content: String) -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.write_gemini_md(&content)
}

/// 清除 Gemini Provider 配置（API Key、Base URL）
#[tauri::command]
pub async fn clear_gemini_config() -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.clear_provider_config()
}

/// 设置 Gemini 认证模式为 API Key（跳过 OAuth 登录）
/// 写入 settings.json 中的 security.auth.selectedType: "gemini-api-key"
#[tauri::command]
pub async fn set_gemini_api_key_auth_mode() -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_api_key_auth_mode()
}

/// 设置 Gemini 认证模式为 OAuth（Google 官方）
/// 写入 settings.json 中的 security.auth.selectedType: "oauth-personal"
#[tauri::command]
pub async fn set_gemini_oauth_auth_mode() -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.set_oauth_auth_mode()
}

/// 获取 Gemini 当前认证类型
#[tauri::command]
pub async fn get_gemini_auth_selected_type() -> Result<Option<String>, String> {
    let manager = GeminiConfigManager::new()?;
    manager.get_auth_selected_type()
}

/// 清除 Gemini 认证类型设置
#[tauri::command]
pub async fn clear_gemini_auth_selected_type() -> Result<(), String> {
    let manager = GeminiConfigManager::new()?;
    manager.clear_auth_selected_type()
}
