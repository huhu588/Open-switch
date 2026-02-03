// Cursor 配置管理器
// 管理 Cursor 的配置文件：
// - ~/.cursor/mcp.json (MCP 配置)
// - ~/.cursor/rules/ (规则目录)
// - ~/.cursor/skills/ (Skills 目录)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Cursor MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Cursor mcp.json 配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorMcpConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, CursorMcpServer>,
    /// 其他未知字段保留
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

pub struct CursorConfigManager {
    // Cursor 用户配置目录
    cursor_dir: PathBuf,
    // mcp.json 路径
    mcp_json: PathBuf,
    // rules 目录路径
    rules_dir: PathBuf,
    // skills 目录路径
    skills_dir: PathBuf,
}

impl CursorConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;
        
        let cursor_dir = user_home.join(".cursor");
        let mcp_json = cursor_dir.join("mcp.json");
        let rules_dir = cursor_dir.join("rules");
        // Cursor 优先使用 skills-cursor 目录，如果不存在则使用 skills
        let skills_cursor = cursor_dir.join("skills-cursor");
        let skills_dir = if skills_cursor.exists() {
            skills_cursor
        } else {
            cursor_dir.join("skills")
        };
        
        Ok(Self {
            cursor_dir,
            mcp_json,
            rules_dir,
            skills_dir,
        })
    }

    /// 确保 Cursor 目录存在
    fn ensure_cursor_dir(&self) -> Result<(), String> {
        if !self.cursor_dir.exists() {
            fs::create_dir_all(&self.cursor_dir)
                .map_err(|e| format!("创建 Cursor 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 确保 rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        self.ensure_cursor_dir()?;
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Cursor rules 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 确保 skills 目录存在
    pub fn ensure_skills_dir(&self) -> Result<(), String> {
        self.ensure_cursor_dir()?;
        if !self.skills_dir.exists() {
            fs::create_dir_all(&self.skills_dir)
                .map_err(|e| format!("创建 Cursor skills 目录失败: {}", e))?;
        }
        Ok(())
    }

    /// 读取 mcp.json
    pub fn read_mcp_config(&self) -> Result<CursorMcpConfig, String> {
        if !self.mcp_json.exists() {
            return Ok(CursorMcpConfig::default());
        }
        
        let content = fs::read_to_string(&self.mcp_json)
            .map_err(|e| format!("读取 mcp.json 失败: {}", e))?;
        
        // 移除 UTF-8 BOM
        let content = content.trim_start_matches('\u{feff}');
        
        serde_json::from_str(content)
            .map_err(|e| format!("解析 mcp.json 失败: {}", e))
    }

    /// 写入 mcp.json
    pub fn write_mcp_config(&self, config: &CursorMcpConfig) -> Result<(), String> {
        self.ensure_cursor_dir()?;
        
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 mcp.json 失败: {}", e))?;
        
        fs::write(&self.mcp_json, content)
            .map_err(|e| format!("写入 mcp.json 失败: {}", e))
    }

    /// 获取 MCP 服务器列表
    pub fn get_mcp_servers(&self) -> Result<HashMap<String, CursorMcpServer>, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers)
    }

    /// 获取 MCP 服务器数量
    pub fn get_mcp_count(&self) -> Result<usize, String> {
        let config = self.read_mcp_config()?;
        Ok(config.mcp_servers.len())
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: &str, server: CursorMcpServer) -> Result<(), String> {
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
    pub fn sync_mcp_servers(&self, servers: HashMap<String, CursorMcpServer>) -> Result<(), String> {
        let mut config = self.read_mcp_config()?;
        config.mcp_servers = servers;
        self.write_mcp_config(&config)
    }

    /// 获取配置目录路径
    pub fn get_config_dir(&self) -> &PathBuf {
        &self.cursor_dir
    }

    /// 获取 rules 目录路径
    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    /// 获取 skills 目录路径
    pub fn get_skills_dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// 获取 mcp.json 路径
    pub fn get_mcp_json_path(&self) -> &PathBuf {
        &self.mcp_json
    }

    /// 检查 Cursor 是否已配置 MCP
    pub fn is_mcp_configured(&self) -> bool {
        self.mcp_json.exists()
    }

    /// 检查 Cursor 是否已安装
    pub fn is_cursor_installed(&self) -> bool {
        self.cursor_dir.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_mcp_config_serialization() {
        let mut config = CursorMcpConfig::default();
        let server = CursorMcpServer {
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            env: HashMap::new(),
            url: None,
        };
        config.mcp_servers.insert("memory".to_string(), server);
        
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("mcpServers"));
        assert!(json.contains("memory"));
        assert!(json.contains("npx"));
    }

    #[test]
    fn test_cursor_mcp_server_serialization() {
        let server = CursorMcpServer {
            command: Some("npx".to_string()),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
            env: HashMap::new(),
            url: None,
        };
        
        let json = serde_json::to_string_pretty(&server).unwrap();
        assert!(json.contains("npx"));
        assert!(json.contains("server-filesystem"));
    }
}
