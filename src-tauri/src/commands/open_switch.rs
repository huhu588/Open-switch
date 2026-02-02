// Ai Switch 统一配置命令
// 管理 ~/.ai-switch/config.json

use serde::{Deserialize, Serialize};
use crate::config::open_switch_manager::{
    OpenSwitchConfigManager, UnifiedProvider, ProviderApps, ProviderModels,
    ClaudeModels, CodexModels, GeminiModels, OpenCodeModels, OpenCodeModelDef,
};
use std::collections::HashMap;

/// 服务商输入（前端传入）
#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedProviderInput {
    /// 服务商ID（更新时必填，新增时可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// 服务商名称
    pub name: String,
    /// API 基础地址
    pub base_url: String,
    /// API 密钥
    pub api_key: String,
    /// 启用的应用
    pub apps: ProviderAppsInput,
    /// 各应用的模型配置
    #[serde(default)]
    pub models: ProviderModelsInput,
    /// 网站链接
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// 图标名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// 图标颜色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProviderAppsInput {
    #[serde(default)]
    pub opencode: bool,
    #[serde(default)]
    pub claude: bool,
    #[serde(default)]
    pub codex: bool,
    #[serde(default)]
    pub gemini: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProviderModelsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<OpenCodeModelsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude: Option<ClaudeModelsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<CodexModelsInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini: Option<GeminiModelsInput>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenCodeModelsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(default)]
    pub models: HashMap<String, OpenCodeModelDefInput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenCodeModelDefInput {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClaudeModelsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haiku_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sonnet_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opus_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CodexModelsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeminiModelsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// 服务商输出（返回前端）
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedProviderOutput {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key_masked: String,
    pub apps: ProviderApps,
    pub models: ProviderModels,
    pub website_url: Option<String>,
    pub notes: Option<String>,
    pub icon: Option<String>,
    pub icon_color: Option<String>,
    pub created_at: Option<i64>,
    pub sort_index: Option<i32>,
}

impl From<UnifiedProvider> for UnifiedProviderOutput {
    fn from(p: UnifiedProvider) -> Self {
        // 脱敏 API Key
        let api_key_masked = if p.api_key.len() <= 12 {
            "*".repeat(p.api_key.len().min(8))
        } else {
            format!("{}...{}", &p.api_key[..4], &p.api_key[p.api_key.len()-4..])
        };
        
        Self {
            id: p.id,
            name: p.name,
            base_url: p.base_url,
            api_key_masked,
            apps: p.apps,
            models: p.models,
            website_url: p.website_url,
            notes: p.notes,
            icon: p.icon,
            icon_color: p.icon_color,
            created_at: p.created_at,
            sort_index: p.sort_index,
        }
    }
}

/// 将输入转换为内部结构
fn input_to_provider(input: UnifiedProviderInput, existing_id: Option<String>) -> UnifiedProvider {
    let id = existing_id.or(input.id).unwrap_or_else(|| {
        format!("openswitch-{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });
    
    // 转换 OpenCode models
    let opencode_models = input.models.opencode.map(|m| {
        OpenCodeModels {
            npm: m.npm,
            models: m.models.into_iter().map(|(k, v)| {
                let limit = if v.context.is_some() || v.output.is_some() {
                    Some(crate::config::open_switch_manager::OpenCodeModelLimit {
                        context: v.context,
                        output: v.output,
                    })
                } else {
                    None
                };
                (k, OpenCodeModelDef { name: v.name, limit })
            }).collect(),
        }
    });
    
    UnifiedProvider {
        id,
        name: input.name,
        base_url: input.base_url,
        api_key: input.api_key,
        apps: ProviderApps {
            opencode: input.apps.opencode,
            claude: input.apps.claude,
            codex: input.apps.codex,
            gemini: input.apps.gemini,
        },
        models: ProviderModels {
            opencode: opencode_models,
            claude: input.models.claude.map(|m| ClaudeModels {
                model: m.model,
                haiku_model: m.haiku_model,
                sonnet_model: m.sonnet_model,
                opus_model: m.opus_model,
            }),
            codex: input.models.codex.map(|m| CodexModels {
                model: m.model,
                reasoning_effort: m.reasoning_effort,
            }),
            gemini: input.models.gemini.map(|m| GeminiModels {
                model: m.model,
            }),
        },
        website_url: input.website_url,
        notes: input.notes,
        icon: input.icon,
        icon_color: input.icon_color,
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        sort_index: None,
    }
}

// ============== Tauri Commands ==============

/// 获取 Ai Switch 所有统一服务商
#[tauri::command]
pub async fn get_open_switch_providers() -> Result<Vec<UnifiedProviderOutput>, String> {
    let manager = OpenSwitchConfigManager::new()?;
    let providers = manager.get_all_providers()?;
    Ok(providers.into_iter().map(|p| p.into()).collect())
}

/// 获取单个统一服务商（包含完整 API Key）
#[tauri::command]
pub async fn get_open_switch_provider(id: String) -> Result<UnifiedProvider, String> {
    let manager = OpenSwitchConfigManager::new()?;
    manager.get_provider(&id)?
        .ok_or_else(|| format!("服务商 {} 不存在", id))
}

/// 添加统一服务商
#[tauri::command]
pub async fn add_open_switch_provider(input: UnifiedProviderInput) -> Result<UnifiedProviderOutput, String> {
    let manager = OpenSwitchConfigManager::new()?;
    let provider = input_to_provider(input, None);
    manager.add_provider(provider.clone())?;
    Ok(provider.into())
}

/// 更新统一服务商
#[tauri::command]
pub async fn update_open_switch_provider(input: UnifiedProviderInput) -> Result<UnifiedProviderOutput, String> {
    let id = input.id.clone().ok_or_else(|| "更新时必须提供服务商 ID".to_string())?;
    let manager = OpenSwitchConfigManager::new()?;
    let provider = input_to_provider(input, Some(id));
    manager.update_provider(provider.clone())?;
    Ok(provider.into())
}

/// 删除统一服务商
#[tauri::command]
pub async fn remove_open_switch_provider(id: String) -> Result<(), String> {
    let manager = OpenSwitchConfigManager::new()?;
    manager.remove_provider(&id)
}

/// 应用服务商到指定工具
#[tauri::command]
pub async fn apply_open_switch_provider(id: String, app: String) -> Result<(), String> {
    let manager = OpenSwitchConfigManager::new()?;
    let provider = manager.get_provider(&id)?
        .ok_or_else(|| format!("服务商 {} 不存在", id))?;
    
    match app.as_str() {
        "claude" => {
            if !provider.apps.claude {
                return Err("该服务商未启用 Claude Code".to_string());
            }
            // 写入 ~/.claude/settings.json
            let home = dirs::home_dir().ok_or("无法获取用户目录")?;
            let claude_dir = home.join(".claude");
            let settings_path = claude_dir.join("settings.json");
            
            // 确保目录存在
            if !claude_dir.exists() {
                std::fs::create_dir_all(&claude_dir)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            
            // 读取现有配置或创建新配置
            let mut settings: serde_json::Value = if settings_path.exists() {
                let content = std::fs::read_to_string(&settings_path)
                    .map_err(|e| format!("读取配置失败: {}", e))?;
                serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({})
            };
            
            // 更新 env
            let env = manager.to_claude_env(&provider);
            settings["env"] = env;
            
            // 写入
            std::fs::write(&settings_path, serde_json::to_string_pretty(&settings).unwrap())
                .map_err(|e| format!("写入配置失败: {}", e))?;
            
            // 更新当前服务商
            manager.set_current_provider("claude", Some(id))?;
        }
        "codex" => {
            if !provider.apps.codex {
                return Err("该服务商未启用 Codex".to_string());
            }
            // 写入 ~/.codex/config.toml 和 ~/.codex/auth.json
            let home = dirs::home_dir().ok_or("无法获取用户目录")?;
            let codex_dir = home.join(".codex");
            let config_path = codex_dir.join("config.toml");
            let auth_path = codex_dir.join("auth.json");
            
            // 确保目录存在
            if !codex_dir.exists() {
                std::fs::create_dir_all(&codex_dir)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            
            // 写入 config.toml
            let config_toml = manager.to_codex_config(&provider);
            std::fs::write(&config_path, config_toml)
                .map_err(|e| format!("写入 config.toml 失败: {}", e))?;
            
            // 写入 auth.json
            let auth = serde_json::json!({
                "OPENAI_API_KEY": provider.api_key
            });
            std::fs::write(&auth_path, serde_json::to_string_pretty(&auth).unwrap())
                .map_err(|e| format!("写入 auth.json 失败: {}", e))?;
            
            // 更新当前服务商
            manager.set_current_provider("codex", Some(id))?;
        }
        "gemini" => {
            if !provider.apps.gemini {
                return Err("该服务商未启用 Gemini".to_string());
            }
            // 写入 ~/.gemini/.env
            let home = dirs::home_dir().ok_or("无法获取用户目录")?;
            let gemini_dir = home.join(".gemini");
            let env_path = gemini_dir.join(".env");
            
            // 确保目录存在
            if !gemini_dir.exists() {
                std::fs::create_dir_all(&gemini_dir)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
            
            // 写入 .env
            let env_content = manager.to_gemini_env(&provider);
            std::fs::write(&env_path, env_content)
                .map_err(|e| format!("写入 .env 失败: {}", e))?;
            
            // 更新当前服务商
            manager.set_current_provider("gemini", Some(id))?;
        }
        "opencode" => {
            if !provider.apps.opencode {
                return Err("该服务商未启用 OpenCode".to_string());
            }
            // OpenCode 配置由 OpenCodeConfigManager 管理，这里只记录当前服务商
            manager.set_current_provider("opencode", Some(id))?;
        }
        _ => return Err(format!("未知应用: {}", app)),
    }
    
    Ok(())
}

/// 从现有服务商导入到 Ai Switch 统一配置
#[tauri::command]
pub async fn import_to_open_switch(
    name: String,
    base_url: String,
    api_key: String,
    apps: ProviderAppsInput,
) -> Result<UnifiedProviderOutput, String> {
    let manager = OpenSwitchConfigManager::new()?;
    
    let provider = UnifiedProvider {
        id: format!("openswitch-{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
        name,
        base_url,
        api_key,
        apps: ProviderApps {
            opencode: apps.opencode,
            claude: apps.claude,
            codex: apps.codex,
            gemini: apps.gemini,
        },
        models: ProviderModels::default(),
        website_url: None,
        notes: None,
        icon: None,
        icon_color: None,
        created_at: Some(chrono::Utc::now().timestamp_millis()),
        sort_index: None,
    };
    
    manager.add_provider(provider.clone())?;
    Ok(provider.into())
}

/// 获取 Ai Switch 配置文件路径
#[tauri::command]
pub async fn get_open_switch_config_path() -> Result<String, String> {
    let manager = OpenSwitchConfigManager::new()?;
    Ok(manager.config_path().display().to_string())
}
