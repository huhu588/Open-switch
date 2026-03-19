// Augment Code 相关 Tauri 命令

use crate::modules::opencode_config::augment_manager::AugmentConfigManager;
use serde::Serialize;

/// Augment Code 配置状态
#[derive(Debug, Serialize)]
pub struct AugmentStatus {
    pub is_installed: bool,
    pub extension_version: Option<String>,
    pub rules_count: usize,
    pub rules_dir: String,
}

/// 获取 Augment Code 配置状态
#[tauri::command]
pub async fn get_augment_status() -> Result<AugmentStatus, String> {
    let manager = AugmentConfigManager::new()?;

    Ok(AugmentStatus {
        is_installed: manager.is_installed(),
        extension_version: manager.get_extension_version(),
        rules_count: manager.get_rules_count(),
        rules_dir: manager.get_rules_dir().to_string_lossy().to_string(),
    })
}

/// 确保 Augment Code rules 目录存在
#[tauri::command]
pub async fn ensure_augment_rules_dir() -> Result<String, String> {
    let manager = AugmentConfigManager::new()?;
    manager.ensure_rules_dir()?;
    Ok(manager.get_rules_dir().to_string_lossy().to_string())
}
