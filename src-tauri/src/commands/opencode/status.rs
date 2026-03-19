// 状态相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Mutex;
use tauri::State;

use crate::modules::opencode_config::ConfigManager;
use crate::opencode_error::AppError;

// Windows 平台：隐藏命令行窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 应用状态信息
#[derive(Debug, Serialize)]
pub struct AppStatus {
    pub has_global_config: bool,
    pub has_project_config: bool,
    pub active_provider: Option<String>,
    pub provider_count: usize,
    pub mcp_server_count: usize,
    pub config_paths: ConfigPaths,
}

/// 配置文件路径
#[derive(Debug, Serialize)]
pub struct ConfigPaths {
    pub global_config_dir: String,
    pub global_opencode_dir: String,
    pub project_opencode_dir: Option<String>,
}

/// 获取应用状态
#[tauri::command]
pub fn get_status(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<AppStatus, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 获取 provider 数量
    let provider_count = manager
        .opencode()
        .get_all_providers()
        .map(|p| p.len())
        .unwrap_or(0);
    
    // 获取 MCP 服务器数量
    let mcp_server_count = manager
        .mcp()
        .get_sorted_server_names()
        .map(|s| s.len())
        .unwrap_or(0);
    
    // 获取激活的配置
    let active_provider = manager
        .get_active_opencode_config()
        .ok()
        .flatten()
        .map(|c| c.provider);
    
    // 获取配置路径
    let home_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let current_dir = std::env::current_dir()
        .ok()
        .map(|p| p.join(".opencode").to_string_lossy().to_string());
    
    let has_project_config = current_dir
        .as_ref()
        .map(|p| std::path::Path::new(p).join("opencode.json").exists())
        .unwrap_or(false);
    
    Ok(AppStatus {
        has_global_config: true, // 总是有全局配置目录
        has_project_config,
        active_provider,
        provider_count,
        mcp_server_count,
        config_paths: ConfigPaths {
            global_config_dir: format!("{}/.Ai Switch", home_dir),
            global_opencode_dir: format!("{}/.opencode", home_dir),
            project_opencode_dir: current_dir,
        },
    })
}

/// 获取版本信息
/// 注意：使用 tauri.conf.json 中的版本号，这与 updater 使用的版本一致
#[tauri::command]
pub fn get_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

/// 获取公网 IP 地址
#[tauri::command]
pub async fn get_local_ip() -> String {
    // 通过外部 API 获取公网 IP
    let apis = [
        "https://api.ipify.org",
        "https://ipinfo.io/ip",
        "https://api.ip.sb/ip",
    ];
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    
    for api in apis {
        if let Ok(resp) = client.get(api).send().await {
            if let Ok(ip) = resp.text().await {
                let ip = ip.trim().to_string();
                if !ip.is_empty() && ip.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    return ip;
                }
            }
        }
    }
    
    "---.---.---.---".to_string()
}

// ========== CLI 工具版本检测与更新 ==========

/// CLI 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolInfo {
    /// 工具标识，如 "claude", "codex", "gemini"
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 是否已安装
    pub installed: bool,
    /// 当前版本
    pub current_version: Option<String>,
    /// 最新版本（可选，需额外查询）
    pub latest_version: Option<String>,
    /// 是否有可用更新
    pub has_update: bool,
    /// npm 包名（用于安装/更新）
    pub npm_package: String,
    /// 说明
    pub description: String,
}

/// 执行命令并返回 stdout（隐藏窗口）
fn cli_run_cmd(program: &str, args: &[&str]) -> Option<String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", program])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !stdout.is_empty() {
                return Some(stdout);
            }
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            if stderr.is_empty() { None } else { Some(stderr) }
        }
        _ => None,
    }
}

/// 从原始版本字符串中提取纯版本号
fn parse_cli_version(raw: &str) -> String {
    // 处理各种格式：
    // claude: "1.0.15" 或 "claude v1.0.15"
    // codex:  "0.1.2025011400" 或 "codex/0.1.xxx"
    // gemini: "@google/gemini-cli/0.1.32 darwin-arm64 node-v22.15.0"
    let s = raw.trim();
    
    // 尝试匹配 x.y.z 格式的版本号
    for part in s.split_whitespace() {
        let cleaned = part.trim_start_matches('v')
            .trim_start_matches("claude/")
            .trim_start_matches("codex/")
            .trim_start_matches("@google/gemini-cli/")
            .trim_start_matches("@anthropic-ai/claude-code/");
        // 检查是否是版本号格式
        if cleaned.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return cleaned.to_string();
        }
    }
    s.to_string()
}

/// 检测所有 CLI 工具版本
#[tauri::command]
pub async fn detect_cli_tools() -> Result<Vec<CliToolInfo>, String> {
    let tools = vec![
        ("claude", "Claude Code", "@anthropic-ai/claude-code", "Anthropic Claude Code CLI"),
        ("codex", "Codex CLI", "@openai/codex", "OpenAI Codex CLI"),
        ("gemini", "Gemini CLI", "@google/gemini-cli", "Google Gemini CLI"),
    ];

    let mut results = Vec::new();
    for (cmd, name, npm_pkg, desc) in tools {
        let version_output = cli_run_cmd(cmd, &["--version"]);
        let installed = version_output.is_some();
        let current_version = version_output.map(|v| parse_cli_version(&v));

        results.push(CliToolInfo {
            id: cmd.to_string(),
            name: name.to_string(),
            installed,
            current_version,
            latest_version: None,
            has_update: false,
            npm_package: npm_pkg.to_string(),
            description: desc.to_string(),
        });
    }
    Ok(results)
}

/// 查询单个 CLI 工具的最新版本（npm view）
#[tauri::command]
pub async fn check_cli_latest_version(npm_package: String) -> Result<String, String> {
    cli_run_cmd("npm", &["view", &npm_package, "version"])
        .map(|v| v.trim().to_string())
        .ok_or_else(|| format!("无法获取 {} 的最新版本，请确认 npm 可用", npm_package))
}

/// 更新指定 CLI 工具（npm install -g <package>@latest）
#[tauri::command]
pub async fn update_cli_tool(npm_package: String) -> Result<String, String> {
    let pkg_latest = format!("{}@latest", npm_package);

    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", "npm", "install", "-g", &pkg_latest])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("npm")
        .args(["install", "-g", &pkg_latest])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(format!("{}{}", stdout.trim(), if stderr.trim().is_empty() { String::new() } else { format!("\n{}", stderr.trim()) }))
            } else {
                Err(format!("更新失败: {}{}", stdout.trim(), stderr.trim()))
            }
        }
        Err(e) => Err(format!("执行 npm 命令失败: {}。请确认已安装 Node.js 和 npm", e)),
    }
}
