// Model 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::config::{ConfigManager, OpenCodeModelInfo, OpenCodeModelLimit, Detector};
use crate::error::AppError;

/// Model 列表项
#[derive(Debug, Clone, Serialize)]
pub struct ModelItem {
    pub id: String,
    pub name: String,
    pub context_limit: Option<u64>,
    pub output_limit: Option<u64>,
}

/// 添加 Model 的参数
#[derive(Debug, Deserialize)]
pub struct ModelInput {
    pub id: String,
    pub name: Option<String>,
    pub context_limit: Option<u64>,
    pub output_limit: Option<u64>,
}

/// 获取 Provider 下的所有 Model
#[tauri::command]
pub fn get_models(
    provider_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<ModelItem>, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let models = manager.opencode().get_models(&provider_name)?;
    
    let mut items: Vec<ModelItem> = models
        .iter()
        .map(|(id, info)| ModelItem {
            id: id.clone(),
            name: info.name.clone(),
            context_limit: info.limit.as_ref().and_then(|l| l.context),
            output_limit: info.limit.as_ref().and_then(|l| l.output),
        })
        .collect();
    
    items.sort_by(|a, b| a.id.cmp(&b.id));
    
    Ok(items)
}

/// 添加 Model
#[tauri::command]
pub fn add_model(
    provider_name: String,
    input: ModelInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    let limit = if input.context_limit.is_some() || input.output_limit.is_some() {
        Some(OpenCodeModelLimit {
            context: input.context_limit,
            output: input.output_limit,
        })
    } else {
        None
    };
    
    let model_info = OpenCodeModelInfo {
        id: input.id.clone(),
        name: input.name.unwrap_or_else(|| input.id.clone()),
        limit,
        model_detection: None,
    };
    
    manager.opencode_mut().add_model(&provider_name, input.id, model_info)?;
    
    Ok(())
}

/// 删除 Model
#[tauri::command]
pub fn delete_model(
    provider_name: String,
    model_id: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    manager.opencode_mut().delete_model(&provider_name, &model_id)?;
    Ok(())
}

/// Claude 预设模型列表 (Anthropic 协议使用)
const CLAUDE_PRESET_MODELS: &[&str] = &[
    "claude-4-sonnet",
    "claude-4.1-opus",
    "claude-4.5-haiku",
    "claude-4.5-opus",
    "claude-4.5-sonnet",
];

/// 检测是否为 Anthropic 协议 URL
fn is_anthropic_protocol(base_url: &str) -> bool {
    let url_lower = base_url.to_lowercase();
    // 检测 URL 中是否包含 anthropic 关键字
    url_lower.contains("anthropic") || 
    url_lower.contains("api.anthropic.com") ||
    // 常见的 Anthropic 协议中转服务
    url_lower.contains("packyapi.com") ||
    url_lower.contains("cubence.com") ||
    url_lower.contains("aigocode.com")
}

/// 从站点获取可用模型列表
/// - Anthropic 协议: 返回预设的 Claude 模型列表
/// - OpenAI 协议: 调用 /v1/models API 获取
#[tauri::command]
pub async fn fetch_site_models(
    provider_name: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<Vec<String>, AppError> {
    // 获取 provider 信息
    let (base_url, api_key) = {
        let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let provider = manager
            .opencode()
            .get_provider(&provider_name)?
            .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", provider_name)))?;
        (provider.options.base_url.clone(), provider.options.api_key.clone())
    };
    
    // 检测是否为 Anthropic 协议
    if is_anthropic_protocol(&base_url) {
        // Anthropic 协议: 直接返回预设的 Claude 模型列表
        return Ok(CLAUDE_PRESET_MODELS.iter().map(|s| s.to_string()).collect());
    }
    
    // OpenAI 协议: 调用检测器获取模型列表
    let detector = Detector::new();
    let result = detector.detect_site(&base_url, &api_key).await;
    
    if result.is_available {
        Ok(result.available_models)
    } else {
        Err(AppError::Custom(
            result.error_message.unwrap_or_else(|| "获取模型列表失败".to_string())
        ))
    }
}

/// 批量添加 Model
#[tauri::command]
pub fn add_models_batch(
    provider_name: String,
    model_ids: Vec<String>,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    for model_id in model_ids {
        let model_info = OpenCodeModelInfo {
            id: model_id.clone(),
            name: model_id.clone(),
            limit: None,
            model_detection: None,
        };
        
        // 忽略已存在的模型
        let _ = manager.opencode_mut().add_model(&provider_name, model_id, model_info);
    }
    
    Ok(())
}
