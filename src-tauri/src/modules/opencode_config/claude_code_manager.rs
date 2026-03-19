// Claude Code 配置管理器
// 管理 Claude Code CLI 的配置文件：
// - ~/.claude/settings.json (用户设置)
// - ~/.claude.json (MCP 配置)
// - ~/.claude/CLAUDE.md (系统提示)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Claude Code 权限配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudePermissions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ask: Vec<String>,
}

/// Claude Code settings.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeCodeSettings {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<ClaudePermissions>,
    #[serde(rename = "apiKeyHelper", skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,
    #[serde(rename = "cleanupPeriodDays", skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Claude Code MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

/// Claude Code ~/.claude.json 配置结构 (MCP 配置)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeJsonConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, ClaudeMcpServer>,
    /// 是否已完成首次启动引导（跳过登录确认）
    #[serde(rename = "hasCompletedOnboarding", default, skip_serializing_if = "Option::is_none")]
    pub has_completed_onboarding: Option<bool>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Claude Code Provider 信息（用于 Ai Switch 管理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeProvider {
    pub name: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub struct ClaudeCodeConfigManager {
    // Claude Code 用户设置目录
    claude_dir: PathBuf,
    // settings.json 路径
    settings_json: PathBuf,
    // ~/.claude.json MCP 配置路径
    claude_json: PathBuf,
    // CLAUDE.md 系统提示路径
    claude_md: PathBuf,
}

impl ClaudeCodeConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;
        
        let claude_dir = user_home.join(".claude");
        let settings_json = claude_dir.join("settings.json");
        let claude_json = user_home.join(".claude.json");
        let claude_md = claude_dir.join("CLAUDE.md");
        
        Ok(Self {
            claude_dir,
            settings_json,
            claude_json,
            claude_md,
        })
    }

    /// 确保 Claude 目录存在
    fn ensure_claude_dir(&self) -> Result<(), String> {
        if !self.claude_dir.exists() {
            fs::create_dir_all(&self.claude_dir)
                .map_err(|e| format!("创建 Claude 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 读取 settings.json
    pub fn read_settings(&self) -> Result<ClaudeCodeSettings, String> {
        if !self.settings_json.exists() {
            return Ok(ClaudeCodeSettings::default());
        }
        
        let content = fs::read_to_string(&self.settings_json)
            .map_err(|e| format!("读取 settings.json 失败: {}", e))?;
        
        // 移除 UTF-8 BOM
        let content = content.trim_start_matches('\u{feff}');
        
        serde_json::from_str(content)
            .map_err(|e| format!("解析 settings.json 失败: {}", e))
    }

    /// 写入 settings.json
    pub fn write_settings(&self, settings: &ClaudeCodeSettings) -> Result<(), String> {
        self.ensure_claude_dir()?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("序列化 settings.json 失败: {}", e))?;
        
        fs::write(&self.settings_json, content)
            .map_err(|e| format!("写入 settings.json 失败: {}", e))
    }

    /// 读取 ~/.claude.json (MCP 配置)
    pub fn read_claude_json(&self) -> Result<ClaudeJsonConfig, String> {
        if !self.claude_json.exists() {
            return Ok(ClaudeJsonConfig::default());
        }
        
        let content = fs::read_to_string(&self.claude_json)
            .map_err(|e| format!("读取 .claude.json 失败: {}", e))?;
        
        let content = content.trim_start_matches('\u{feff}');
        
        serde_json::from_str(content)
            .map_err(|e| format!("解析 .claude.json 失败: {}", e))
    }

    /// 写入 ~/.claude.json (MCP 配置)
    pub fn write_claude_json(&self, config: &ClaudeJsonConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 .claude.json 失败: {}", e))?;
        
        fs::write(&self.claude_json, content)
            .map_err(|e| format!("写入 .claude.json 失败: {}", e))
    }

    /// 获取当前配置的 API Key
    pub fn get_api_key(&self) -> Result<Option<String>, String> {
        let settings = self.read_settings()?;
        
        // 优先检查 ANTHROPIC_API_KEY
        if let Some(key) = settings.env.get("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                return Ok(Some(key.clone()));
            }
        }
        
        // 然后检查 ANTHROPIC_AUTH_TOKEN
        if let Some(key) = settings.env.get("ANTHROPIC_AUTH_TOKEN") {
            if !key.is_empty() {
                return Ok(Some(key.clone()));
            }
        }
        
        Ok(None)
    }

    /// 设置 API Key
    pub fn set_api_key(&self, api_key: &str) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
        self.write_settings(&settings)
    }

    /// 设置 Base URL (通过环境变量)
    pub fn set_base_url(&self, base_url: &str) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        // Claude Code 使用 ANTHROPIC_BASE_URL 环境变量
        settings.env.insert("ANTHROPIC_BASE_URL".to_string(), base_url.to_string());
        self.write_settings(&settings)
    }

    /// 获取 Base URL
    pub fn get_base_url(&self) -> Result<Option<String>, String> {
        let settings = self.read_settings()?;
        Ok(settings.env.get("ANTHROPIC_BASE_URL").cloned())
    }

    /// 设置模型
    pub fn set_model(&self, model: &str) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.model = Some(model.to_string());
        self.write_settings(&settings)
    }

    /// 获取当前模型
    pub fn get_model(&self) -> Result<Option<String>, String> {
        let settings = self.read_settings()?;
        Ok(settings.model)
    }

    /// 应用 Provider 配置到 Claude Code
    pub fn apply_provider(&self, provider: &ClaudeCodeProvider) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        
        // 设置 API Key
        settings.env.insert("ANTHROPIC_API_KEY".to_string(), provider.api_key.clone());
        
        // 设置 Base URL（如果有）
        if let Some(ref base_url) = provider.base_url {
            settings.env.insert("ANTHROPIC_BASE_URL".to_string(), base_url.clone());
        } else {
            // 移除自定义 base_url，使用官方
            settings.env.remove("ANTHROPIC_BASE_URL");
        }
        
        // 设置模型（如果有）
        if let Some(ref model) = provider.model {
            settings.model = Some(model.clone());
        }
        
        self.write_settings(&settings)
    }

    /// 读取 CLAUDE.md 系统提示
    pub fn read_claude_md(&self) -> Result<Option<String>, String> {
        if !self.claude_md.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&self.claude_md)
            .map_err(|e| format!("读取 CLAUDE.md 失败: {}", e))?;
        
        Ok(Some(content))
    }

    /// 写入 CLAUDE.md 系统提示
    pub fn write_claude_md(&self, content: &str) -> Result<(), String> {
        self.ensure_claude_dir()?;
        
        fs::write(&self.claude_md, content)
            .map_err(|e| format!("写入 CLAUDE.md 失败: {}", e))
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, ClaudeMcpServer>, String> {
        let config = self.read_claude_json()?;
        Ok(config.mcp_servers)
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: ClaudeMcpServer) -> Result<(), String> {
        let mut config = self.read_claude_json()?;
        config.mcp_servers.insert(name.to_string(), server);
        self.write_claude_json(&config)
    }

    /// 删除 MCP 服务器
    pub fn remove_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_claude_json()?;
        config.mcp_servers.remove(name);
        self.write_claude_json(&config)
    }

    /// 同步 MCP 服务器配置（从 Ai Switch 格式转换）
    pub fn sync_mcp_servers(&self, servers: HashMap<String, ClaudeMcpServer>) -> Result<(), String> {
        let mut config = self.read_claude_json()?;
        config.mcp_servers = servers;
        self.write_claude_json(&config)
    }

    /// 获取配置目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.claude_dir
    }

    /// 检查 Claude Code 是否已配置
    pub fn is_configured(&self) -> bool {
        self.settings_json.exists() || self.claude_json.exists()
    }

    /// 设置 hasCompletedOnboarding 为 true（跳过首次登录确认）
    /// 这会在 ~/.claude.json 中写入 hasCompletedOnboarding: true
    pub fn set_has_completed_onboarding(&self) -> Result<bool, String> {
        let mut config = self.read_claude_json()?;
        
        // 如果已经是 true，则无需再次写入
        if config.has_completed_onboarding == Some(true) {
            return Ok(false);
        }
        
        config.has_completed_onboarding = Some(true);
        self.write_claude_json(&config)?;
        Ok(true)
    }

    /// 清除 hasCompletedOnboarding（恢复首次登录确认）
    pub fn clear_has_completed_onboarding(&self) -> Result<bool, String> {
        let mut config = self.read_claude_json()?;
        
        // 如果本来就没有该字段，则无需删除
        if config.has_completed_onboarding.is_none() {
            return Ok(false);
        }
        
        config.has_completed_onboarding = None;
        self.write_claude_json(&config)?;
        Ok(true)
    }

    /// 获取 hasCompletedOnboarding 状态
    pub fn get_has_completed_onboarding(&self) -> Result<bool, String> {
        let config = self.read_claude_json()?;
        Ok(config.has_completed_onboarding.unwrap_or(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_settings_serialization() {
        let mut settings = ClaudeCodeSettings::default();
        settings.env.insert("ANTHROPIC_API_KEY".to_string(), "test-key".to_string());
        settings.model = Some("claude-sonnet-4".to_string());
        
        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("ANTHROPIC_API_KEY"));
        assert!(json.contains("claude-sonnet-4"));
    }

    #[test]
    fn test_claude_mcp_server_serialization() {
        let server = ClaudeMcpServer {
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            env: HashMap::new(),
            url: None,
            headers: HashMap::new(),
        };
        
        let json = serde_json::to_string_pretty(&server).unwrap();
        assert!(json.contains("npx"));
        assert!(json.contains("server-memory"));
    }
}
