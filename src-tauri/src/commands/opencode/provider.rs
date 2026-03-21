// Provider 相关的 Tauri commands

use serde::{Deserialize, Serialize};
use tauri::State;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::modules::opencode_config::claude_code_manager::ClaudeCodeConfigManager;
use crate::modules::opencode_config::codex_manager::CodexConfigManager;
use crate::modules::opencode_config::gemini_manager::GeminiConfigManager;
use crate::modules::opencode_config::models::OpenCodeModelInfo;
use crate::modules::opencode_config::openclaw_manager::OpenClawConfigManager;
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

/// 本机 Provider 导入结果
#[derive(Debug, Clone, Serialize)]
pub struct LocalProviderImportResult {
    pub imported: usize,
    pub updated: usize,
    pub skipped: usize,
    pub provider_names: Vec<String>,
}

#[derive(Debug, Clone)]
struct LocalProviderSeed {
    name: String,
    base_url: String,
    api_key: String,
    description: Option<String>,
    models: Vec<String>,
}

enum LocalProviderImportAction {
    Imported(String),
    Updated(String),
    Skipped,
}

fn push_unique_model(models: &mut Vec<String>, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return;
    }
    if !models.iter().any(|item| item == trimmed) {
        models.push(trimmed.to_string());
    }
}

fn normalize_description(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn default_imported_model(model_id: &str) -> OpenCodeModelInfo {
    OpenCodeModelInfo {
        id: model_id.to_string(),
        name: model_id.to_string(),
        limit: None,
        reasoning: None,
        variants: None,
        options: None,
        reasoning_effort: None,
        thinking_budget: None,
        model_detection: None,
    }
}

fn parse_env_file(path: &PathBuf) -> Result<HashMap<String, String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取 {} 失败: {}", path.display(), e))?;

    let mut env = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !key.is_empty() {
                env.insert(key.to_string(), value.to_string());
            }
        }
    }

    Ok(env)
}

fn first_env_value(env: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = env.get(*key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn first_json_string(
    map: &serde_json::Map<String, serde_json::Value>,
    keys: &[&str],
) -> Option<String> {
    for key in keys {
        if let Some(value) = map.get(*key).and_then(|item| item.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn build_claude_like_seed(
    env: &HashMap<String, String>,
    name: &str,
    description: &str,
    fallback_base_url: &str,
    extra_models: &[String],
) -> Option<LocalProviderSeed> {
    let api_key = first_env_value(env, &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"])
        .unwrap_or_default();
    let base_url = first_env_value(env, &["ANTHROPIC_BASE_URL"])
        .unwrap_or_else(|| fallback_base_url.to_string());

    let mut models = Vec::new();
    push_unique_model(&mut models, first_env_value(env, &["ANTHROPIC_MODEL"]));
    push_unique_model(
        &mut models,
        first_env_value(env, &["ANTHROPIC_REASONING_MODEL"]),
    );
    push_unique_model(
        &mut models,
        first_env_value(env, &["ANTHROPIC_DEFAULT_HAIKU_MODEL"]),
    );
    push_unique_model(
        &mut models,
        first_env_value(env, &["ANTHROPIC_DEFAULT_SONNET_MODEL"]),
    );
    push_unique_model(
        &mut models,
        first_env_value(env, &["ANTHROPIC_DEFAULT_OPUS_MODEL"]),
    );
    for model in extra_models {
        push_unique_model(&mut models, Some(model.clone()));
    }

    let has_material = !api_key.is_empty()
        || env.contains_key("ANTHROPIC_BASE_URL")
        || !models.is_empty();
    if !has_material {
        return None;
    }

    Some(LocalProviderSeed {
        name: name.to_string(),
        base_url,
        api_key,
        description: Some(description.to_string()),
        models,
    })
}

fn parse_openclaw_json_seed(path: &PathBuf) -> Result<Option<LocalProviderSeed>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取 {} 失败: {}", path.display(), e))?;
    let content = content.trim_start_matches('\u{feff}');
    let json: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("解析 {} 失败: {}", path.display(), e))?;

    let mut env = HashMap::new();
    if let Some(env_obj) = json.get("env").and_then(|value| value.as_object()) {
        if let Some(api_key) = first_json_string(env_obj, &["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"]) {
            env.insert("ANTHROPIC_API_KEY".to_string(), api_key);
        }
        if let Some(base_url) = first_json_string(env_obj, &["ANTHROPIC_BASE_URL"]) {
            env.insert("ANTHROPIC_BASE_URL".to_string(), base_url);
        }
        if let Some(model) = first_json_string(env_obj, &["ANTHROPIC_MODEL"]) {
            env.insert("ANTHROPIC_MODEL".to_string(), model);
        }
        if let Some(model) = first_json_string(env_obj, &["ANTHROPIC_REASONING_MODEL"]) {
            env.insert("ANTHROPIC_REASONING_MODEL".to_string(), model);
        }
        if let Some(model) = first_json_string(env_obj, &["ANTHROPIC_DEFAULT_HAIKU_MODEL"]) {
            env.insert("ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(), model);
        }
        if let Some(model) = first_json_string(env_obj, &["ANTHROPIC_DEFAULT_SONNET_MODEL"]) {
            env.insert("ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(), model);
        }
        if let Some(model) = first_json_string(env_obj, &["ANTHROPIC_DEFAULT_OPUS_MODEL"]) {
            env.insert("ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(), model);
        }
    }

    let mut extra_models = Vec::new();
    if let Some(model) = json
        .pointer("/agents/defaults/model/primary")
        .and_then(|value| value.as_str())
    {
        push_unique_model(&mut extra_models, Some(model.to_string()));
    }

    Ok(build_claude_like_seed(
        &env,
        "OpenClaw Local",
        "从本机 OpenClaw 配置导入",
        "https://api.anthropic.com",
        &extra_models,
    ))
}

fn collect_claude_local_seeds() -> Result<Vec<LocalProviderSeed>, String> {
    let manager = ClaudeCodeConfigManager::new()?;
    let settings = manager.read_settings()?;
    Ok(build_claude_like_seed(
        &settings.env,
        "Claude Code Local",
        "从本机 Claude Code 配置导入",
        "https://api.anthropic.com",
        &[],
    )
    .into_iter()
    .collect())
}

fn collect_codex_local_seeds() -> Result<Vec<LocalProviderSeed>, String> {
    let manager = CodexConfigManager::new()?;
    let providers = manager.get_model_providers()?;
    let api_key = manager.get_api_key()?.unwrap_or_default();

    let mut seeds = Vec::new();
    for (id, provider) in providers {
        let provider_name = if provider.name.trim().is_empty() {
            id
        } else {
            provider.name
        };
        let suffix = provider
            .env_key
            .as_deref()
            .map(|env_key| format!("，env_key={env_key}"))
            .unwrap_or_default();
        seeds.push(LocalProviderSeed {
            name: format!("Codex Local - {}", provider_name),
            base_url: if provider.base_url.trim().is_empty() {
                "https://api.openai.com/v1".to_string()
            } else {
                provider.base_url
            },
            api_key: api_key.clone(),
            description: Some(format!("从本机 Codex 配置导入{}", suffix)),
            models: Vec::new(),
        });
    }

    Ok(seeds)
}

fn collect_gemini_local_seeds() -> Result<Vec<LocalProviderSeed>, String> {
    let manager = GeminiConfigManager::new()?;
    let api_key = manager.get_api_key()?.unwrap_or_default();
    let base_url = manager
        .get_base_url()?
        .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
    let model = manager.get_model()?;

    let has_material = !api_key.is_empty() || model.is_some();
    if !has_material {
        return Ok(Vec::new());
    }

    let mut models = Vec::new();
    push_unique_model(&mut models, model);

    Ok(vec![LocalProviderSeed {
        name: "Gemini Local".to_string(),
        base_url,
        api_key,
        description: Some("从本机 Gemini 配置导入".to_string()),
        models,
    }])
}

fn collect_openclaw_local_seeds() -> Result<Vec<LocalProviderSeed>, String> {
    let manager = OpenClawConfigManager::new()?;
    let config_dir = manager.get_config_dir().clone();
    let json_candidates = [
        config_dir.join("openclaw.json"),
        config_dir.join("settings.json"),
        config_dir.join("settings.local.json"),
        config_dir.join("config.json"),
    ];

    for candidate in json_candidates {
        if let Some(seed) = parse_openclaw_json_seed(&candidate)? {
            return Ok(vec![seed]);
        }
    }

    let env_path = config_dir.join(".env");
    if env_path.exists() {
        let env = parse_env_file(&env_path)?;
        if let Some(seed) = build_claude_like_seed(
            &env,
            "OpenClaw Local",
            "从本机 OpenClaw 配置导入",
            "https://api.anthropic.com",
            &[],
        ) {
            return Ok(vec![seed]);
        }
    }

    Ok(Vec::new())
}

fn upsert_local_provider(
    manager: &mut ConfigManager,
    target_model_type: &str,
    seed: LocalProviderSeed,
) -> Result<LocalProviderImportAction, String> {
    let description = normalize_description(seed.description);
    let mut models = Vec::new();
    for model in seed.models {
        push_unique_model(&mut models, Some(model));
    }

    let existing = manager.opencode().get_provider(&seed.name)?;
    if let Some(existing_provider) = existing {
        let needs_base_url =
            !seed.base_url.trim().is_empty() && existing_provider.options.base_url != seed.base_url;
        let needs_api_key =
            !seed.api_key.trim().is_empty() && existing_provider.options.api_key != seed.api_key;
        let needs_description = description
            .as_deref()
            .map(|desc| existing_provider.metadata.description.as_deref() != Some(desc))
            .unwrap_or(false);
        let needs_model_type =
            existing_provider.model_type.as_deref() != Some(target_model_type);

        let mut touched = false;
        if needs_base_url || needs_api_key || needs_description || needs_model_type {
            manager.opencode_mut().update_provider_metadata(
                &seed.name,
                needs_base_url.then(|| seed.base_url.clone()),
                needs_api_key.then(|| seed.api_key.clone()),
                None,
                needs_description.then(|| description.clone()).flatten(),
                needs_model_type.then(|| target_model_type.to_string()),
            )?;
            touched = true;
        }

        for model_id in models {
            if existing_provider.models.contains_key(&model_id) {
                continue;
            }
            manager
                .opencode_mut()
                .add_model(&seed.name, model_id.clone(), default_imported_model(&model_id))?;
            touched = true;
        }

        return Ok(if touched {
            LocalProviderImportAction::Updated(seed.name)
        } else {
            LocalProviderImportAction::Skipped
        });
    }

    manager.opencode_mut().add_provider(
        seed.name.clone(),
        seed.base_url.clone(),
        seed.api_key.clone(),
        None,
        description.clone(),
        Some(target_model_type.to_string()),
        false,
    )?;

    for model_id in models {
        manager
            .opencode_mut()
            .add_model(&seed.name, model_id.clone(), default_imported_model(&model_id))?;
    }

    Ok(LocalProviderImportAction::Imported(seed.name))
}

fn import_seed_batch(
    manager: &mut ConfigManager,
    target_model_type: &str,
    seeds: Vec<LocalProviderSeed>,
) -> Result<LocalProviderImportResult, String> {
    let mut result = LocalProviderImportResult {
        imported: 0,
        updated: 0,
        skipped: 0,
        provider_names: Vec::new(),
    };

    for seed in seeds {
        match upsert_local_provider(manager, target_model_type, seed)? {
            LocalProviderImportAction::Imported(name) => {
                result.imported += 1;
                result.provider_names.push(name);
            }
            LocalProviderImportAction::Updated(name) => {
                result.updated += 1;
                result.provider_names.push(name);
            }
            LocalProviderImportAction::Skipped => {
                result.skipped += 1;
            }
        }
    }

    Ok(result)
}

fn import_opencode_local_providers(
    manager: &mut ConfigManager,
) -> Result<LocalProviderImportResult, String> {
    let config = manager.opencode().read_config()?;
    let mut result = LocalProviderImportResult {
        imported: 0,
        updated: 0,
        skipped: 0,
        provider_names: Vec::new(),
    };

    for (name, provider) in config.provider {
        if provider.model_type.is_some() {
            result.skipped += 1;
            continue;
        }

        manager.opencode_mut().update_provider_metadata(
            &name,
            None,
            None,
            None,
            None,
            Some("opencode".to_string()),
        )?;
        result.updated += 1;
        result.provider_names.push(name);
    }

    Ok(result)
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

/// 从本机 CLI / 配置文件导入 Provider 到当前平台分类
#[tauri::command]
pub fn import_local_provider_configs(
    model_type: String,
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<LocalProviderImportResult, AppError> {
    let mut manager = config_manager
        .lock()
        .map_err(|e| AppError::Custom(e.to_string()))?;

    let result = match model_type.as_str() {
        "claude" => {
            let seeds = collect_claude_local_seeds().map_err(AppError::Custom)?;
            import_seed_batch(&mut manager, "claude", seeds)
        }
        "codex" => {
            let seeds = collect_codex_local_seeds().map_err(AppError::Custom)?;
            import_seed_batch(&mut manager, "codex", seeds)
        }
        "gemini" => {
            let seeds = collect_gemini_local_seeds().map_err(AppError::Custom)?;
            import_seed_batch(&mut manager, "gemini", seeds)
        }
        "opencode" => import_opencode_local_providers(&mut manager),
        "openclaw" => {
            let seeds = collect_openclaw_local_seeds().map_err(AppError::Custom)?;
            import_seed_batch(&mut manager, "openclaw", seeds)
        }
        other => Err(format!("不支持的 Provider 类型: {}", other)),
    }
    .map_err(AppError::Custom)?;

    Ok(result)
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

// ============================================================================
// Cursor Welfare 一键应用到各工具
// ============================================================================

/// 一键应用到各工具的参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyToToolsInput {
    pub api_key: String,
    pub proxy_port: u16,
    #[serde(default)]
    pub tools: Vec<String>,
}

/// 各工具应用结果
#[derive(Debug, Serialize)]
pub struct ApplyToToolsResult {
    pub success: Vec<String>,
    pub failed: Vec<ApplyToolError>,
}

#[derive(Debug, Serialize)]
pub struct ApplyToolError {
    pub tool: String,
    pub error: String,
}

/// 一键将 Cursor 福利 Provider 应用到多个 CLI 工具
#[tauri::command]
pub async fn apply_cursor_welfare_to_tools(
    input: ApplyToToolsInput,
) -> Result<ApplyToToolsResult, String> {
    use crate::modules::opencode_config::claude_code_manager::{
        ClaudeCodeConfigManager, ClaudeCodeProvider,
    };
    use crate::modules::opencode_config::codex_manager::{CodexConfigManager, CodexProvider};
    use crate::modules::opencode_config::gemini_manager::{GeminiConfigManager, GeminiProvider};
    use crate::modules::opencode_config::openclaw_manager::{
        OpenClawConfigManager, OpenClawProvider,
    };

    let proxy_base = format!("http://localhost:{}", input.proxy_port);
    let tools = if input.tools.is_empty() {
        vec![
            "claude".to_string(),
            "codex".to_string(),
            "gemini".to_string(),
            "opencode".to_string(),
            "openclaw".to_string(),
        ]
    } else {
        input.tools
    };

    let mut result = ApplyToToolsResult {
        success: Vec::new(),
        failed: Vec::new(),
    };

    for tool in &tools {
        let res = match tool.as_str() {
            "claude" => {
                let manager = ClaudeCodeConfigManager::new();
                manager.and_then(|m| {
                    m.apply_provider(&ClaudeCodeProvider {
                        name: "Cursor Welfare".to_string(),
                        api_key: input.api_key.clone(),
                        base_url: Some(format!("{}/cursor-welfare", proxy_base)),
                        model: Some("claude-sonnet-4.6".to_string()),
                        enabled: true,
                        description: Some("Cursor 福利自动配置".to_string()),
                    })
                })
            }
            "codex" => {
                let manager = CodexConfigManager::new();
                manager.and_then(|m| {
                    m.apply_provider(&CodexProvider {
                        name: "Cursor Welfare".to_string(),
                        api_key: input.api_key.clone(),
                        base_url: format!("{}/cursor-welfare", proxy_base),
                        env_key: None,
                        enabled: true,
                        description: Some("Cursor 福利自动配置".to_string()),
                    })
                })
            }
            "gemini" => {
                let manager = GeminiConfigManager::new();
                manager.and_then(|m| {
                    m.apply_provider(&GeminiProvider {
                        name: "Cursor Welfare".to_string(),
                        api_key: input.api_key.clone(),
                        base_url: Some(format!("{}/cursor-welfare", proxy_base)),
                        model: Some("claude-sonnet-4.6".to_string()),
                        enabled: true,
                        description: Some("Cursor 福利自动配置".to_string()),
                    })
                })
            }
            "opencode" => {
                Ok(())
            }
            "openclaw" => {
                let manager = OpenClawConfigManager::new();
                manager.and_then(|m| {
                    m.apply_provider(&OpenClawProvider {
                        name: "Cursor Welfare".to_string(),
                        base_url: format!("{}/cursor-welfare/v1", proxy_base),
                        api_key: Some(input.api_key.clone()),
                        model: Some("claude-sonnet-4.6".to_string()),
                    })
                })
            }
            _ => Err(format!("未知工具: {}", tool)),
        };

        match res {
            Ok(()) => result.success.push(tool.clone()),
            Err(e) => result.failed.push(ApplyToolError {
                tool: tool.clone(),
                error: e,
            }),
        }
    }

    Ok(result)
}
