// Warp 相关 Tauri 命令

use crate::modules::opencode_config::warp_manager::{WarpConfigManager, WarpUsageStats};
use serde::Serialize;

/// Warp 配置状态
#[derive(Debug, Serialize)]
pub struct WarpStatus {
    pub is_installed: bool,
    pub is_db_accessible: bool,
    pub config_dir: String,
    pub db_path: String,
    pub rules_dir: String,
}

/// 获取 Warp 配置状态
#[tauri::command]
pub async fn get_warp_status() -> Result<WarpStatus, String> {
    let manager = WarpConfigManager::new()?;

    Ok(WarpStatus {
        is_installed: manager.is_installed(),
        is_db_accessible: manager.is_db_accessible(),
        config_dir: manager.get_config_dir().to_string_lossy().to_string(),
        db_path: manager.get_db_path().to_string_lossy().to_string(),
        rules_dir: manager.get_rules_dir().to_string_lossy().to_string(),
    })
}

/// 从 Warp 本地数据库获取用量统计
#[tauri::command]
pub async fn get_warp_usage_from_local_db() -> Result<WarpUsageStats, String> {
    let manager = WarpConfigManager::new()?;
    manager.get_usage_stats()
}

/// 确保 Warp rules 目录存在
#[tauri::command]
pub async fn ensure_warp_rules_dir() -> Result<String, String> {
    let manager = WarpConfigManager::new()?;
    manager.ensure_rules_dir()?;
    Ok(manager.get_rules_dir().to_string_lossy().to_string())
}
