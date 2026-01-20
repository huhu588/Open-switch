// MCP 服务器相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::{ConfigManager, McpServer, McpServerType, McpOAuthConfig};
use crate::error::AppError;

/// MCP 服务器列表项
#[derive(Debug, Clone, Serialize)]
pub struct McpServerItem {
    pub name: String,
    pub server_type: String,
    pub enabled: bool,
    pub url: Option<String>,
    pub command: Option<Vec<String>>,
    /// 配置文件安装位置
    pub install_path: String,
    /// 包名（从npm命令中提取）
    pub package_name: Option<String>,
}

/// MCP 服务器输入
#[derive(Debug, Deserialize)]
pub struct McpServerInput {
    pub name: String,
    pub server_type: String, // "local" | "remote"
    pub enabled: bool,
    pub timeout: Option<u32>,
    // Local
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    // Remote
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub oauth: Option<OAuthInput>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthInput {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

/// 同步配置参数
#[derive(Debug, Deserialize)]
pub struct SyncMcpInput {
    pub server_names: Vec<String>,
    pub sync_to_global: bool,
    pub sync_to_project: bool,
}

/// 从命令中提取npm包名
fn extract_package_name(command: &Option<Vec<String>>) -> Option<String> {
    if let Some(cmd) = command {
        // 查找 @ 开头的包名，如 @modelcontextprotocol/server-memory
        for arg in cmd {
            if arg.starts_with('@') || (arg.contains('/') && !arg.starts_with('-')) {
                return Some(arg.clone());
            }
            // 处理类似 oh-my-opencode 这样的包名
            if !arg.starts_with('-') && !arg.starts_with('/') && arg.contains('-') && !cmd.first().map(|s| s == arg).unwrap_or(false) {
                // 确保不是第一个参数（命令本身）
                if let Some(first) = cmd.first() {
                    if first == "npx" || first == "bunx" || first == "node" {
                        return Some(arg.clone());
                    }
                }
            }
        }
    }
    None
}

/// 获取所有 MCP 服务器列表
#[tauri::command]
pub fn get_mcp_servers(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<McpServerItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mcp_config = manager.mcp().read_config()?;
    
    // 获取 MCP 配置目录路径
    let mcp_dir = manager.mcp().get_mcp_dir();
    
    let mut items: Vec<McpServerItem> = mcp_config
        .servers
        .iter()
        .map(|(name, server)| {
            // 构建配置文件的安装位置
            let install_path = mcp_dir.join(format!("{}.json", name))
                .to_string_lossy()
                .to_string();
            
            // 从npm命令中提取包名
            let package_name = extract_package_name(&server.command);
            
            McpServerItem {
                name: name.clone(),
                server_type: server.server_type.to_string(),
                enabled: server.enabled,
                url: server.url.clone(),
                command: server.command.clone(),
                install_path,
                package_name,
            }
        })
        .collect();
    
    items.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(items)
}

/// 获取单个 MCP 服务器详情
#[tauri::command]
pub fn get_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Option<McpServer>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    Ok(manager.mcp().get_server(&name)?)
}

/// 添加 MCP 服务器
#[tauri::command]
pub fn add_mcp_server(
    input: McpServerInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let server = build_mcp_server(&input)?;
    manager.mcp_mut().save_server(&input.name, server)?;
    
    // 自动同步到 OpenCode
    let _ = manager.mcp().sync_to_opencode(None);
    
    Ok(())
}

/// 更新 MCP 服务器
#[tauri::command]
pub fn update_mcp_server(
    old_name: String,
    input: McpServerInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 如果名称变了，需要重命名
    if old_name != input.name {
        manager.mcp_mut().rename_server(&old_name, &input.name)?;
    }
    
    let server = build_mcp_server(&input)?;
    manager.mcp_mut().save_server(&input.name, server)?;
    
    // 自动同步到 OpenCode
    let _ = manager.mcp().sync_to_opencode(None);
    
    Ok(())
}

/// 删除 MCP 服务器
#[tauri::command]
pub fn delete_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.mcp_mut().delete_server(&name)?;
    
    // 自动同步到 OpenCode
    let _ = manager.mcp().sync_to_opencode(None);
    
    Ok(())
}

/// 切换 MCP 服务器启用状态
#[tauri::command]
pub fn toggle_mcp_server(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<bool, AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let new_state = manager.mcp_mut().toggle_server_enabled(&name)?;
    
    // 自动同步到 OpenCode
    let _ = manager.mcp().sync_to_opencode(None);
    
    Ok(new_state)
}

/// 检查 MCP 服务器是否可用（尝试启动）
#[tauri::command]
pub fn check_mcp_server_health(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<McpHealthResult, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let server = manager.mcp().get_server(&name)?
        .ok_or_else(|| AppError::Custom(format!("MCP 服务器 '{}' 不存在", name)))?;
    
    // 只检查本地服务器
    if let Some(ref cmd) = server.command {
        if cmd.is_empty() {
            return Ok(McpHealthResult {
                healthy: false,
                message: "命令为空".to_string(),
            });
        }
        
        // 检查命令是否存在
        let program = &cmd[0];
        
        // Windows 上需要使用 cmd /c 来执行 npx 等命令
        #[cfg(windows)]
        let check_result = {
            if program == "npx" || program == "node" || program == "bun" || program == "bunx" {
                std::process::Command::new("cmd")
                    .args(["/c", program, "--version"])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output()
            } else {
                std::process::Command::new("cmd")
                    .args(["/c", program, "--help"])
                    .creation_flags(0x08000000)
                    .output()
            }
        };
        
        #[cfg(not(windows))]
        let check_result = {
            if program == "npx" || program == "node" || program == "bun" || program == "bunx" {
                std::process::Command::new(program)
                    .arg("--version")
                    .output()
            } else {
                std::process::Command::new(program)
                    .arg("--help")
                    .output()
            }
        };
        
        match check_result {
            Ok(output) if output.status.success() => {
                Ok(McpHealthResult {
                    healthy: true,
                    message: format!("{} 可用", program),
                })
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(McpHealthResult {
                    healthy: false,
                    message: format!("{} 执行失败: {}", program, stderr.lines().next().unwrap_or("未知错误")),
                })
            }
            Err(e) => {
                Ok(McpHealthResult {
                    healthy: false,
                    message: format!("{} 未找到: {}", program, e),
                })
            }
        }
    } else {
        Ok(McpHealthResult {
            healthy: false,
            message: "未配置启动命令".to_string(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct McpHealthResult {
    pub healthy: bool,
    pub message: String,
}

/// 同步 MCP 配置到 opencode.json
#[tauri::command]
pub fn sync_mcp_config(
    input: SyncMcpInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let names = if input.server_names.is_empty() {
        None
    } else {
        Some(input.server_names.as_slice())
    };
    
    if input.sync_to_global {
        manager.mcp().sync_to_opencode(names)?;
    }
    
    if input.sync_to_project {
        manager.mcp().sync_to_project(names)?;
    }
    
    Ok(())
}

/// 默认推荐的 MCP 服务器配置
#[derive(Debug, Clone, Serialize)]
pub struct DefaultMcpServer {
    pub name: String,
    pub description: String,
    pub command: Vec<String>,
    pub url: String,
}

/// 获取推荐的 MCP 服务器列表
#[tauri::command]
pub fn get_recommended_mcp_servers() -> Vec<DefaultMcpServer> {
    vec![
        // 官方服务器
        DefaultMcpServer {
            name: "server-memory".to_string(),
            description: "知识图谱记忆 - 用于实体和关系的持久化存储".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-memory".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-sequential-thinking".to_string(),
            description: "顺序思考 - 用于复杂问题的逐步推理".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-sequential-thinking".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-filesystem".to_string(),
            description: "文件系统 - 安全的文件读写操作".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string(), "/path/to/allowed/files".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-fetch".to_string(),
            description: "网页获取 - 获取网页内容并转换为 LLM 可用格式".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-fetch".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-github".to_string(),
            description: "GitHub 集成 - 管理 Issues、PR、仓库等".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-github".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-git".to_string(),
            description: "Git 操作 - 读取、搜索和操作 Git 仓库".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-git".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        // 社区热门服务器
        DefaultMcpServer {
            name: "context7-mcp".to_string(),
            description: "Context7 - 获取最新的文档和代码示例".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@upstash/context7-mcp@latest".to_string()],
            url: "https://context7.com".to_string(),
        },
        DefaultMcpServer {
            name: "playwright".to_string(),
            description: "Playwright - 浏览器自动化、网页抓取和测试".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@playwright/mcp@latest".to_string()],
            url: "https://github.com/microsoft/playwright-mcp".to_string(),
        },
        DefaultMcpServer {
            name: "brave-search".to_string(),
            description: "Brave 搜索 - 网页、图片、视频、新闻搜索".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@anthropics/mcp-server-brave-search".to_string()],
            url: "https://github.com/anthropics/mcp-servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-postgres".to_string(),
            description: "PostgreSQL - 数据库查询和操作".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-postgres".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
        DefaultMcpServer {
            name: "server-sqlite".to_string(),
            description: "SQLite - 轻量级数据库操作".to_string(),
            command: vec!["npx".to_string(), "-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()],
            url: "https://github.com/modelcontextprotocol/servers".to_string(),
        },
    ]
}

/// 添加推荐的 MCP 服务器（批量）
#[tauri::command]
pub fn add_recommended_mcp_servers(
    server_names: Vec<String>,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<AddRecommendedResult, AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let recommended = get_recommended_mcp_servers();
    let mut added = Vec::new();
    let mut skipped = Vec::new();
    
    for name in &server_names {
        if let Some(rec) = recommended.iter().find(|r| &r.name == name) {
            // 检查是否已存在
            match manager.mcp().get_server(name) {
                Ok(Some(_)) => {
                    skipped.push(name.clone());
                }
                _ => {
                    // 添加服务器
                    manager.mcp_mut().add_local_server(
                        rec.name.clone(),
                        rec.command.clone(),
                        HashMap::new(),
                        None,
                    )?;
                    added.push(name.clone());
                }
            }
        }
    }
    
    // 自动同步到 OpenCode
    if !added.is_empty() {
        let _ = manager.mcp().sync_to_opencode(None);
    }
    
    Ok(AddRecommendedResult { added, skipped })
}

#[derive(Debug, Serialize)]
pub struct AddRecommendedResult {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
}

/// 构建 McpServer 对象
fn build_mcp_server(input: &McpServerInput) -> Result<McpServer, AppError> {
    let server_type = match input.server_type.as_str() {
        "local" => McpServerType::Local,
        "remote" => McpServerType::Remote,
        _ => return Err(AppError::Custom("无效的服务器类型".to_string())),
    };
    
    let oauth = input.oauth.as_ref().map(|o| McpOAuthConfig {
        client_id: o.client_id.clone(),
        client_secret: o.client_secret.clone(),
        scope: o.scope.clone(),
    });
    
    Ok(McpServer {
        server_type,
        enabled: input.enabled,
        timeout: input.timeout,
        command: input.command.clone(),
        environment: input.environment.clone().unwrap_or_default(),
        url: input.url.clone(),
        headers: input.headers.clone().unwrap_or_default(),
        oauth,
        metadata: Default::default(),
    })
}
