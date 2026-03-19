// Windsurf 配置管理器
// 管理 Windsurf 的配置文件：
// - ~/.codeium/windsurf/mcp_config.json (MCP 配置)
// - .windsurf/rules/ (项目级规则目录)
// - .windsurf/skills/ (项目级 Skills 目录)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Windsurf MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindsurfMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// SSE/HTTP 模式的服务器 URL
    #[serde(rename = "serverUrl", skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// 是否禁用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// 允许的工具列表
    #[serde(rename = "alwaysAllow", skip_serializing_if = "Option::is_none")]
    pub always_allow: Option<Vec<String>>,
}

/// Windsurf mcp_config.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindsurfMcpConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, WindsurfMcpServer>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

pub struct WindsurfConfigManager {
    /// Windsurf 用户配置目录 (~/.codeium/windsurf/)
    codeium_dir: PathBuf,
    /// mcp_config.json 路径
    mcp_config_json: PathBuf,
    /// 项目级 rules 目录 (.windsurf/rules/)
    rules_dir: PathBuf,
    /// 项目级 skills 目录 (.windsurf/skills/)
    skills_dir: PathBuf,
}

impl WindsurfConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        // 跨平台：macOS/Linux 使用 ~/.codeium/windsurf/，Windows 使用 %USERPROFILE%\.codeium\windsurf\
        let codeium_dir = user_home.join(".codeium").join("windsurf");
        let mcp_config_json = codeium_dir.join("mcp_config.json");

        // 项目级目录
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| user_home.clone());
        let rules_dir = cwd.join(".windsurf").join("rules");
        let skills_dir = cwd.join(".windsurf").join("skills");

        Ok(Self {
            codeium_dir,
            mcp_config_json,
            rules_dir,
            skills_dir,
        })
    }

    /// 确保 Windsurf 配置目录存在
    fn ensure_codeium_dir(&self) -> Result<(), String> {
        if !self.codeium_dir.exists() {
            fs::create_dir_all(&self.codeium_dir)
                .map_err(|e| format!("创建 Windsurf 配置目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 确保 rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Windsurf rules 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 确保 skills 目录存在
    pub fn ensure_skills_dir(&self) -> Result<(), String> {
        if !self.skills_dir.exists() {
            fs::create_dir_all(&self.skills_dir)
                .map_err(|e| format!("创建 Windsurf skills 目录失败: {}", e))?;
        }
        Ok(())
    }

    // ==================== MCP 管理 ====================

    /// 读取 mcp_config.json
    pub fn read_mcp_config(&self) -> Result<WindsurfMcpConfig, String> {
        if !self.mcp_config_json.exists() {
            return Ok(WindsurfMcpConfig::default());
        }

        let content = fs::read_to_string(&self.mcp_config_json)
            .map_err(|e| format!("读取 Windsurf mcp_config.json 失败: {}", e))?;

        // 移除 UTF-8 BOM
        let content = content.trim_start_matches('\u{feff}');

        serde_json::from_str(content)
            .map_err(|e| format!("解析 Windsurf mcp_config.json 失败: {}", e))
    }

    /// 写入 mcp_config.json
    pub fn write_mcp_config(&self, config: &WindsurfMcpConfig) -> Result<(), String> {
        self.ensure_codeium_dir()?;

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 Windsurf mcp_config.json 失败: {}", e))?;

        fs::write(&self.mcp_config_json, content)
            .map_err(|e| format!("写入 Windsurf mcp_config.json 失败: {}", e))
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, WindsurfMcpServer>, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers)
    }

    /// 获取 MCP 服务器数量
    pub fn get_mcp_count(&self) -> Result<usize, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers.len())
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: WindsurfMcpServer) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers.insert(name.to_string(), server);
        self.write_mcp_config(&config)
    }

    /// 删除 MCP 服务器
    pub fn remove_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers.remove(name);
        self.write_mcp_config(&config)
    }

    /// 同步 MCP 服务器配置（从 Ai Switch 格式转换）
    pub fn sync_mcp_servers(&self, servers: HashMap<String, WindsurfMcpServer>) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers = servers;
        self.write_mcp_config(&config)
    }

    // ==================== 路径获取 ====================

    /// 获取配置目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.codeium_dir
    }

    /// 获取 rules 目录路径
    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    /// 获取 skills 目录路径
    pub fn get_skills_dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// 获取 mcp_config.json 路径
    pub fn get_mcp_config_path(&self) -> &PathBuf {
        &self.mcp_config_json
    }

    // ==================== 状态检测 ====================

    /// 检查 Windsurf 是否已安装
    pub fn is_installed(&self) -> bool {
        self.codeium_dir.exists()
    }

    /// 检查 MCP 是否已配置
    pub fn is_mcp_configured(&self) -> bool {
        self.mcp_config_json.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windsurf_mcp_config_serialization() {
        let mut config = WindsurfMcpConfig::default();
        let server = WindsurfMcpServer {
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
            env: HashMap::new(),
            server_url: None,
            headers: HashMap::new(),
            disabled: None,
            always_allow: None,
        };
        config.mcp_servers.insert("github".to_string(), server);

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("mcpServers"));
        assert!(json.contains("github"));
    }

    #[test]
    fn test_windsurf_sse_server_serialization() {
        let server = WindsurfMcpServer {
            command: None,
            args: vec![],
            env: HashMap::new(),
            server_url: Some("https://example.com/mcp".to_string()),
            headers: {
                let mut h = HashMap::new();
                h.insert("Authorization".to_string(), "Bearer ${env:TOKEN}".to_string());
                h
            },
            disabled: None,
            always_allow: None,
        };

        let json = serde_json::to_string_pretty(&server).unwrap();
        assert!(json.contains("serverUrl"));
        assert!(json.contains("${env:TOKEN}"));
    }
}
