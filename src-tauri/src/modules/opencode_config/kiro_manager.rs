// Kiro 配置管理器
// 管理 Kiro 的配置文件：
// - ~/.kiro/settings/mcp.json (MCP 配置)
// - ~/.kiro/steering/ (全局 Steering/Rules 目录)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Kiro MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// 远程服务器 URL (Streamable HTTP)
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
    /// 禁用的工具列表
    #[serde(rename = "disabledTools", skip_serializing_if = "Option::is_none")]
    pub disabled_tools: Option<Vec<String>>,
}

/// Kiro mcp.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KiroMcpConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, KiroMcpServer>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

pub struct KiroConfigManager {
    /// Kiro 全局配置目录 (~/.kiro/)
    kiro_dir: PathBuf,
    /// settings 目录 (~/.kiro/settings/)
    settings_dir: PathBuf,
    /// mcp.json 路径 (~/.kiro/settings/mcp.json)
    mcp_config_json: PathBuf,
    /// 全局 steering/rules 目录 (~/.kiro/steering/)
    rules_dir: PathBuf,
}

impl KiroConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        let kiro_dir = user_home.join(".kiro");
        let settings_dir = kiro_dir.join("settings");
        let mcp_config_json = settings_dir.join("mcp.json");
        let rules_dir = kiro_dir.join("steering");

        Ok(Self {
            kiro_dir,
            settings_dir,
            mcp_config_json,
            rules_dir,
        })
    }

    /// 确保 settings 目录存在
    fn ensure_settings_dir(&self) -> Result<(), String> {
        if !self.settings_dir.exists() {
            fs::create_dir_all(&self.settings_dir)
                .map_err(|e| format!("创建 Kiro settings 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 确保 steering/rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Kiro steering 目录失败: {}", e))?;
        }
        Ok(())
    }

    // ==================== MCP 管理 ====================

    /// 读取 mcp.json
    pub fn read_mcp_config(&self) -> Result<KiroMcpConfig, String> {
        if !self.mcp_config_json.exists() {
            return Ok(KiroMcpConfig::default());
        }

        let content = fs::read_to_string(&self.mcp_config_json)
            .map_err(|e| format!("读取 Kiro mcp.json 失败: {}", e))?;

        // 移除 UTF-8 BOM
        let content = content.trim_start_matches('\u{feff}');

        serde_json::from_str(content)
            .map_err(|e| format!("解析 Kiro mcp.json 失败: {}", e))
    }

    /// 写入 mcp.json
    pub fn write_mcp_config(&self, config: &KiroMcpConfig) -> Result<(), String> {
        self.ensure_settings_dir()?;

        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 Kiro mcp.json 失败: {}", e))?;

        fs::write(&self.mcp_config_json, content)
            .map_err(|e| format!("写入 Kiro mcp.json 失败: {}", e))
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, KiroMcpServer>, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers)
    }

    /// 获取 MCP 服务器数量
    pub fn get_mcp_count(&self) -> Result<usize, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers.len())
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: KiroMcpServer) -> Result<(), String> {
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
    pub fn sync_mcp_servers(&self, servers: HashMap<String, KiroMcpServer>) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers = servers;
        self.write_mcp_config(&config)
    }

    // ==================== 路径获取 ====================

    /// 获取 Kiro 配置根目录
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.kiro_dir
    }

    /// 获取 rules/steering 目录路径
    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    /// 获取 mcp.json 路径
    pub fn get_mcp_config_path(&self) -> &PathBuf {
        &self.mcp_config_json
    }

    // ==================== 状态检测 ====================

    /// 检查 Kiro 是否已安装
    pub fn is_installed(&self) -> bool {
        self.kiro_dir.exists()
    }

    /// 检查 MCP 是否已配置
    pub fn is_mcp_configured(&self) -> bool {
        self.mcp_config_json.exists()
    }

    /// 获取 steering 目录中的规则数量
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
