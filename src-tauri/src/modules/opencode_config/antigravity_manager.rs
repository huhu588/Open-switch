// Google Antigravity (反重力) 配置管理器
// 管理 Antigravity 的配置文件：
// - MCP 配置 (基于 VS Code fork，路径按平台区分)
// - Rules/Customizations 目录

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Antigravity MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// 远程服务器 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// 是否禁用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// 自动批准的工具列表
    #[serde(rename = "autoApprove", skip_serializing_if = "Option::is_none")]
    pub auto_approve: Option<Vec<String>>,
}

/// Antigravity mcp.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AntigravityMcpConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, AntigravityMcpServer>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

pub struct AntigravityConfigManager {
    /// Antigravity 用户配置目录
    config_dir: PathBuf,
    /// MCP 配置文件路径
    mcp_config_json: PathBuf,
    /// 全局 rules 目录
    rules_dir: PathBuf,
}

impl AntigravityConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        let (config_dir, mcp_config_json, rules_dir) = Self::resolve_paths(&user_home)?;

        Ok(Self {
            config_dir,
            mcp_config_json,
            rules_dir,
        })
    }

    /// 按平台解析 Antigravity 路径
    /// Antigravity 基于 VS Code fork，配置路径类似 VS Code
    fn resolve_paths(user_home: &PathBuf) -> Result<(PathBuf, PathBuf, PathBuf), String> {
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA")
                .unwrap_or_else(|_| user_home.join("AppData").join("Roaming").to_string_lossy().to_string());
            let config_dir = PathBuf::from(&app_data).join("Antigravity");
            let mcp_config_json = config_dir.join("User").join("globalStorage").join("mcp.json");
            let rules_dir = config_dir.join("User").join("rules");

            // 如果标准路径不存在，尝试备用路径
            if !config_dir.exists() {
                let alt_config_dir = user_home.join(".antigravity");
                if alt_config_dir.exists() {
                    let alt_mcp = alt_config_dir.join("settings").join("mcp.json");
                    let alt_rules = alt_config_dir.join("rules");
                    return Ok((alt_config_dir, alt_mcp, alt_rules));
                }
            }

            Ok((config_dir, mcp_config_json, rules_dir))
        }

        #[cfg(target_os = "macos")]
        {
            let config_dir = user_home
                .join("Library")
                .join("Application Support")
                .join("Antigravity");
            let mcp_config_json = config_dir.join("User").join("globalStorage").join("mcp.json");
            let rules_dir = config_dir.join("User").join("rules");

            if !config_dir.exists() {
                let alt_config_dir = user_home.join(".antigravity");
                if alt_config_dir.exists() {
                    let alt_mcp = alt_config_dir.join("settings").join("mcp.json");
                    let alt_rules = alt_config_dir.join("rules");
                    return Ok((alt_config_dir, alt_mcp, alt_rules));
                }
            }

            Ok((config_dir, mcp_config_json, rules_dir))
        }

        #[cfg(target_os = "linux")]
        {
            let config_dir = user_home.join(".config").join("Antigravity");
            let mcp_config_json = config_dir.join("User").join("globalStorage").join("mcp.json");
            let rules_dir = config_dir.join("User").join("rules");

            if !config_dir.exists() {
                let alt_config_dir = user_home.join(".antigravity");
                if alt_config_dir.exists() {
                    let alt_mcp = alt_config_dir.join("settings").join("mcp.json");
                    let alt_rules = alt_config_dir.join("rules");
                    return Ok((alt_config_dir, alt_mcp, alt_rules));
                }
            }

            Ok((config_dir, mcp_config_json, rules_dir))
        }
    }

    /// 确保配置目录存在
    fn ensure_config_dir(&self) -> Result<(), String> {
        if let Some(parent) = self.mcp_config_json.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建 Antigravity 配置目录失败: {}", e))?;
            }
        }
        Ok(())
    }

    /// 确保 rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Antigravity rules 目录失败: {}", e))?;
        }
        Ok(())
    }

    // ==================== MCP 管理 ====================

    /// 读取 mcp.json
    pub fn read_mcp_config(&self) -> Result<AntigravityMcpConfig, String> {
        if !self.mcp_config_json.exists() {
            return Ok(AntigravityMcpConfig::default());
        }

        let content = fs::read_to_string(&self.mcp_config_json)
            .map_err(|e| format!("读取 Antigravity mcp.json 失败: {}", e))?;

        let content = content.trim_start_matches('\u{feff}');

        serde_json::from_str(content)
            .map_err(|e| format!("解析 Antigravity mcp.json 失败: {}", e))
    }

    /// 写入 mcp.json
    pub fn write_mcp_config(&self, config: &AntigravityMcpConfig) -> Result<(), String> {
        self.ensure_config_dir()?;

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 Antigravity mcp.json 失败: {}", e))?;

        fs::write(&self.mcp_config_json, content)
            .map_err(|e| format!("写入 Antigravity mcp.json 失败: {}", e))
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, AntigravityMcpServer>, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers)
    }

    /// 获取 MCP 服务器数量
    pub fn get_mcp_count(&self) -> Result<usize, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers.len())
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: AntigravityMcpServer) -> Result<(), String> {
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
    pub fn sync_mcp_servers(&self, servers: HashMap<String, AntigravityMcpServer>) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers = servers;
        self.write_mcp_config(&config)
    }

    // ==================== 路径获取 ====================

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    pub fn get_mcp_config_path(&self) -> &PathBuf {
        &self.mcp_config_json
    }

    // ==================== 状态检测 ====================

    /// 检查 Antigravity 是否已安装
    pub fn is_installed(&self) -> bool {
        self.config_dir.exists()
    }

    /// 检查 MCP 是否已配置
    pub fn is_mcp_configured(&self) -> bool {
        self.mcp_config_json.exists()
    }

    /// 获取 rules 目录中的规则数量
    pub fn get_rules_count(&self) -> usize {
        if !self.rules_dir.is_dir() {
            return 0;
        }

        fs::read_dir(&self.rules_dir)
            .map(|entries| {
                entries.flatten().filter(|entry| {
                    let path = entry.path();
                    path.is_file() && path.extension().map_or(false, |ext| ext == "md")
                }).count()
            })
            .unwrap_or(0)
    }
}
