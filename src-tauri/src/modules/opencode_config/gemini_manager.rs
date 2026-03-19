// Gemini CLI 配置管理器
// 管理 Google Gemini CLI 的配置文件：
// - ~/.gemini/.env (环境变量/API Key)
// - ~/.gemini/settings.json (设置)
// - ~/.gemini/GEMINI.md (系统提示)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Gemini MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Gemini security.auth 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiSecurityAuth {
    #[serde(rename = "selectedType", skip_serializing_if = "Option::is_none")]
    pub selected_type: Option<String>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Gemini security 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiSecurity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<GeminiSecurityAuth>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Gemini settings.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiSettings {
    #[serde(rename = "authMode", skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, GeminiMcpServer>,
    /// security 配置（包含 auth.selectedType）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<GeminiSecurity>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Gemini .env 环境变量配置
#[derive(Debug, Clone, Default)]
pub struct GeminiEnv {
    pub gemini_api_key: Option<String>,
    pub google_gemini_api_key: Option<String>,
    pub google_gemini_base_url: Option<String>,
    pub gemini_model: Option<String>,
    pub other: HashMap<String, String>,
}

/// Gemini Provider 信息（用于 Ai Switch 管理）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiProvider {
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

pub struct GeminiConfigManager {
    // Gemini 配置目录
    gemini_dir: PathBuf,
    // .env 路径
    env_file: PathBuf,
    // settings.json 路径
    settings_json: PathBuf,
    // GEMINI.md 系统提示路径
    gemini_md: PathBuf,
}

impl GeminiConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;
        
        let gemini_dir = user_home.join(".gemini");
        let env_file = gemini_dir.join(".env");
        let settings_json = gemini_dir.join("settings.json");
        let gemini_md = gemini_dir.join("GEMINI.md");
        
        Ok(Self {
            gemini_dir,
            env_file,
            settings_json,
            gemini_md,
        })
    }

    /// 确保 Gemini 目录存在
    fn ensure_gemini_dir(&self) -> Result<(), String> {
        if !self.gemini_dir.exists() {
            fs::create_dir_all(&self.gemini_dir)
                .map_err(|e| format!("创建 Gemini 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 读取 .env 文件
    pub fn read_env(&self) -> Result<GeminiEnv, String> {
        if !self.env_file.exists() {
            return Ok(GeminiEnv::default());
        }
        
        let content = fs::read_to_string(&self.env_file)
            .map_err(|e| format!("读取 .env 失败: {}", e))?;
        
        self.parse_env(&content)
    }

    /// 解析 .env 文件内容
    fn parse_env(&self, content: &str) -> Result<GeminiEnv, String> {
        let mut env = GeminiEnv::default();
        
        for line in content.lines() {
            let line = line.trim();
            
            // 跳过空行和注释
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // 解析 KEY=VALUE
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos+1..].trim();
                // 移除可能的引号
                let value = value.trim_matches('"').trim_matches('\'');
                
                match key {
                    "GEMINI_API_KEY" => env.gemini_api_key = Some(value.to_string()),
                    "GOOGLE_GEMINI_API_KEY" => env.google_gemini_api_key = Some(value.to_string()),
                    "GOOGLE_GEMINI_BASE_URL" => env.google_gemini_base_url = Some(value.to_string()),
                    "GEMINI_MODEL" => env.gemini_model = Some(value.to_string()),
                    _ => {
                        env.other.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
        
        Ok(env)
    }

    /// 写入 .env 文件
    pub fn write_env(&self, env: &GeminiEnv) -> Result<(), String> {
        self.ensure_gemini_dir()?;
        
        let mut content = String::new();
        
        if let Some(ref key) = env.gemini_api_key {
            content.push_str(&format!("GEMINI_API_KEY={}\n", key));
        }
        if let Some(ref key) = env.google_gemini_api_key {
            content.push_str(&format!("GOOGLE_GEMINI_API_KEY={}\n", key));
        }
        if let Some(ref url) = env.google_gemini_base_url {
            content.push_str(&format!("GOOGLE_GEMINI_BASE_URL={}\n", url));
        }
        if let Some(ref model) = env.gemini_model {
            content.push_str(&format!("GEMINI_MODEL={}\n", model));
        }
        
        // 写入其他环境变量
        for (key, value) in &env.other {
            content.push_str(&format!("{}={}\n", key, value));
        }
        
        fs::write(&self.env_file, content)
            .map_err(|e| format!("写入 .env 失败: {}", e))
    }

    /// 读取 settings.json
    pub fn read_settings(&self) -> Result<GeminiSettings, String> {
        if !self.settings_json.exists() {
            return Ok(GeminiSettings::default());
        }
        
        let content = fs::read_to_string(&self.settings_json)
            .map_err(|e| format!("读取 settings.json 失败: {}", e))?;
        
        let content = content.trim_start_matches('\u{feff}');
        
        serde_json::from_str(content)
            .map_err(|e| format!("解析 settings.json 失败: {}", e))
    }

    /// 写入 settings.json
    pub fn write_settings(&self, settings: &GeminiSettings) -> Result<(), String> {
        self.ensure_gemini_dir()?;
        
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("序列化 settings.json 失败: {}", e))?;
        
        fs::write(&self.settings_json, content)
            .map_err(|e| format!("写入 settings.json 失败: {}", e))
    }

    /// 获取 API Key
    pub fn get_api_key(&self) -> Result<Option<String>, String> {
        let env = self.read_env()?;
        
        // 优先使用 GEMINI_API_KEY
        if let Some(ref key) = env.gemini_api_key {
            if !key.is_empty() {
                return Ok(Some(key.clone()));
            }
        }
        
        // 然后尝试 GOOGLE_GEMINI_API_KEY
        if let Some(ref key) = env.google_gemini_api_key {
            if !key.is_empty() {
                return Ok(Some(key.clone()));
            }
        }
        
        Ok(None)
    }

    /// 设置 API Key
    pub fn set_api_key(&self, api_key: &str) -> Result<(), String> {
        let mut env = self.read_env()?;
        env.gemini_api_key = Some(api_key.to_string());
        self.write_env(&env)
    }

    /// 获取 Base URL
    pub fn get_base_url(&self) -> Result<Option<String>, String> {
        let env = self.read_env()?;
        Ok(env.google_gemini_base_url)
    }

    /// 设置 Base URL
    pub fn set_base_url(&self, base_url: Option<String>) -> Result<(), String> {
        let mut env = self.read_env()?;
        env.google_gemini_base_url = base_url;
        self.write_env(&env)
    }

    /// 获取模型
    pub fn get_model(&self) -> Result<Option<String>, String> {
        let env = self.read_env()?;
        Ok(env.gemini_model)
    }

    /// 设置模型
    pub fn set_model(&self, model: Option<String>) -> Result<(), String> {
        let mut env = self.read_env()?;
        env.gemini_model = model;
        self.write_env(&env)
    }

    /// 获取认证模式
    pub fn get_auth_mode(&self) -> Result<Option<String>, String> {
        let settings = self.read_settings()?;
        Ok(settings.auth_mode)
    }

    /// 设置认证模式
    pub fn set_auth_mode(&self, auth_mode: &str) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.auth_mode = Some(auth_mode.to_string());
        self.write_settings(&settings)
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, GeminiMcpServer>, String> {
        let settings = self.read_settings()?;
        Ok(settings.mcp_servers)
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: GeminiMcpServer) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.mcp_servers.insert(name.to_string(), server);
        self.write_settings(&settings)
    }

    /// 删除 MCP 服务器
    pub fn remove_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.mcp_servers.remove(name);
        self.write_settings(&settings)
    }

    /// 同步 MCP 服务器配置
    pub fn sync_mcp_servers(&self, servers: HashMap<String, GeminiMcpServer>) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        settings.mcp_servers = servers;
        self.write_settings(&settings)
    }

    /// 读取 GEMINI.md 系统提示
    pub fn read_gemini_md(&self) -> Result<Option<String>, String> {
        if !self.gemini_md.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&self.gemini_md)
            .map_err(|e| format!("读取 GEMINI.md 失败: {}", e))?;
        
        Ok(Some(content))
    }

    /// 写入 GEMINI.md 系统提示
    pub fn write_gemini_md(&self, content: &str) -> Result<(), String> {
        self.ensure_gemini_dir()?;
        
        fs::write(&self.gemini_md, content)
            .map_err(|e| format!("写入 GEMINI.md 失败: {}", e))
    }

    /// 应用 Provider 配置到 Gemini
    pub fn apply_provider(&self, provider: &GeminiProvider) -> Result<(), String> {
        // 更新 .env
        let mut env = self.read_env()?;
        env.gemini_api_key = Some(provider.api_key.clone());
        
        if let Some(ref base_url) = provider.base_url {
            env.google_gemini_base_url = Some(base_url.clone());
        } else {
            env.google_gemini_base_url = None;
        }
        
        if let Some(ref model) = provider.model {
            env.gemini_model = Some(model.clone());
        }
        
        self.write_env(&env)?;
        
        // 更新 settings.json 的 authMode
        let mut settings = self.read_settings()?;
        settings.auth_mode = Some("api_key".to_string());
        self.write_settings(&settings)
    }

    /// 获取配置目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.gemini_dir
    }

    /// 检查 Gemini 是否已配置
    pub fn is_configured(&self) -> bool {
        self.env_file.exists() || self.settings_json.exists()
    }

    /// 清除 Provider 配置（API Key、Base URL、Model）
    pub fn clear_provider_config(&self) -> Result<(), String> {
        // 清除 .env 文件中的配置
        let mut env = self.read_env()?;
        env.gemini_api_key = None;
        env.google_gemini_api_key = None;
        env.google_gemini_base_url = None;
        env.gemini_model = None;
        self.write_env(&env)
    }

    /// 设置认证类型为 API Key 模式（跳过 OAuth 登录）
    /// 写入 settings.json 中的 security.auth.selectedType: "gemini-api-key"
    pub fn set_api_key_auth_mode(&self) -> Result<(), String> {
        self.set_auth_selected_type("gemini-api-key")
    }

    /// 设置认证类型为 OAuth 模式（Google 官方）
    /// 写入 settings.json 中的 security.auth.selectedType: "oauth-personal"
    pub fn set_oauth_auth_mode(&self) -> Result<(), String> {
        self.set_auth_selected_type("oauth-personal")
    }

    /// 设置 security.auth.selectedType
    fn set_auth_selected_type(&self, selected_type: &str) -> Result<(), String> {
        self.ensure_gemini_dir()?;
        
        let mut settings = self.read_settings()?;
        
        // 确保 security 存在
        if settings.security.is_none() {
            settings.security = Some(GeminiSecurity::default());
        }
        
        // 确保 security.auth 存在
        if let Some(ref mut security) = settings.security {
            if security.auth.is_none() {
                security.auth = Some(GeminiSecurityAuth::default());
            }
            
            // 设置 selectedType
            if let Some(ref mut auth) = security.auth {
                auth.selected_type = Some(selected_type.to_string());
            }
        }
        
        self.write_settings(&settings)
    }

    /// 获取当前认证类型
    pub fn get_auth_selected_type(&self) -> Result<Option<String>, String> {
        let settings = self.read_settings()?;
        Ok(settings
            .security
            .and_then(|s| s.auth)
            .and_then(|a| a.selected_type))
    }

    /// 清除认证类型设置
    pub fn clear_auth_selected_type(&self) -> Result<(), String> {
        let mut settings = self.read_settings()?;
        
        if let Some(ref mut security) = settings.security {
            if let Some(ref mut auth) = security.auth {
                auth.selected_type = None;
            }
        }
        
        self.write_settings(&settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_env_parsing() {
        let manager = GeminiConfigManager::new().unwrap();
        
        let env_content = r#"
GEMINI_API_KEY=test-key-123
GOOGLE_GEMINI_BASE_URL=https://api.example.com
GEMINI_MODEL=gemini-2.5-pro
"#;
        
        let env = manager.parse_env(env_content).unwrap();
        assert_eq!(env.gemini_api_key, Some("test-key-123".to_string()));
        assert_eq!(env.google_gemini_base_url, Some("https://api.example.com".to_string()));
        assert_eq!(env.gemini_model, Some("gemini-2.5-pro".to_string()));
    }

    #[test]
    fn test_gemini_settings_serialization() {
        let mut settings = GeminiSettings::default();
        settings.auth_mode = Some("api_key".to_string());
        
        let server = GeminiMcpServer {
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            env: HashMap::new(),
            url: None,
        };
        settings.mcp_servers.insert("memory".to_string(), server);
        
        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("authMode"));
        assert!(json.contains("mcpServers"));
    }
}
