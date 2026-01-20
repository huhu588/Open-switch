// Model 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::config::{ConfigManager, OpenCodeModelInfo, Detector};
use crate::error::AppError;

/// Model 列表项
#[derive(Debug, Clone, Serialize)]
pub struct ModelItem {
    pub id: String,
    pub name: String,
}

/// 添加 Model 的参数
#[derive(Debug, Deserialize)]
pub struct ModelInput {
    pub id: String,
    pub name: Option<String>,
    /// 推理强度 (仅用于 GPT5.2/GPT5.1 等推理模型)
    pub reasoning_effort: Option<String>,
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
    
    let model_info = OpenCodeModelInfo {
        id: input.id.clone(),
        name: input.name.unwrap_or_else(|| input.id.clone()),
        limit: None,
        reasoning_effort: input.reasoning_effort,
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

    // #region agent log
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let log_path = r#"e:\项目仓库\研究仓库\OpencodeNewbie\Open Switch\.cursor\debug.log"#;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let run_id = std::env::var("DEBUG_RUN_ID").unwrap_or_else(|_| "run1".to_string());
        let base_url_str = base_url.replace('\\', "\\\\").replace('"', "\\\"");
        let npm_str = npm.clone().unwrap_or_default().replace('\\', "\\\\").replace('"', "\\\"");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(
                file,
                "{{\"sessionId\":\"debug-session\",\"runId\":\"{}\",\"hypothesisId\":\"H1\",\"location\":\"model.rs:fetch_site_models\",\"message\":\"fetch input\",\"data\":{{\"provider\":\"{}\",\"baseUrl\":\"{}\",\"modelType\":\"{}\",\"npm\":\"{}\"}},\"timestamp\":{}}}",
                run_id,
                provider_name,
                base_url_str,
                model_type,
                npm_str,
                ts
            );
        }
    }
    // #endregion
    
    let preset = resolve_preset_models(&base_url, &model_type, npm.as_deref());
    if let Some((source, models)) = preset {
        // #region agent log
        {
            use std::fs::OpenOptions;
            use std::io::Write;
            let log_path = r#"e:\项目仓库\研究仓库\OpencodeNewbie\Open Switch\.cursor\debug.log"#;
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let run_id = std::env::var("DEBUG_RUN_ID").unwrap_or_else(|_| "run1".to_string());
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
                let _ = writeln!(
                    file,
                    "{{\"sessionId\":\"debug-session\",\"runId\":\"{}\",\"hypothesisId\":\"H2\",\"location\":\"model.rs:fetch_site_models\",\"message\":\"preset selected\",\"data\":{{\"source\":\"{}\",\"modelCount\":{}}},\"timestamp\":{}}}",
                    run_id,
                    source,
                    models.len(),
                    ts
                );
            }
        }
        // #endregion
        return Ok(models);
    }

    // #region agent log
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let log_path = r#"e:\项目仓库\研究仓库\OpencodeNewbie\Open Switch\.cursor\debug.log"#;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let run_id = std::env::var("DEBUG_RUN_ID").unwrap_or_else(|_| "run1".to_string());
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(
                file,
                "{{\"sessionId\":\"debug-session\",\"runId\":\"{}\",\"hypothesisId\":\"H2\",\"location\":\"model.rs:fetch_site_models\",\"message\":\"detector selected\",\"data\":{{}},\"timestamp\":{}}}",
                run_id,
                ts
            );
        }
    }
    // #endregion
    
    // OpenAI 协议: 调用检测器获取模型列表
    let detector = Detector::new();
    let result = detector.detect_site(&base_url, &api_key).await;

    // #region agent log
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        let log_path = r#"e:\项目仓库\研究仓库\OpencodeNewbie\Open Switch\.cursor\debug.log"#;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let run_id = std::env::var("DEBUG_RUN_ID").unwrap_or_else(|_| "run1".to_string());
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(
                file,
                "{{\"sessionId\":\"debug-session\",\"runId\":\"{}\",\"hypothesisId\":\"H3\",\"location\":\"model.rs:fetch_site_models\",\"message\":\"detect_site result\",\"data\":{{\"isAvailable\":{},\"modelCount\":{}}},\"timestamp\":{}}}",
                run_id,
                result.is_available,
                result.available_models.len(),
                ts
            );
        }
    }
    // #endregion
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
            reasoning_effort: None,
            model_detection: None,
        };
        
        // 忽略已存在的模型
        let _ = manager.opencode_mut().add_model(&provider_name, model_id, model_info);
    }
    
    Ok(())
}
