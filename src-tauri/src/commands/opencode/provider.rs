// Provider 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::modules::opencode_config::ConfigManager;
use crate::opencode_error::AppError;

/// Base URL 配置（传递给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseUrlItem {
    pub url: String,
    pub latency_ms: Option<u64>,
    pub last_tested: Option<String>,
    pub quality: String,
}

/// Provider 列表项（传递给前端）
#[derive(Debug, Clone, Serialize)]
pub struct ProviderItem {
    pub name: String,
    pub base_url: String,           // 当前激活的 URL（向后兼容）
    pub base_urls: Vec<BaseUrlItem>, // 所有 URL 列表
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
    pub models: std::collections::HashMap<String, crate::modules::opencode_config::OpenCodeModelInfo>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderOptions {
    pub base_url: String,           // 当前激活的 URL
    pub base_urls: Vec<BaseUrlItem>, // 所有 URL 列表
    pub api_key: String,
}

/// 添加/编辑 Provider 的参数
#[derive(Debug, Deserialize)]
pub struct ProviderInput {
    pub name: String,
    pub api_key: String,
    pub base_url: String,           // 激活的 URL
    #[serde(default)]
    pub base_urls: Vec<String>,     // 所有 URL 列表（可选，为空时使用 base_url）
    pub npm: Option<String>,
    pub description: Option<String>,
    pub model_type: Option<String>,
    #[serde(default = "default_auto_add_v1_suffix")]
    pub auto_add_v1_suffix: bool,
}

fn default_auto_add_v1_suffix() -> bool {
    true
}

/// 添加 Base URL 的参数
#[derive(Debug, Deserialize)]
pub struct AddBaseUrlInput {
    pub provider_name: String,
    pub url: String,
}

/// 删除 Base URL 的参数
#[derive(Debug, Deserialize)]
pub struct RemoveBaseUrlInput {
    pub provider_name: String,
    pub url: String,
}

/// 设置激活 Base URL 的参数
#[derive(Debug, Deserialize)]
pub struct SetActiveBaseUrlInput {
    pub provider_name: String,
    pub url: String,
}

/// 更新 URL 延迟的参数
#[derive(Debug, Deserialize)]
pub struct UpdateUrlLatencyInput {
    pub provider_name: String,
    pub url: String,
    pub latency_ms: Option<u64>,
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
        .map(|(name, provider)| {
            // 转换 base_urls 到前端格式
            let base_urls: Vec<BaseUrlItem> = provider.options.base_urls
                .iter()
                .map(|u| BaseUrlItem {
                    url: u.url.clone(),
                    latency_ms: u.latency_ms,
                    last_tested: u.last_tested.clone(),
                    quality: u.get_quality().to_string(),
                })
                .collect();
            
            // 如果 base_urls 为空，使用 base_url 创建一个
            let base_urls = if base_urls.is_empty() {
                vec![BaseUrlItem {
                    url: provider.options.base_url.clone(),
                    latency_ms: None,
                    last_tested: None,
                    quality: "untested".to_string(),
                }]
            } else {
                base_urls
            };
            
            ProviderItem {
                name: name.clone(),
                base_url: provider.options.base_url.clone(),
                base_urls,
                model_count: provider.models.len(),
                description: provider.metadata.description.clone(),
                model_type: provider.model_type.clone().unwrap_or_else(|| "claude".to_string()),
                enabled: provider.enabled,
            }
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
    
    Ok(provider.map(|p| {
        // 转换 base_urls 到前端格式
        let base_urls: Vec<BaseUrlItem> = p.options.base_urls
            .iter()
            .map(|u| BaseUrlItem {
                url: u.url.clone(),
                latency_ms: u.latency_ms,
                last_tested: u.last_tested.clone(),
                quality: u.get_quality().to_string(),
            })
            .collect();
        
        // 如果 base_urls 为空，使用 base_url 创建一个
        let base_urls = if base_urls.is_empty() {
            vec![BaseUrlItem {
                url: p.options.base_url.clone(),
                latency_ms: None,
                last_tested: None,
                quality: "untested".to_string(),
            }]
        } else {
            base_urls
        };
        
        ProviderDetail {
            name: p.name.clone(),
            npm: p.npm.clone(),
            model_type: p.model_type.clone(),
            options: ProviderOptions {
                base_url: p.options.base_url.clone(),
                base_urls,
                api_key: p.options.api_key.clone(),
            },
            models: p.models.clone(),
            description: p.metadata.description.clone(),
        }
    }))
}

/// Provider 应用信息（用于应用到 CLI 工具）
#[derive(Debug, Clone, Serialize)]
pub struct ProviderApplyInfo {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model_type: String,
}

/// 获取 Provider 信息用于应用到其他 CLI 工具
#[tauri::command]
pub fn get_provider_for_apply(
    provider_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<ProviderApplyInfo, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let provider = manager.opencode().get_provider(&provider_name)?
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' not found", provider_name)))?;
    
    Ok(ProviderApplyInfo {
        name: provider.name.clone(),
        api_key: provider.options.api_key.clone(),
        base_url: provider.options.base_url.clone(),
        model_type: provider.model_type.clone().unwrap_or_else(|| "claude".to_string()),
    })
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
fn infer_model_type(provider_name: &str, models: &std::collections::HashMap<String, crate::modules::opencode_config::OpenCodeModelInfo>) -> Option<String> {
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

/// 导入已部署的服务商到 Ai Switch 管理界面
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

// ============================================================================
// 多 Base URL 管理命令
// ============================================================================

/// 添加 Base URL 到 Provider
#[tauri::command]
pub fn add_provider_base_url(
    input: AddBaseUrlInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 获取 provider
    let provider = config.get_provider_mut(&input.provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", input.provider_name)))?;
    
    // 添加 URL
    provider.add_base_url(input.url);
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(())
}

/// 从 Provider 删除 Base URL
#[tauri::command]
pub fn remove_provider_base_url(
    input: RemoveBaseUrlInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 获取 provider
    let provider = config.get_provider_mut(&input.provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", input.provider_name)))?;
    
    // 确保至少保留一个 URL
    if provider.options.base_urls.len() <= 1 {
        return Err(AppError::Custom("至少需要保留一个 Base URL".to_string()));
    }
    
    // 删除 URL
    if !provider.remove_base_url(&input.url) {
        return Err(AppError::Custom(format!("URL '{}' 不存在", input.url)));
    }
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(())
}

/// 设置 Provider 的激活 Base URL
#[tauri::command]
pub fn set_active_base_url(
    input: SetActiveBaseUrlInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 获取 provider
    let provider = config.get_provider_mut(&input.provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", input.provider_name)))?;
    
    // 设置激活 URL
    if !provider.set_active_base_url(&input.url) {
        return Err(AppError::Custom(format!("URL '{}' 不在 Provider 的 URL 列表中", input.url)));
    }
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(())
}

/// 更新 URL 的延迟测试结果
#[tauri::command]
pub fn update_url_latency(
    input: UpdateUrlLatencyInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 获取 provider
    let provider = config.get_provider_mut(&input.provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", input.provider_name)))?;
    
    // 更新延迟
    provider.update_url_latency(&input.url, input.latency_ms);
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(())
}

/// 自动选择最快的 Base URL
#[tauri::command]
pub fn auto_select_fastest_base_url(
    provider_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<String, AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 读取当前配置
    let mut config = manager.opencode().read_config()?;
    
    // 获取 provider
    let provider = config.get_provider_mut(&provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", provider_name)))?;
    
    // 自动选择最快的 URL
    if !provider.auto_select_fastest_url() {
        return Err(AppError::Custom("没有可用的延迟测试结果，无法自动选择".to_string()));
    }
    
    let selected_url = provider.get_active_base_url().to_string();
    
    // 写回配置文件
    manager.opencode_mut().write_config(&config)?;
    
    Ok(selected_url)
}
