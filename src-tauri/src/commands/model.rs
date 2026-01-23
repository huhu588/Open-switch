// Model 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use std::collections::HashMap;
use crate::config::{ConfigManager, Detector, OpenCodeModelInfo, OpenCodeThinkingConfig, OpenCodeVariantConfig};
use crate::error::AppError;

/// Model 列表项
#[derive(Debug, Clone, Serialize)]
pub struct ModelItem {
    pub id: String,
    pub name: String,
    /// 思考预算 (Claude 模型)
    pub thinking_budget: Option<u32>,
}

/// 添加 Model 的参数
#[derive(Debug, Deserialize)]
pub struct ModelInput {
    pub id: String,
    pub name: Option<String>,
    /// 推理强度 (OpenAI/Codex 模型)
    /// 可选值: "none", "minimal", "low", "medium", "high", "xhigh"
    pub reasoning_effort: Option<String>,
    /// 思考预算 (Anthropic/Claude 模型)
    /// 建议范围: 1024 - 128000
    pub thinking_budget: Option<u32>,
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
        .map(|(id, info)| {
            // 从 options.thinking 中提取 thinking_budget
            let thinking_budget = info.options.as_ref()
                .and_then(|opts| opts.thinking.as_ref())
                .map(|t| t.budget_tokens)
                .or(info.thinking_budget);
            ModelItem {
                id: id.clone(),
                name: info.name.clone(),
                thinking_budget,
            }
        })
        .collect();
    
    items.sort_by(|a, b| a.id.cmp(&b.id));
    
    Ok(items)
}

/// 获取单个 Model 详情
#[tauri::command]
pub fn get_model(
    provider_name: String,
    model_id: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<ModelItem, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let models = manager.opencode().get_models(&provider_name)?;
    
    let info = models.get(&model_id)
        .ok_or_else(|| AppError::Custom(format!("模型 '{}' 不存在", model_id)))?;
    
    let thinking_budget = info.options.as_ref()
        .and_then(|opts| opts.thinking.as_ref())
        .map(|t| t.budget_tokens)
        .or(info.thinking_budget);
    
    Ok(ModelItem {
        id: model_id,
        name: info.name.clone(),
        thinking_budget,
    })
}

/// 添加 Model
#[tauri::command]
pub fn add_model(
    provider_name: String,
    input: ModelInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 获取 provider 的 model_type
    let model_type = manager.opencode()
        .get_provider(&provider_name)?
        .and_then(|p| p.model_type.clone())
        .unwrap_or_else(|| "claude".to_string());
    
    // 根据 model_type 生成 variants
    let variants = build_variants(&model_type);
    
    let model_info = OpenCodeModelInfo {
        id: input.id.clone(),
        name: input.name.unwrap_or_else(|| input.id.clone()),
        limit: None,
        reasoning: Some(true),  // 启用 opencode 思考强度切换 (ctrl+t)
        variants: Some(variants),
        options: None,
        reasoning_effort: None,
        thinking_budget: None,
        model_detection: None,
    };
    
    manager.opencode_mut().add_model(&provider_name, input.id, model_info)?;
    
    Ok(())
}

/// 根据 model_type 构建 variants 配置
/// - Claude: 默认 / High / Max (使用 thinking.budgetTokens)
/// - Codex/Gemini: 默认 / Minimal / Low / Medium / High (使用 reasoningEffort)
/// 根据 model_type 生成默认 variants 配置
pub fn build_variants(model_type: &str) -> HashMap<String, OpenCodeVariantConfig> {
    let mut variants = HashMap::new();
    
    if model_type.eq_ignore_ascii_case("claude") {
        // Claude 模型: 默认 / High / Max
        variants.insert("default".to_string(), OpenCodeVariantConfig {
            reasoning_effort: None,
            thinking: Some(OpenCodeThinkingConfig {
                thinking_type: "enabled".to_string(),
                budget_tokens: 10000,  // 默认 10K
            }),
        });
        variants.insert("high".to_string(), OpenCodeVariantConfig {
            reasoning_effort: None,
            thinking: Some(OpenCodeThinkingConfig {
                thinking_type: "enabled".to_string(),
                budget_tokens: 50000,  // High 50K
            }),
        });
        variants.insert("max".to_string(), OpenCodeVariantConfig {
            reasoning_effort: None,
            thinking: Some(OpenCodeThinkingConfig {
                thinking_type: "enabled".to_string(),
                budget_tokens: 128000,  // Max 128K
            }),
        });
    } else {
        // Codex/Gemini 模型: 默认 / Minimal / Low / Medium / High
        variants.insert("default".to_string(), OpenCodeVariantConfig {
            reasoning_effort: Some("low".to_string()),
            thinking: None,
        });
        variants.insert("minimal".to_string(), OpenCodeVariantConfig {
            reasoning_effort: Some("minimal".to_string()),
            thinking: None,
        });
        variants.insert("low".to_string(), OpenCodeVariantConfig {
            reasoning_effort: Some("low".to_string()),
            thinking: None,
        });
        variants.insert("medium".to_string(), OpenCodeVariantConfig {
            reasoning_effort: Some("medium".to_string()),
            thinking: None,
        });
        variants.insert("high".to_string(), OpenCodeVariantConfig {
            reasoning_effort: Some("high".to_string()),
            thinking: None,
        });
    }
    
    variants
}

/// 更新 Model
#[tauri::command]
pub fn update_model(
    provider_name: String,
    model_id: String,
    input: ModelInput,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    // 获取 provider 的 model_type
    let model_type = manager.opencode()
        .get_provider(&provider_name)?
        .and_then(|p| p.model_type.clone())
        .unwrap_or_else(|| "claude".to_string());
    
    // 根据 model_type 生成 variants
    let variants = build_variants(&model_type);
    
    let model_info = OpenCodeModelInfo {
        id: input.id.clone(),
        name: input.name.unwrap_or_else(|| input.id.clone()),
        limit: None,
        reasoning: Some(true),  // 启用 opencode 思考强度切换 (ctrl+t)
        variants: Some(variants),
        options: None,
        reasoning_effort: None,
        thinking_budget: None,
        model_detection: None,
    };
    
    manager.opencode_mut().update_model(&provider_name, &model_id, model_info)?;
    
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
    "claude-4.1-opus",
    "claude-4.5-haiku",
    "claude-4.5-opus",
    "claude-4.5-sonnet",
];

/// Zhipu 预设模型列表
const ZHIPU_PRESET_MODELS: &[&str] = &[
    "glm-4.7",
    "glm-4.6",
];

/// Codex 预设模型列表
const CODEX_PRESET_MODELS: &[&str] = &[
    "gpt-5.2-codex",
    "gpt-5.2",
    "gpt-5.1-codex-max",
    "gpt-5.1-codex-mini",
    "gpt-5.1",
];

/// Gemini 预设模型列表
const GEMINI_PRESET_MODELS: &[&str] = &[
    "gemini-3-pro",
    "gemini-2.5-pro",
    "gemini-2.5-flash",
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

/// 判断是否应使用 Claude 预设模型
fn should_use_claude_preset(base_url: &str, model_type: &str, npm: Option<&str>) -> bool {
    if model_type.eq_ignore_ascii_case("claude") {
        return true;
    }
    if let Some(npm_name) = npm {
        if npm_name.to_lowercase().contains("anthropic") {
            return true;
        }
    }
    is_anthropic_protocol(base_url)
}

/// 判断是否为 Zhipu 协议
fn is_zhipu_protocol(base_url: &str) -> bool {
    let url_lower = base_url.to_lowercase();
    url_lower.contains("bigmodel.cn") ||
        url_lower.contains("zhipu") ||
        url_lower.contains("glm")
}

/// 解析预设模型列表
fn resolve_preset_models(
    base_url: &str,
    model_type: &str,
    npm: Option<&str>,
) -> Option<(&'static str, Vec<String>)> {
    if is_zhipu_protocol(base_url) {
        return Some((
            "zhipu",
            ZHIPU_PRESET_MODELS.iter().map(|s| s.to_string()).collect(),
        ));
    }

    if model_type.eq_ignore_ascii_case("codex") {
        return Some((
            "codex",
            CODEX_PRESET_MODELS.iter().map(|s| s.to_string()).collect(),
        ));
    }

    if model_type.eq_ignore_ascii_case("gemini") {
        return Some((
            "gemini",
            GEMINI_PRESET_MODELS.iter().map(|s| s.to_string()).collect(),
        ));
    }

    if should_use_claude_preset(base_url, model_type, npm) {
        return Some((
            "claude",
            CLAUDE_PRESET_MODELS.iter().map(|s| s.to_string()).collect(),
        ));
    }

    None
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
    let (base_url, api_key, model_type, npm) = {
        let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
        let provider = manager
            .opencode()
            .get_provider(&provider_name)?
            .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", provider_name)))?;
        (
            provider.options.base_url.clone(),
            provider.options.api_key.clone(),
            provider.model_type.clone().unwrap_or_else(|| "none".to_string()),
            provider.npm.clone(),
        )
    };

    // 检查是否匹配预设模型
    let preset = resolve_preset_models(&base_url, &model_type, npm.as_deref());
    if let Some((_source, models)) = preset {
        return Ok(models);
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

/// 内部批量添加模型（只读取/写入一次 opencode.json）
fn add_models_batch_internal(
    manager: &mut ConfigManager,
    provider_name: &str,
    inputs: Vec<ModelInput>,
) -> Result<(), AppError> {
    // 读取一次配置
    let mut config = manager
        .opencode()
        .read_config()
        .map_err(AppError::Custom)?;

    let provider = config
        .get_provider_mut(provider_name)
        .ok_or_else(|| AppError::Custom(format!("Provider '{}' 不存在", provider_name)))?;

    // 获取 provider 的 model_type
    let model_type = provider
        .model_type
        .clone()
        .unwrap_or_else(|| "claude".to_string());

    // 根据 model_type 生成 variants（只生成一次）
    let variants = build_variants(&model_type);

    for input in inputs {
        let model_id = input.id.clone();
        let model_name = input.name.unwrap_or_else(|| model_id.clone());

        // 忽略已存在的模型
        if provider.get_model(&model_id).is_some() {
            continue;
        }

        let model_info = OpenCodeModelInfo {
            id: model_id.clone(),
            name: model_name,
            limit: None,
            reasoning: Some(true), // 启用 opencode 思考强度切换 (ctrl+t)
            variants: Some(variants.clone()),
            options: None,
            reasoning_effort: None,
            thinking_budget: None,
            model_detection: None,
        };

        provider.add_model(model_id, model_info);
    }

    // 写回一次配置
    manager
        .opencode()
        .write_config(&config)
        .map_err(AppError::Custom)?;

    Ok(())
}

/// 批量添加 Model（仅传 ID，显示名默认等于 ID）
#[tauri::command]
pub fn add_models_batch(
    provider_name: String,
    model_ids: Vec<String>,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;

    let inputs = model_ids
        .into_iter()
        .map(|id| ModelInput {
            id,
            name: None,
            reasoning_effort: None,
            thinking_budget: None,
        })
        .collect();

    add_models_batch_internal(&mut manager, &provider_name, inputs)
}

/// 批量添加 Model（支持传入显示名，供 UI 预设一键添加使用）
#[tauri::command]
pub fn add_models_batch_detailed(
    provider_name: String,
    inputs: Vec<ModelInput>,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<(), AppError> {
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    add_models_batch_internal(&mut manager, &provider_name, inputs)
}
