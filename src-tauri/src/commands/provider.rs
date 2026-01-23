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

/// 已部署的 Provider 列表项（传递给前端）
#[derive(Debug, Clone, Serialize)]
pub struct DeployedProviderItem {
    pub name: String,
    pub base_url: String,
    pub model_count: usize,
    pub source: String, // "global" 或 "project"
    pub inferred_model_type: Option<String>, // 推断的模型类型
}

/// 删除已部署 Provider 的参数
#[derive(Debug, Deserialize)]
pub struct RemoveDeployedProviderInput {
    pub name: String,
    pub from_global: bool,
    pub from_project: bool,
}

/// 获取已部署到 opencode 的 Provider 列表
#[tauri::command]
pub fn get_deployed_providers(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<DeployedProviderItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let providers = manager.opencode().get_deployed_providers()?;
    let config = manager.opencode().read_config()?;
    
    let items: Vec<DeployedProviderItem> = providers
        .into_iter()
        .map(|p| {
            // 尝试从 opencode 配置中获取 provider 信息，用于推断 model_type
            let inferred_model_type = if let Some(provider) = config.get_provider(&p.name) {
                // 如果已经有 model_type，直接使用
                if provider.model_type.is_some() {
                    provider.model_type.clone()
                } else {
                    // 否则尝试自动推断
                    infer_model_type(&p.name, &provider.models)
                }
            } else {
                None
            };
            
            DeployedProviderItem {
                name: p.name,
                base_url: p.base_url,
                model_count: p.model_count,
                source: p.source,
                inferred_model_type,
            }
        })
        .collect();
    
    Ok(items)
}

/// 从已部署的 opencode 配置中删除 Provider
#[tauri::command]
pub fn remove_deployed_provider(
    input: RemoveDeployedProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode().remove_deployed_provider(&input.name, input.from_global, input.from_project)?;
    Ok(())
}

/// 导入已部署服务商的参数
#[derive(Debug, Deserialize)]
pub struct ImportDeployedProviderInput {
    pub name: String,
    pub model_type: String, // 用户选择的模型类型：claude/codex/gemini
}

/// 根据服务商名称和模型列表自动推断 model_type
fn infer_model_type(provider_name: &str, models: &std::collections::HashMap<String, crate::config::OpenCodeModelInfo>) -> Option<String> {
    // 优先根据 provider 名称推断（不区分大小写）
    let name_lower = provider_name.to_lowercase();
    
    // Claude 相关关键词
    if name_lower.contains("claude") || name_lower.contains("anthropic") {
        return Some("claude".to_string());
    }
    
    // GPT/Code 相关关键词
    if name_lower.contains("gpt") || name_lower.contains("openai") 
        || name_lower.contains("code") || name_lower.contains("codex") {
        return Some("codex".to_string());
    }
    
    // Gemini 相关关键词
    if name_lower.contains("gemini") || name_lower.contains("google") {
        return Some("gemini".to_string());
    }
    
    // 如果名称无法推断，则检查模型列表
    let mut claude_count = 0;
    let mut codex_count = 0;
    let mut gemini_count = 0;
    
    for model_id in models.keys() {
        let model_lower = model_id.to_lowercase();
        
        if model_lower.contains("claude") {
            claude_count += 1;
        } else if model_lower.contains("gpt") || model_lower.contains("code") {
            codex_count += 1;
        } else if model_lower.contains("gemini") {
            gemini_count += 1;
        }
    }
    
    // 返回占比最高的类型
    if claude_count > 0 && claude_count >= codex_count && claude_count >= gemini_count {
        return Some("claude".to_string());
    } else if codex_count > 0 && codex_count >= gemini_count {
        return Some("codex".to_string());
    } else if gemini_count > 0 {
        return Some("gemini".to_string());
    }
    
    // 无法推断
    None
}

/// 导入已部署的服务商到 Open Switch 管理界面
/// 实际上是确保 provider 包含 model_type 字段
#[tauri::command]
pub fn import_deployed_provider(
    input: ImportDeployedProviderInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 检查 provider 是否存在
    let provider = config.get_provider_mut(&input.name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", input.name)))?;
    
    // 设置 model_type 字段
    provider.model_type = Some(input.model_type);
    provider.update_timestamp();
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(())
}
