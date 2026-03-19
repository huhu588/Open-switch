// Claude Code 相关 Tauri 命令

use crate::modules::opencode_config::claude_code_manager::{
    ClaudeCodeConfigManager, ClaudeCodeProvider, ClaudeCodeSettings, ClaudeMcpServer,
};
use serde::Serialize;
use std::collections::HashMap;

/// Claude Code 配置状态
#[derive(Debug, Serialize)]
pub struct ClaudeCodeStatus {
    pub is_configured: bool,
    pub has_api_key: bool,
    pub api_key_masked: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub mcp_server_count: usize,
}

/// 获取 Claude Code 配置状态
#[tauri::command]
pub async fn get_claude_code_status() -> Result<ClaudeCodeStatus, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    
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
    let mcp_servers = manager.get_mcp_servers()?;
    
    Ok(ClaudeCodeStatus {
        is_configured: manager.is_configured(),
        has_api_key,
        api_key_masked,
        base_url,
        model,
        mcp_server_count: mcp_servers.len(),
    })
}

/// 获取 Claude Code 设置
#[tauri::command]
pub async fn get_claude_code_settings() -> Result<ClaudeCodeSettings, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.read_settings()
}

/// 保存 Claude Code 设置
#[tauri::command]
pub async fn save_claude_code_settings(settings: ClaudeCodeSettings) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.write_settings(&settings)
}

/// 设置 Claude Code API Key
#[tauri::command]
pub async fn set_claude_code_api_key(api_key: String) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.set_api_key(&api_key)
}

/// 设置 Claude Code Base URL
#[tauri::command]
pub async fn set_claude_code_base_url(base_url: Option<String>) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    
    if let Some(url) = base_url {
        manager.set_base_url(&url)
    } else {
        // 清除自定义 base_url
        let mut settings = manager.read_settings()?;
        settings.env.remove("ANTHROPIC_BASE_URL");
        manager.write_settings(&settings)
    }
}

/// 设置 Claude Code 模型
#[tauri::command]
pub async fn set_claude_code_model(model: Option<String>) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    
    let mut settings = manager.read_settings()?;
    settings.model = model;
    manager.write_settings(&settings)
}

/// 应用 Provider 到 Claude Code
#[tauri::command]
pub async fn apply_provider_to_claude_code(provider: ClaudeCodeProvider) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.apply_provider(&provider)
}

/// 获取 Claude Code MCP 服务器列表
#[tauri::command]
pub async fn get_claude_code_mcp_servers() -> Result<HashMap<String, ClaudeMcpServer>, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.get_mcp_servers()
}

/// 添加 Claude Code MCP 服务器
#[tauri::command]
pub async fn add_claude_code_mcp_server(name: String, server: ClaudeMcpServer) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.add_mcp_server(&name, server)
}

/// 删除 Claude Code MCP 服务器
#[tauri::command]
pub async fn remove_claude_code_mcp_server(name: String) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.remove_mcp_server(&name)
}

/// 同步 MCP 服务器到 Claude Code
#[tauri::command]
pub async fn sync_mcp_to_claude_code(servers: HashMap<String, ClaudeMcpServer>) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.sync_mcp_servers(servers)
}

/// 获取 CLAUDE.md 内容
#[tauri::command]
pub async fn get_claude_md() -> Result<Option<String>, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.read_claude_md()
}

/// 保存 CLAUDE.md 内容
#[tauri::command]
pub async fn save_claude_md(content: String) -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.write_claude_md(&content)
}

/// 清除 Claude Code Provider 配置（API Key、Base URL）
#[tauri::command]
pub async fn clear_claude_code_config() -> Result<(), String> {
    let manager = ClaudeCodeConfigManager::new()?;
    let mut settings = manager.read_settings()?;
    
    // 清除 API Key 和 Base URL
    settings.env.remove("ANTHROPIC_API_KEY");
    settings.env.remove("ANTHROPIC_BASE_URL");
    settings.model = None;
    
    manager.write_settings(&settings)
}

/// 设置 Claude Code 跳过首次登录确认
/// 写入 ~/.claude.json 中的 hasCompletedOnboarding: true
#[tauri::command]
pub async fn set_claude_code_skip_onboarding() -> Result<bool, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.set_has_completed_onboarding()
}

/// 清除 Claude Code 跳过首次登录确认
/// 删除 ~/.claude.json 中的 hasCompletedOnboarding 字段
#[tauri::command]
pub async fn clear_claude_code_skip_onboarding() -> Result<bool, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.clear_has_completed_onboarding()
}

/// 获取 Claude Code 跳过首次登录确认状态
#[tauri::command]
pub async fn get_claude_code_skip_onboarding() -> Result<bool, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    manager.get_has_completed_onboarding()
}
