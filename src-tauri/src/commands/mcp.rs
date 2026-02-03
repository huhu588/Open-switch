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

/// 聚合的 MCP 管理信息（类似 ManagedSkill）
#[derive(Debug, Clone, Serialize)]
pub struct ManagedMcp {
    pub name: String,
    pub server_type: String,
    pub command: Option<Vec<String>>,
    pub url: Option<String>,
    pub package_name: Option<String>,
    // 各应用启用状态
    pub opencode_enabled: bool,
    pub claude_enabled: bool,
    pub codex_enabled: bool,
    pub gemini_enabled: bool,
    pub cursor_enabled: bool,
}

/// MCP 统计信息
#[derive(Debug, Clone, Serialize, Default)]
pub struct McpStats {
    pub opencode_count: usize,
    pub claude_count: usize,
    pub codex_count: usize,
    pub gemini_count: usize,
    pub cursor_count: usize,
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
    manager.mcp().sync_to_opencode(None)?;
    
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
    manager.mcp().sync_to_opencode(None)?;
    
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
    manager.mcp().sync_to_opencode(None)?;
    
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
    manager.mcp().sync_to_opencode(None)?;
    
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
        manager.mcp().sync_to_opencode(None)?;
    }
    
    Ok(AddRecommendedResult { added, skipped })
}

#[derive(Debug, Serialize)]
pub struct AddRecommendedResult {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
}

/// 跨应用 MCP 同步目标
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum McpSyncTarget {
    OpenCode,
    ClaudeCode,
    Codex,
    Gemini,
    Cursor,
}

/// 跨应用 MCP 同步输入
#[derive(Debug, Deserialize)]
pub struct CrossAppMcpSyncInput {
    pub server_names: Vec<String>,
    pub targets: Vec<McpSyncTarget>,
}

/// 跨应用 MCP 同步结果
#[derive(Debug, Serialize)]
pub struct CrossAppMcpSyncResult {
    pub target: String,
    pub success: bool,
    pub message: String,
    pub synced_count: usize,
}

/// 同步 MCP 服务器到多个应用
#[tauri::command]
pub fn sync_mcp_to_apps(
    input: CrossAppMcpSyncInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<CrossAppMcpSyncResult>, AppError> {
    use crate::config::claude_code_manager::{ClaudeCodeConfigManager, ClaudeMcpServer};
    use crate::config::codex_manager::{CodexConfigManager, CodexMcpServer};
    use crate::config::gemini_manager::{GeminiConfigManager, GeminiMcpServer};
    use crate::config::cursor_manager::{CursorConfigManager, CursorMcpServer};
    
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 获取要同步的服务器
    let mcp_config = manager.mcp().read_config()?;
    let servers_to_sync: Vec<(&String, &McpServer)> = if input.server_names.is_empty() {
        mcp_config.servers.iter().filter(|(_, s)| s.enabled).collect()
    } else {
        mcp_config.servers.iter()
            .filter(|(name, s)| s.enabled && input.server_names.contains(name))
            .collect()
    };
    
    let mut results = Vec::new();
    
    for target in &input.targets {
        let result = match target {
            McpSyncTarget::OpenCode => {
                // 使用现有的 OpenCode 同步逻辑
                let names: Option<&[String]> = if input.server_names.is_empty() {
                    None
                } else {
                    Some(&input.server_names)
                };
                
                match manager.mcp().sync_to_opencode(names) {
                    Ok(_) => CrossAppMcpSyncResult {
                        target: "OpenCode".to_string(),
                        success: true,
                        message: "同步成功".to_string(),
                        synced_count: servers_to_sync.len(),
                    },
                    Err(e) => CrossAppMcpSyncResult {
                        target: "OpenCode".to_string(),
                        success: false,
                        message: e.to_string(),
                        synced_count: 0,
                    },
                }
            }
            McpSyncTarget::ClaudeCode => {
                match ClaudeCodeConfigManager::new() {
                    Ok(claude_manager) => {
                        // 转换为 Claude Code 格式
                        let claude_servers: HashMap<String, ClaudeMcpServer> = servers_to_sync.iter()
                            .map(|(name, server)| {
                                let claude_server = ClaudeMcpServer {
                                    command: server.command.as_ref().and_then(|c| c.first().cloned()),
                                    args: server.command.as_ref()
                                        .map(|c| c.iter().skip(1).cloned().collect())
                                        .unwrap_or_default(),
                                    env: server.environment.clone(),
                                    url: server.url.clone(),
                                    headers: server.headers.clone(),
                                };
                                ((*name).clone(), claude_server)
                            })
                            .collect();
                        
                        match claude_manager.sync_mcp_servers(claude_servers) {
                            Ok(_) => CrossAppMcpSyncResult {
                                target: "Claude Code".to_string(),
                                success: true,
                                message: "同步成功".to_string(),
                                synced_count: servers_to_sync.len(),
                            },
                            Err(e) => CrossAppMcpSyncResult {
                                target: "Claude Code".to_string(),
                                success: false,
                                message: e,
                                synced_count: 0,
                            },
                        }
                    }
                    Err(e) => CrossAppMcpSyncResult {
                        target: "Claude Code".to_string(),
                        success: false,
                        message: e,
                        synced_count: 0,
                    },
                }
            }
            McpSyncTarget::Codex => {
                match CodexConfigManager::new() {
                    Ok(codex_manager) => {
                        // 转换为 Codex 格式
                        let codex_servers: HashMap<String, CodexMcpServer> = servers_to_sync.iter()
                            .filter_map(|(name, server)| {
                                // Codex 只支持本地服务器
                                server.command.as_ref().map(|cmd| {
                                    let codex_server = CodexMcpServer {
                                        command: cmd.clone(),
                                        env: server.environment.clone(),
                                    };
                                    ((*name).clone(), codex_server)
                                })
                            })
                            .collect();
                        
                        match codex_manager.sync_mcp_servers(codex_servers) {
                            Ok(_) => CrossAppMcpSyncResult {
                                target: "Codex".to_string(),
                                success: true,
                                message: "同步成功".to_string(),
                                synced_count: servers_to_sync.len(),
                            },
                            Err(e) => CrossAppMcpSyncResult {
                                target: "Codex".to_string(),
                                success: false,
                                message: e,
                                synced_count: 0,
                            },
                        }
                    }
                    Err(e) => CrossAppMcpSyncResult {
                        target: "Codex".to_string(),
                        success: false,
                        message: e,
                        synced_count: 0,
                    },
                }
            }
            McpSyncTarget::Gemini => {
                match GeminiConfigManager::new() {
                    Ok(gemini_manager) => {
                        // 转换为 Gemini 格式
                        let gemini_servers: HashMap<String, GeminiMcpServer> = servers_to_sync.iter()
                            .map(|(name, server)| {
                                let gemini_server = GeminiMcpServer {
                                    command: server.command.as_ref().and_then(|c| c.first().cloned()),
                                    args: server.command.as_ref()
                                        .map(|c| c.iter().skip(1).cloned().collect())
                                        .unwrap_or_default(),
                                    env: server.environment.clone(),
                                    url: server.url.clone(),
                                };
                                ((*name).clone(), gemini_server)
                            })
                            .collect();
                        
                        match gemini_manager.sync_mcp_servers(gemini_servers) {
                            Ok(_) => CrossAppMcpSyncResult {
                                target: "Gemini CLI".to_string(),
                                success: true,
                                message: "同步成功".to_string(),
                                synced_count: servers_to_sync.len(),
                            },
                            Err(e) => CrossAppMcpSyncResult {
                                target: "Gemini CLI".to_string(),
                                success: false,
                                message: e,
                                synced_count: 0,
                            },
                        }
                    }
                    Err(e) => CrossAppMcpSyncResult {
                        target: "Gemini CLI".to_string(),
                        success: false,
                        message: e,
                        synced_count: 0,
                    },
                }
            }
            McpSyncTarget::Cursor => {
                match CursorConfigManager::new() {
                    Ok(cursor_manager) => {
                        // 转换为 Cursor 格式
                        let cursor_servers: HashMap<String, CursorMcpServer> = servers_to_sync.iter()
                            .map(|(name, server)| {
                                let cursor_server = CursorMcpServer {
                                    command: server.command.as_ref().and_then(|c| c.first().cloned()),
                                    args: server.command.as_ref()
                                        .map(|c| c.iter().skip(1).cloned().collect())
                                        .unwrap_or_default(),
                                    env: server.environment.clone(),
                                    url: server.url.clone(),
                                };
                                ((*name).clone(), cursor_server)
                            })
                            .collect();
                        
                        match cursor_manager.sync_mcp_servers(cursor_servers) {
                            Ok(_) => CrossAppMcpSyncResult {
                                target: "Cursor".to_string(),
                                success: true,
                                message: "同步成功".to_string(),
                                synced_count: servers_to_sync.len(),
                            },
                            Err(e) => CrossAppMcpSyncResult {
                                target: "Cursor".to_string(),
                                success: false,
                                message: e,
                                synced_count: 0,
                            },
                        }
                    }
                    Err(e) => CrossAppMcpSyncResult {
                        target: "Cursor".to_string(),
                        success: false,
                        message: e,
                        synced_count: 0,
                    },
                }
            }
        };
        
        results.push(result);
    }
    
    Ok(results)
}

/// 获取各应用的 MCP 配置状态
#[derive(Debug, Serialize)]
pub struct AppMcpStatus {
    pub app_name: String,
    pub is_configured: bool,
    pub server_count: usize,
    pub server_names: Vec<String>,
}

#[tauri::command]
pub fn get_apps_mcp_status(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<AppMcpStatus>, AppError> {
    use crate::config::claude_code_manager::ClaudeCodeConfigManager;
    use crate::config::codex_manager::CodexConfigManager;
    use crate::config::gemini_manager::GeminiConfigManager;
    use crate::config::cursor_manager::CursorConfigManager;
    
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let mut statuses = Vec::new();
    
    // OpenCode
    let opencode_mcp = manager.mcp().read_config()?;
    let opencode_servers: Vec<String> = opencode_mcp.servers.keys().cloned().collect();
    statuses.push(AppMcpStatus {
        app_name: "OpenCode".to_string(),
        is_configured: !opencode_servers.is_empty(),
        server_count: opencode_servers.len(),
        server_names: opencode_servers,
    });
    
    // Claude Code
    if let Ok(claude_manager) = ClaudeCodeConfigManager::new() {
        if let Ok(claude_servers) = claude_manager.get_mcp_servers() {
            let server_names: Vec<String> = claude_servers.keys().cloned().collect();
            statuses.push(AppMcpStatus {
                app_name: "Claude Code".to_string(),
                is_configured: !server_names.is_empty(),
                server_count: server_names.len(),
                server_names,
            });
        } else {
            statuses.push(AppMcpStatus {
                app_name: "Claude Code".to_string(),
                is_configured: false,
                server_count: 0,
                server_names: Vec::new(),
            });
        }
    }
    
    // Codex
    if let Ok(codex_manager) = CodexConfigManager::new() {
        if let Ok(codex_servers) = codex_manager.get_mcp_servers() {
            let server_names: Vec<String> = codex_servers.keys().cloned().collect();
            statuses.push(AppMcpStatus {
                app_name: "Codex".to_string(),
                is_configured: !server_names.is_empty(),
                server_count: server_names.len(),
                server_names,
            });
        } else {
            statuses.push(AppMcpStatus {
                app_name: "Codex".to_string(),
                is_configured: false,
                server_count: 0,
                server_names: Vec::new(),
            });
        }
    }
    
    // Gemini
    if let Ok(gemini_manager) = GeminiConfigManager::new() {
        if let Ok(gemini_servers) = gemini_manager.get_mcp_servers() {
            let server_names: Vec<String> = gemini_servers.keys().cloned().collect();
            statuses.push(AppMcpStatus {
                app_name: "Gemini CLI".to_string(),
                is_configured: !server_names.is_empty(),
                server_count: server_names.len(),
                server_names,
            });
        } else {
            statuses.push(AppMcpStatus {
                app_name: "Gemini CLI".to_string(),
                is_configured: false,
                server_count: 0,
                server_names: Vec::new(),
            });
        }
    }
    
    // Cursor
    if let Ok(cursor_manager) = CursorConfigManager::new() {
        if let Ok(cursor_servers) = cursor_manager.get_mcp_servers() {
            let server_names: Vec<String> = cursor_servers.keys().cloned().collect();
            statuses.push(AppMcpStatus {
                app_name: "Cursor".to_string(),
                is_configured: !server_names.is_empty(),
                server_count: server_names.len(),
                server_names,
            });
        } else {
            statuses.push(AppMcpStatus {
                app_name: "Cursor".to_string(),
                is_configured: false,
                server_count: 0,
                server_names: Vec::new(),
            });
        }
    }
    
    Ok(statuses)
}

/// 从其他应用导入 MCP 配置
#[derive(Debug, Serialize)]
pub struct ImportMcpResult {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

/// 从其他应用导入 MCP 配置到 Ai Switch
#[tauri::command]
pub fn import_mcp_from_apps(
    app_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<ImportMcpResult, AppError> {
    use crate::config::claude_code_manager::ClaudeCodeConfigManager;
    use crate::config::codex_manager::CodexConfigManager;
    use crate::config::gemini_manager::GeminiConfigManager;
    use crate::config::cursor_manager::CursorConfigManager;
    
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let mut result = ImportMcpResult {
        imported: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };
    
    // 获取当前已有的 MCP 名称
    let existing_config = manager.mcp().read_config()?;
    let existing_names: std::collections::HashSet<String> = existing_config.servers.keys().cloned().collect();
    
    // 根据应用名称获取 MCP 配置
    let servers_to_import: Vec<(String, McpServer)> = match app_name.as_str() {
        "Claude Code" => {
            if let Ok(claude_manager) = ClaudeCodeConfigManager::new() {
                if let Ok(servers) = claude_manager.get_mcp_servers() {
                    servers.into_iter().map(|(name, srv)| {
                        // Claude Code 格式: command 是单独的字符串, args 是数组
                        let command = if let Some(cmd) = srv.command {
                            let mut full_cmd = vec![cmd];
                            full_cmd.extend(srv.args);
                            Some(full_cmd)
                        } else if !srv.args.is_empty() {
                            Some(srv.args)
                        } else {
                            None
                        };
                        let mcp = McpServer {
                            server_type: if command.is_some() { McpServerType::Local } else { McpServerType::Remote },
                            enabled: true,
                            timeout: None,
                            command,
                            environment: srv.env,
                            url: srv.url,
                            headers: srv.headers,
                            oauth: None,
                            metadata: Default::default(),
                        };
                        (name, mcp)
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        },
        "Codex" => {
            if let Ok(codex_manager) = CodexConfigManager::new() {
                if let Ok(servers) = codex_manager.get_mcp_servers() {
                    servers.into_iter().map(|(name, srv)| {
                        // Codex 格式: command 已经是 Vec<String>，没有 url
                        let command = if !srv.command.is_empty() { Some(srv.command) } else { None };
                        let mcp = McpServer {
                            server_type: McpServerType::Local, // Codex 只有本地命令
                            enabled: true,
                            timeout: None,
                            command,
                            environment: srv.env,
                            url: None,
                            headers: HashMap::new(),
                            oauth: None,
                            metadata: Default::default(),
                        };
                        (name, mcp)
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        },
        "Gemini CLI" => {
            if let Ok(gemini_manager) = GeminiConfigManager::new() {
                if let Ok(servers) = gemini_manager.get_mcp_servers() {
                    servers.into_iter().map(|(name, srv)| {
                        // Gemini 格式: command 是单独的字符串, args 是数组
                        let command = if let Some(cmd) = srv.command {
                            let mut full_cmd = vec![cmd];
                            full_cmd.extend(srv.args);
                            Some(full_cmd)
                        } else if !srv.args.is_empty() {
                            Some(srv.args)
                        } else {
                            None
                        };
                        let mcp = McpServer {
                            server_type: if command.is_some() { McpServerType::Local } else { McpServerType::Remote },
                            enabled: true,
                            timeout: None,
                            command,
                            environment: srv.env,
                            url: srv.url,
                            headers: HashMap::new(),
                            oauth: None,
                            metadata: Default::default(),
                        };
                        (name, mcp)
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        },
        "Cursor" => {
            if let Ok(cursor_manager) = CursorConfigManager::new() {
                if let Ok(servers) = cursor_manager.get_mcp_servers() {
                    servers.into_iter().map(|(name, srv)| {
                        // Cursor 格式: command 是单独的字符串, args 是数组
                        let command = if let Some(cmd) = srv.command {
                            let mut full_cmd = vec![cmd];
                            full_cmd.extend(srv.args);
                            Some(full_cmd)
                        } else if !srv.args.is_empty() {
                            Some(srv.args)
                        } else {
                            None
                        };
                        let mcp = McpServer {
                            server_type: if command.is_some() { McpServerType::Local } else { McpServerType::Remote },
                            enabled: true,
                            timeout: None,
                            command,
                            environment: srv.env,
                            url: srv.url,
                            headers: HashMap::new(),
                            oauth: None,
                            metadata: Default::default(),
                        };
                        (name, mcp)
                    }).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        },
        _ => {
            return Err(AppError::Custom(format!("不支持的应用: {}", app_name)));
        }
    };
    
    // 导入 MCP
    for (name, server) in servers_to_import {
        if existing_names.contains(&name) {
            result.skipped.push(name);
        } else {
            match manager.mcp_mut().save_server(&name, server) {
                Ok(_) => result.imported.push(name),
                Err(e) => result.failed.push(format!("{}: {}", name, e)),
            }
        }
    }
    
    // 同步到 OpenCode
    if !result.imported.is_empty() {
        let _ = manager.mcp().sync_to_opencode(None);
    }
    
    Ok(result)
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

// ============================================================================
// MCP 多应用统一管理
// ============================================================================

/// 获取所有管理的 MCP（聚合各应用的状态）
#[tauri::command]
pub fn get_managed_mcps(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<ManagedMcp>, AppError> {
    use crate::config::claude_code_manager::ClaudeCodeConfigManager;
    use crate::config::codex_manager::CodexConfigManager;
    use crate::config::gemini_manager::GeminiConfigManager;
    use crate::config::cursor_manager::CursorConfigManager;
    use std::collections::HashSet;
    
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 收集所有 MCP 名称
    let mut all_mcp_names: HashSet<String> = HashSet::new();
    let mut managed_mcps: HashMap<String, ManagedMcp> = HashMap::new();
    
    // 从 Ai Switch 获取 MCP
    let ai_switch_config = manager.mcp().read_config()?;
    for (name, server) in &ai_switch_config.servers {
        all_mcp_names.insert(name.clone());
        managed_mcps.insert(name.clone(), ManagedMcp {
            name: name.clone(),
            server_type: server.server_type.to_string(),
            command: server.command.clone(),
            url: server.url.clone(),
            package_name: extract_package_name(&server.command),
            opencode_enabled: false,
            claude_enabled: false,
            codex_enabled: false,
            gemini_enabled: false,
            cursor_enabled: false,
        });
    }
    
    // 检查 OpenCode
    if let Ok(opencode_config) = manager.mcp().read_opencode_config() {
        if let Some(mcps) = opencode_config.get("mcpServers").and_then(|m| m.as_object()) {
            for name in mcps.keys() {
                all_mcp_names.insert(name.clone());
                if let Some(mcp) = managed_mcps.get_mut(name) {
                    mcp.opencode_enabled = true;
                }
            }
        }
    }
    
    // 检查 Claude Code
    if let Ok(claude_manager) = ClaudeCodeConfigManager::new() {
        if let Ok(servers) = claude_manager.get_mcp_servers() {
            for name in servers.keys() {
                all_mcp_names.insert(name.clone());
                if let Some(mcp) = managed_mcps.get_mut(name) {
                    mcp.claude_enabled = true;
                }
            }
        }
    }
    
    // 检查 Codex
    if let Ok(codex_manager) = CodexConfigManager::new() {
        if let Ok(servers) = codex_manager.get_mcp_servers() {
            for name in servers.keys() {
                all_mcp_names.insert(name.clone());
                if let Some(mcp) = managed_mcps.get_mut(name) {
                    mcp.codex_enabled = true;
                }
            }
        }
    }
    
    // 检查 Gemini
    if let Ok(gemini_manager) = GeminiConfigManager::new() {
        if let Ok(servers) = gemini_manager.get_mcp_servers() {
            for name in servers.keys() {
                all_mcp_names.insert(name.clone());
                if let Some(mcp) = managed_mcps.get_mut(name) {
                    mcp.gemini_enabled = true;
                }
            }
        }
    }
    
    // 检查 Cursor
    if let Ok(cursor_manager) = CursorConfigManager::new() {
        if let Ok(servers) = cursor_manager.get_mcp_servers() {
            for name in servers.keys() {
                all_mcp_names.insert(name.clone());
                if let Some(mcp) = managed_mcps.get_mut(name) {
                    mcp.cursor_enabled = true;
                }
            }
        }
    }
    
    let mut result: Vec<ManagedMcp> = managed_mcps.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(result)
}

/// 获取 MCP 统计信息
#[tauri::command]
pub fn get_mcp_stats(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<McpStats, AppError> {
    use crate::config::claude_code_manager::ClaudeCodeConfigManager;
    use crate::config::codex_manager::CodexConfigManager;
    use crate::config::gemini_manager::GeminiConfigManager;
    use crate::config::cursor_manager::CursorConfigManager;
    
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut stats = McpStats::default();
    
    // OpenCode
    if let Ok(opencode_config) = manager.mcp().read_opencode_config() {
        if let Some(mcps) = opencode_config.get("mcpServers").and_then(|m| m.as_object()) {
            stats.opencode_count = mcps.len();
        }
    }
    
    // Claude Code
    if let Ok(claude_manager) = ClaudeCodeConfigManager::new() {
        if let Ok(servers) = claude_manager.get_mcp_servers() {
            stats.claude_count = servers.len();
        }
    }
    
    // Codex
    if let Ok(codex_manager) = CodexConfigManager::new() {
        if let Ok(servers) = codex_manager.get_mcp_servers() {
            stats.codex_count = servers.len();
        }
    }
    
    // Gemini
    if let Ok(gemini_manager) = GeminiConfigManager::new() {
        if let Ok(servers) = gemini_manager.get_mcp_servers() {
            stats.gemini_count = servers.len();
        }
    }
    
    // Cursor
    if let Ok(cursor_manager) = CursorConfigManager::new() {
        if let Ok(servers) = cursor_manager.get_mcp_servers() {
            stats.cursor_count = servers.len();
        }
    }
    
    Ok(stats)
}

/// 切换 MCP 在某个应用上的启用状态
#[tauri::command]
pub fn toggle_mcp_app(
    mcp_name: String,
    app: String,
    enabled: bool,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    use crate::config::claude_code_manager::{ClaudeCodeConfigManager, ClaudeMcpServer};
    use crate::config::codex_manager::{CodexConfigManager, CodexMcpServer};
    use crate::config::gemini_manager::{GeminiConfigManager, GeminiMcpServer};
    use crate::config::cursor_manager::{CursorConfigManager, CursorMcpServer};
    
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 获取 Ai Switch 中的 MCP 配置（作为源）
    let ai_switch_config = manager.mcp().read_config()?;
    let source_server = ai_switch_config.servers.get(&mcp_name);
    
    match app.as_str() {
        "opencode" => {
            if enabled {
                // 同步到 OpenCode
                manager.mcp().sync_to_opencode(Some(&[mcp_name]))?;
            } else {
                // 从 OpenCode 删除
                manager.mcp().remove_from_opencode(&mcp_name)?;
            }
        }
        "claude" => {
            let claude_manager = ClaudeCodeConfigManager::new()
                .map_err(|e| AppError::Custom(e))?;
            
            if enabled {
                if let Some(server) = source_server {
                    let claude_server = ClaudeMcpServer {
                        command: server.command.as_ref().and_then(|c| c.first().cloned()),
                        args: server.command.as_ref()
                            .map(|c| c.iter().skip(1).cloned().collect())
                            .unwrap_or_default(),
                        env: server.environment.clone(),
                        url: server.url.clone(),
                        headers: server.headers.clone(),
                    };
                    claude_manager.add_mcp_server(&mcp_name, claude_server)
                        .map_err(|e| AppError::Custom(e))?;
                }
            } else {
                claude_manager.remove_mcp_server(&mcp_name)
                    .map_err(|e| AppError::Custom(e))?;
            }
        }
        "codex" => {
            let codex_manager = CodexConfigManager::new()
                .map_err(|e| AppError::Custom(e))?;
            
            if enabled {
                if let Some(server) = source_server {
                    if let Some(cmd) = &server.command {
                        let codex_server = CodexMcpServer {
                            command: cmd.clone(),
                            env: server.environment.clone(),
                        };
                        codex_manager.add_mcp_server(&mcp_name, codex_server)
                            .map_err(|e| AppError::Custom(e))?;
                    }
                }
            } else {
                codex_manager.remove_mcp_server(&mcp_name)
                    .map_err(|e| AppError::Custom(e))?;
            }
        }
        "gemini" => {
            let gemini_manager = GeminiConfigManager::new()
                .map_err(|e| AppError::Custom(e))?;
            
            if enabled {
                if let Some(server) = source_server {
                    let gemini_server = GeminiMcpServer {
                        command: server.command.as_ref().and_then(|c| c.first().cloned()),
                        args: server.command.as_ref()
                            .map(|c| c.iter().skip(1).cloned().collect())
                            .unwrap_or_default(),
                        env: server.environment.clone(),
                        url: server.url.clone(),
                    };
                    gemini_manager.add_mcp_server(&mcp_name, gemini_server)
                        .map_err(|e| AppError::Custom(e))?;
                }
            } else {
                gemini_manager.remove_mcp_server(&mcp_name)
                    .map_err(|e| AppError::Custom(e))?;
            }
        }
        "cursor" => {
            let cursor_manager = CursorConfigManager::new()
                .map_err(|e| AppError::Custom(e))?;
            
            if enabled {
                if let Some(server) = source_server {
                    let cursor_server = CursorMcpServer {
                        command: server.command.as_ref().and_then(|c| c.first().cloned()),
                        args: server.command.as_ref()
                            .map(|c| c.iter().skip(1).cloned().collect())
                            .unwrap_or_default(),
                        env: server.environment.clone(),
                        url: server.url.clone(),
                    };
                    cursor_manager.add_mcp_server(&mcp_name, cursor_server)
                        .map_err(|e| AppError::Custom(e))?;
                }
            } else {
                cursor_manager.remove_mcp_server(&mcp_name)
                    .map_err(|e| AppError::Custom(e))?;
            }
        }
        _ => {
            return Err(AppError::Custom(format!("不支持的应用: {}", app)));
        }
    }
    
    Ok(())
}

/// 从所有应用中删除 MCP
#[tauri::command]
pub fn delete_mcp_from_all(
    mcp_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    use crate::config::claude_code_manager::ClaudeCodeConfigManager;
    use crate::config::codex_manager::CodexConfigManager;
    use crate::config::gemini_manager::GeminiConfigManager;
    use crate::config::cursor_manager::CursorConfigManager;
    
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 从 Ai Switch 删除
    let _ = manager.mcp_mut().delete_server(&mcp_name);
    
    // 从 OpenCode 删除
    let _ = manager.mcp().remove_from_opencode(&mcp_name);
    
    // 从 Claude Code 删除
    if let Ok(claude_manager) = ClaudeCodeConfigManager::new() {
        let _ = claude_manager.remove_mcp_server(&mcp_name);
    }
    
    // 从 Codex 删除
    if let Ok(codex_manager) = CodexConfigManager::new() {
        let _ = codex_manager.remove_mcp_server(&mcp_name);
    }
    
    // 从 Gemini 删除
    if let Ok(gemini_manager) = GeminiConfigManager::new() {
        let _ = gemini_manager.remove_mcp_server(&mcp_name);
    }
    
    // 从 Cursor 删除
    if let Ok(cursor_manager) = CursorConfigManager::new() {
        let _ = cursor_manager.remove_mcp_server(&mcp_name);
    }
    
    Ok(())
}
