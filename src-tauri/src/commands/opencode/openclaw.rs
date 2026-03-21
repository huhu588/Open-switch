use crate::modules::opencode_config::openclaw_manager::{
    OpenClawConfigManager, OpenClawProvider, OpenClawStatus,
};

#[tauri::command]
pub fn get_openclaw_status() -> Result<OpenClawStatus, String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    Ok(manager.get_status())
}

#[tauri::command]
pub fn get_openclaw_config_path() -> Result<String, String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    if !manager.is_installed() {
        return Err("OpenClaw 未安装".to_string());
    }
    Ok(manager.get_config_path())
}

#[tauri::command]
pub fn get_openclaw_agents_content() -> Result<String, String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    manager.get_agents_content()
}

#[tauri::command]
pub fn save_openclaw_agents_content(content: String) -> Result<(), String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    manager.save_agents_content(&content)
}

#[tauri::command]
pub fn get_openclaw_soul_content() -> Result<String, String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    manager.get_soul_content()
}

#[tauri::command]
pub fn save_openclaw_soul_content(content: String) -> Result<(), String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    manager.save_soul_content(&content)
}

#[tauri::command]
pub async fn apply_provider_to_openclaw(provider: OpenClawProvider) -> Result<(), String> {
    let manager =
        OpenClawConfigManager::new().map_err(|e| format!("初始化 OpenClaw 管理器失败: {}", e))?;
    manager.apply_provider(&provider)
}

#[tauri::command]
pub fn get_claude_config_path() -> Result<String, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let claude_dir = home.join(".claude");
    if claude_dir.exists() {
        Ok(claude_dir.to_string_lossy().to_string())
    } else {
        Err("Claude Code 配置目录不存在".to_string())
    }
}
