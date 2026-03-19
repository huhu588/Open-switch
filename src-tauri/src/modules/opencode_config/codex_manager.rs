// Codex 配置管理器
// 管理 OpenAI Codex CLI 的配置文件：
// - ~/.codex/auth.json (认证信息)
// - ~/.codex/config.toml (配置文件)
// - ~/.codex/AGENTS.md (系统提示)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Codex 认证信息 (auth.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// 其他字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Codex 自定义模型提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexModelProvider {
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_openai_auth: Option<bool>,
}

/// Codex MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexMcpServer {
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

/// Codex 配置结构 (从 config.toml 解析)
#[derive(Debug, Clone, Default)]
pub struct CodexConfig {
    pub cli_auth_credentials_store: Option<String>,
    pub model_providers: HashMap<String, CodexModelProvider>,
    pub mcp_servers: HashMap<String, CodexMcpServer>,
    pub forced_login_method: Option<String>,
    pub forced_chatgpt_workspace_id: Option<String>,
    /// 是否跳过 OAuth 登录，直接使用 API Key
    pub skip_oauth_login: Option<bool>,
}

/// Codex Provider 信息（用于 Ai Switch 管理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexProvider {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub struct CodexConfigManager {
    // Codex 配置目录
    codex_dir: PathBuf,
    // auth.json 路径
    auth_json: PathBuf,
    // config.toml 路径
    config_toml: PathBuf,
    // AGENTS.md 系统提示路径
    agents_md: PathBuf,
}

impl CodexConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;
        
        let codex_dir = user_home.join(".codex");
        let auth_json = codex_dir.join("auth.json");
        let config_toml = codex_dir.join("config.toml");
        let agents_md = codex_dir.join("AGENTS.md");
        
        Ok(Self {
            codex_dir,
            auth_json,
            config_toml,
            agents_md,
        })
    }

    /// 确保 Codex 目录存在
    fn ensure_codex_dir(&self) -> Result<(), String> {
        if !self.codex_dir.exists() {
            fs::create_dir_all(&self.codex_dir)
                .map_err(|e| format!("创建 Codex 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 读取 auth.json
    pub fn read_auth(&self) -> Result<CodexAuth, String> {
        if !self.auth_json.exists() {
            return Ok(CodexAuth::default());
        }
        
        let content = fs::read_to_string(&self.auth_json)
            .map_err(|e| format!("读取 auth.json 失败: {}", e))?;
        
        let content = content.trim_start_matches('\u{feff}');
        
        serde_json::from_str(content)
            .map_err(|e| format!("解析 auth.json 失败: {}", e))
    }

    /// 写入 auth.json
    pub fn write_auth(&self, auth: &CodexAuth) -> Result<(), String> {
        self.ensure_codex_dir()?;
        
        let content = serde_json::to_string_pretty(auth)
            .map_err(|e| format!("序列化 auth.json 失败: {}", e))?;
        
        fs::write(&self.auth_json, content)
            .map_err(|e| format!("写入 auth.json 失败: {}", e))
    }

    /// 读取 config.toml
    pub fn read_config(&self) -> Result<CodexConfig, String> {
        if !self.config_toml.exists() {
            return Ok(CodexConfig::default());
        }
        
        let content = fs::read_to_string(&self.config_toml)
            .map_err(|e| format!("读取 config.toml 失败: {}", e))?;
        
        self.parse_toml(&content)
    }

    /// 解析 TOML 配置
    fn parse_toml(&self, content: &str) -> Result<CodexConfig, String> {
        let mut config = CodexConfig::default();
        
        // 简单的 TOML 解析（不使用完整的 toml 库，因为它可能不在依赖中）
        let mut current_section = String::new();
        let mut current_provider_name = String::new();
        let mut current_mcp_name = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // 检测 section
            if line.starts_with('[') && line.ends_with(']') {
                let section = &line[1..line.len()-1];
                
                if section.starts_with("model_providers.") {
                    current_section = "model_providers".to_string();
                    current_provider_name = section["model_providers.".len()..].to_string();
                } else if section.starts_with("mcp_servers.") {
                    current_section = "mcp_servers".to_string();
                    current_mcp_name = section["mcp_servers.".len()..].to_string();
                } else {
                    current_section = section.to_string();
                }
                continue;
            }
            
            // 解析键值对
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos+1..].trim();
                let value = value.trim_matches('"');
                
                match current_section.as_str() {
                    "" => {
                        if key == "cli_auth_credentials_store" {
                            config.cli_auth_credentials_store = Some(value.to_string());
                        } else if key == "forced_login_method" {
                            config.forced_login_method = Some(value.to_string());
                        } else if key == "forced_chatgpt_workspace_id" {
                            config.forced_chatgpt_workspace_id = Some(value.to_string());
                        }
                    }
                    "model_providers" => {
                        if !current_provider_name.is_empty() {
                            let provider = config.model_providers
                                .entry(current_provider_name.clone())
                                .or_insert_with(|| CodexModelProvider {
                                    name: current_provider_name.clone(),
                                    base_url: String::new(),
                                    env_key: None,
                                    requires_openai_auth: None,
                                });
                            
                            match key {
                                "name" => provider.name = value.to_string(),
                                "base_url" => provider.base_url = value.to_string(),
                                "env_key" => provider.env_key = Some(value.to_string()),
                                "requires_openai_auth" => {
                                    provider.requires_openai_auth = Some(value == "true");
                                }
                                _ => {}
                            }
                        }
                    }
                    "mcp_servers" => {
                        if !current_mcp_name.is_empty() {
                            let server = config.mcp_servers
                                .entry(current_mcp_name.clone())
                                .or_insert_with(|| CodexMcpServer {
                                    command: Vec::new(),
                                    env: HashMap::new(),
                                });
                            
                            if key == "command" {
                                // 解析数组 ["cmd", "arg1", "arg2"]
                                if value.starts_with('[') && value.ends_with(']') {
                                    let inner = &value[1..value.len()-1];
                                    server.command = inner
                                        .split(',')
                                        .map(|s| s.trim().trim_matches('"').to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(config)
    }

    /// 写入 config.toml
    pub fn write_config(&self, config: &CodexConfig) -> Result<(), String> {
        self.ensure_codex_dir()?;
        
        let mut content = String::new();
        
        // 写入顶级配置
        if let Some(ref store) = config.cli_auth_credentials_store {
            content.push_str(&format!("cli_auth_credentials_store = \"{}\"\n", store));
        }
        if let Some(ref method) = config.forced_login_method {
            content.push_str(&format!("forced_login_method = \"{}\"\n", method));
        }
        if let Some(ref workspace_id) = config.forced_chatgpt_workspace_id {
            content.push_str(&format!("forced_chatgpt_workspace_id = \"{}\"\n", workspace_id));
        }
        
        // 写入 model_providers
        for (name, provider) in &config.model_providers {
            content.push_str(&format!("\n[model_providers.{}]\n", name));
            content.push_str(&format!("name = \"{}\"\n", provider.name));
            content.push_str(&format!("base_url = \"{}\"\n", provider.base_url));
            if let Some(ref env_key) = provider.env_key {
                content.push_str(&format!("env_key = \"{}\"\n", env_key));
            }
            if let Some(requires_auth) = provider.requires_openai_auth {
                content.push_str(&format!("requires_openai_auth = {}\n", requires_auth));
            }
        }
        
        // 写入 mcp_servers
        for (name, server) in &config.mcp_servers {
            content.push_str(&format!("\n[mcp_servers.{}]\n", name));
            let cmd_str: Vec<String> = server.command.iter()
                .map(|s| format!("\"{}\"", s))
                .collect();
            content.push_str(&format!("command = [{}]\n", cmd_str.join(", ")));
        }
        
        fs::write(&self.config_toml, content)
            .map_err(|e| format!("写入 config.toml 失败: {}", e))
    }

    /// 添加自定义模型提供商
    pub fn add_model_provider(&self, name: &str, provider: CodexModelProvider) -> Result<(), String> {
        let mut config = self.read_config()?;
        config.model_providers.insert(name.to_string(), provider);
        self.write_config(&config)
    }

    /// 删除模型提供商
    pub fn remove_model_provider(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;
        config.model_providers.remove(name);
        self.write_config(&config)
    }

    /// 获取所有模型提供商
    pub fn get_model_providers(&self) -> Result<HashMap<String, CodexModelProvider>, String> {
        let config = self.read_config()?;
        Ok(config.model_providers)
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: CodexMcpServer) -> Result<(), String> {
        let mut config = self.read_config()?;
        config.mcp_servers.insert(name.to_string(), server);
        self.write_config(&config)
    }

    /// 删除 MCP 服务器
    pub fn remove_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;
        config.mcp_servers.remove(name);
        self.write_config(&config)
    }

    /// 获取所有 MCP 服务器
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, CodexMcpServer>, String> {
        let config = self.read_config()?;
        Ok(config.mcp_servers)
    }

    /// 同步 MCP 服务器配置
    pub fn sync_mcp_servers(&self, servers: HashMap<String, CodexMcpServer>) -> Result<(), String> {
        let mut config = self.read_config()?;
        config.mcp_servers = servers;
        self.write_config(&config)
    }

    /// 读取 AGENTS.md 系统提示
    pub fn read_agents_md(&self) -> Result<Option<String>, String> {
        if !self.agents_md.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&self.agents_md)
            .map_err(|e| format!("读取 AGENTS.md 失败: {}", e))?;
        
        Ok(Some(content))
    }

    /// 写入 AGENTS.md 系统提示
    pub fn write_agents_md(&self, content: &str) -> Result<(), String> {
        self.ensure_codex_dir()?;
        
        fs::write(&self.agents_md, content)
            .map_err(|e| format!("写入 AGENTS.md 失败: {}", e))
    }

    /// 应用 Provider 配置到 Codex
    pub fn apply_provider(&self, provider: &CodexProvider) -> Result<(), String> {
        let mut config = self.read_config()?;
        
        // 创建模型提供商配置
        let codex_provider = CodexModelProvider {
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            env_key: provider.env_key.clone(),
            requires_openai_auth: None,
        };
        
        config.model_providers.insert(provider.name.clone(), codex_provider);
        self.write_config(&config)
    }

    /// 获取配置目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.codex_dir
    }

    /// 检查 Codex 是否已配置
    pub fn is_configured(&self) -> bool {
        self.auth_json.exists() || self.config_toml.exists()
    }

    /// 设置 API Key 并跳过 OAuth 登录
    /// 这会在 auth.json 中写入 API Key 并配置 model_provider
    pub fn set_api_key_skip_oauth(
        &self,
        api_key: &str,
        base_url: &str,
        provider_name: &str,
    ) -> Result<(), String> {
        self.ensure_codex_dir()?;
        
        // 写入 auth.json
        let mut auth = self.read_auth()?;
        auth.other.insert(
            "apiKey".to_string(),
            serde_json::Value::String(api_key.to_string()),
        );
        self.write_auth(&auth)?;
        
        // 配置 model_provider
        let provider = CodexModelProvider {
            name: provider_name.to_string(),
            base_url: base_url.to_string(),
            env_key: Some("OPENAI_API_KEY".to_string()),
            requires_openai_auth: Some(false),
        };
        
        let mut config = self.read_config()?;
        config.model_providers.insert(provider_name.to_string(), provider);
        // 设置凭据存储方式为 file（避免 keyring 的问题）
        config.cli_auth_credentials_store = Some("file".to_string());
        self.write_config(&config)?;
        
        Ok(())
    }

    /// 清除 API Key 配置（恢复 OAuth 登录）
    pub fn clear_api_key(&self) -> Result<(), String> {
        let mut auth = self.read_auth()?;
        auth.other.remove("apiKey");
        self.write_auth(&auth)
    }

    /// 获取当前 API Key
    pub fn get_api_key(&self) -> Result<Option<String>, String> {
        let auth = self.read_auth()?;
        Ok(auth
            .other
            .get("apiKey")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_auth_serialization() {
        let auth = CodexAuth {
            access_token: Some("test-token".to_string()),
            refresh_token: Some("test-refresh".to_string()),
            other: HashMap::new(),
        };
        
        let json = serde_json::to_string_pretty(&auth).unwrap();
        assert!(json.contains("access_token"));
    }

    #[test]
    fn test_parse_toml() {
        let manager = CodexConfigManager::new().unwrap();
        
        let toml_content = r#"
cli_auth_credentials_store = "keyring"

[model_providers.custom]
name = "Custom Provider"
base_url = "https://api.example.com/v1"
env_key = "CUSTOM_API_KEY"

[mcp_servers.memory]
command = ["npx", "-y", "@modelcontextprotocol/server-memory"]
"#;
        
        let config = manager.parse_toml(toml_content).unwrap();
        assert_eq!(config.cli_auth_credentials_store, Some("keyring".to_string()));
        assert!(config.model_providers.contains_key("custom"));
        assert!(config.mcp_servers.contains_key("memory"));
    }
}
