// 状态相关的 Tauri commands

use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

use crate::config::ConfigManager;
use crate::error::AppError;

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
            global_config_dir: format!("{}/.Open Switch", home_dir),
            global_opencode_dir: format!("{}/.opencode", home_dir),
            project_opencode_dir: current_dir,
        },
    })
}

/// 获取版本信息
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
