// Provider 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::config::ConfigManager;
use crate::error::AppError;

/// Provider 列表项（传递给前端）
#[derive(Debug, Clone, Serialize)]
pub struct ProviderItem {
    pub name: String,
    pub base_url: String,
    pub model_count: usize,
    pub description: Option<String>,
    pub model_type: String,
    pub enabled: bool,
}

/// Provider 详情（传递给前端）
#[derive(Debug, Clone, Serialize)]
pub struct ProviderDetail {
    pub name: String,
    pub npm: Option<String>,
    pub model_type: Option<String>,
    pub options: ProviderOptions,
    pub models: std::collections::HashMap<String, crate::config::OpenCodeModelInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderOptions {
    pub base_url: String,
    pub api_key: String,
}

/// 添加/编辑 Provider 的参数
#[derive(Debug, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub npm: Option<String>,
    pub description: Option<String>,
    pub model_type: Option<String>,
    #[serde(default = "default_auto_add_v1_suffix")]
    pub auto_add_v1_suffix: bool,
}

fn default_auto_add_v1_suffix() -> bool {
    true
}

/// 应用配置的参数
#[derive(Debug, Deserialize)]
pub struct ApplyConfigInput {
    pub provider_names: Vec<String>,
    pub apply_to_global: bool,
    pub apply_to_project: bool,
}

/// 获取所有 Provider 列表
#[tauri::command]
pub fn get_providers(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<ProviderItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let providers = manager.opencode().get_all_providers()?;
    
    let mut items: Vec<ProviderItem> = providers
        .iter()
        .map(|(name, provider)| ProviderItem {
            name: name.clone(),
            base_url: provider.options.base_url.clone(),
            model_count: provider.models.len(),
            description: provider.metadata.description.clone(),
            // 从顶级字段读取 model_type
            model_type: provider.model_type.clone().unwrap_or_else(|| "claude".to_string()),
            enabled: provider.enabled,
        })
        .collect();
    
    // 按名称排序
    items.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(items)
}

/// 获取单个 Provider 详情
#[tauri::command]
pub fn get_provider(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Option<ProviderDetail>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let provider = manager.opencode().get_provider(&name)?;
    
    Ok(provider.map(|p| ProviderDetail {
        name: p.name.clone(),
        npm: p.npm.clone(),
        model_type: p.model_type.clone(),
        options: ProviderOptions {
            base_url: p.options.base_url.clone(),
            api_key: p.options.api_key.clone(),
        },
        models: p.models.clone(),
        description: p.metadata.description.clone(),
    }))
}

/// 添加新 Provider
#[tauri::command]
pub fn add_provider(
    input: ProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().add_provider(
        input.name,
        input.base_url,
        input.api_key,
        input.npm,
        input.description,
        input.model_type,
        input.auto_add_v1_suffix,
    )?;
    Ok(())
}

/// 更新 Provider
#[tauri::command]
pub fn update_provider(
    name: String,
    input: ProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().update_provider_metadata(
        &name,
        Some(input.base_url),
        Some(input.api_key),
        input.npm,
        input.description,
        input.model_type,
    )?;
    Ok(())
}

/// 删除 Provider
#[tauri::command]
pub fn delete_provider(
    name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().delete_provider(&name)?;
    Ok(())
}

/// 检查 Provider 是否已应用到全局/项目配置
#[derive(Debug, Serialize)]
pub struct AppliedStatus {
    pub in_global: bool,
    pub in_project: bool,
}

#[tauri::command]
pub fn check_provider_applied(
    provider_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<AppliedStatus, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let status = manager.check_provider_applied(&provider_name)?;
    Ok(AppliedStatus {
        in_global: status.0,
        in_project: status.1,
    })
}

/// 应用配置到全局/项目
#[tauri::command]
pub fn apply_config(
    input: ApplyConfigInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    if input.apply_to_global {
        manager.apply_multiple_opencode_to_global(&input.provider_names)?;
    }
    
    if input.apply_to_project {
        manager.apply_multiple_opencode_to_project(&input.provider_names)?;
    }
    
    Ok(())
}

/// 切换 Provider 启用状态
#[tauri::command]
pub fn toggle_provider(
    name: String,
    enabled: bool,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().toggle_provider(&name, enabled)?;
    Ok(())
}
