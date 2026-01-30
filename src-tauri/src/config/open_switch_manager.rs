// Open Switch 统一配置管理器
// 管理 ~/.open-switch/config.json，存储跨工具的服务商配置
// 支持 OpenCode、Claude Code、Codex、Gemini

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Open Switch 配置文件版本
const CONFIG_VERSION: &str = "1.0";

/// Open Switch 统一配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSwitchConfig {
    /// 配置版本
    pub version: String,
    /// 服务商列表
    #[serde(default)]
    pub providers: HashMap<String, UnifiedProvider>,
    /// 当前激活的服务商（按工具）
    #[serde(default)]
    pub current: CurrentProviders,
    /// 元数据
    #[serde(default)]
    pub metadata: ConfigMetadata,
}

impl Default for OpenSwitchConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION.to_string(),
            providers: HashMap::new(),
            current: CurrentProviders::default(),
            metadata: ConfigMetadata::default(),
        }
    }
}

/// 当前激活的服务商
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CurrentProviders {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini: Option<String>,
}

/// 配置元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
}

/// 统一服务商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProvider {
    /// 唯一标识
    pub id: String,
    /// 服务商名称
    pub name: String,
    /// API 基础地址
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    /// API 密钥
    #[serde(rename = "apiKey")]
    pub api_key: String,
    /// 启用的应用
    pub apps: ProviderApps,
    /// 各应用的模型配置
    #[serde(default)]
    pub models: ProviderModels,
    /// 网站链接
    #[serde(skip_serializing_if = "Option::is_none", rename = "websiteUrl")]
    pub website_url: Option<String>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// 图标名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// 图标颜色
    #[serde(skip_serializing_if = "Option::is_none", rename = "iconColor")]
    pub icon_color: Option<String>,
    /// 创建时间
    #[serde(skip_serializing_if = "Option::is_none", rename = "createdAt")]
    pub created_at: Option<i64>,
    /// 排序索引
    #[serde(skip_serializing_if = "Option::is_none", rename = "sortIndex")]
    pub sort_index: Option<i32>,
}

/// 服务商启用的应用
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderApps {
    #[serde(default)]
    pub opencode: bool,
    #[serde(default)]
    pub claude: bool,
    #[serde(default)]
    pub codex: bool,
    #[serde(default)]
    pub gemini: bool,
}

/// 各应用的模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderModels {
    /// OpenCode 模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<OpenCodeModels>,
    /// Claude Code 模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude: Option<ClaudeModels>,
    /// Codex 模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<CodexModels>,
    /// Gemini 模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini: Option<GeminiModels>,
}

/// OpenCode 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeModels {
    /// npm 包名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    /// 模型列表
    #[serde(default)]
    pub models: HashMap<String, OpenCodeModelDef>,
}

/// OpenCode 单个模型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModelDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<OpenCodeModelLimit>,
}

/// OpenCode 模型限制
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

/// Claude Code 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeModels {
    /// 主模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Haiku 默认模型
    #[serde(skip_serializing_if = "Option::is_none", rename = "haikuModel")]
    pub haiku_model: Option<String>,
    /// Sonnet 默认模型
    #[serde(skip_serializing_if = "Option::is_none", rename = "sonnetModel")]
    pub sonnet_model: Option<String>,
    /// Opus 默认模型
    #[serde(skip_serializing_if = "Option::is_none", rename = "opusModel")]
    pub opus_model: Option<String>,
}

/// Codex 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexModels {
    /// 模型名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 推理强度
    #[serde(skip_serializing_if = "Option::is_none", rename = "reasoningEffort")]
    pub reasoning_effort: Option<String>,
}

/// Gemini 模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiModels {
    /// 模型名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Open Switch 配置管理器
pub struct OpenSwitchConfigManager {
    config_dir: PathBuf,
    config_path: PathBuf,
}

impl OpenSwitchConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;
        
        let config_dir = user_home.join(".open-switch");
        let config_path = config_dir.join("config.json");
        
        // 确保配置目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        
        Ok(Self {
            config_dir,
            config_path,
        })
    }
    
    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
    
    /// 获取配置目录路径
    #[allow(dead_code)]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
    
    /// 读取配置
    pub fn read_config(&self) -> Result<OpenSwitchConfig, String> {
        if !self.config_path.exists() {
            return Ok(OpenSwitchConfig::default());
        }
        
        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))
    }
    
    /// 写入配置
    pub fn write_config(&self, config: &OpenSwitchConfig) -> Result<(), String> {
        let mut config = config.clone();
        config.metadata.updated_at = chrono::Utc::now().timestamp_millis();
        
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        fs::write(&self.config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))
    }
    
    /// 添加服务商
    pub fn add_provider(&self, provider: UnifiedProvider) -> Result<(), String> {
        let mut config = self.read_config()?;
        
        if config.providers.contains_key(&provider.id) {
            return Err(format!("服务商 {} 已存在", provider.name));
        }
        
        config.providers.insert(provider.id.clone(), provider);
        self.write_config(&config)
    }
    
    /// 更新服务商
    pub fn update_provider(&self, provider: UnifiedProvider) -> Result<(), String> {
        let mut config = self.read_config()?;
        
        if !config.providers.contains_key(&provider.id) {
            return Err(format!("服务商 {} 不存在", provider.name));
        }
        
        config.providers.insert(provider.id.clone(), provider);
        self.write_config(&config)
    }
    
    /// 删除服务商
    pub fn remove_provider(&self, id: &str) -> Result<(), String> {
        let mut config = self.read_config()?;
        
        if config.providers.remove(id).is_none() {
            return Err(format!("服务商 {} 不存在", id));
        }
        
        self.write_config(&config)
    }
    
    /// 获取服务商
    pub fn get_provider(&self, id: &str) -> Result<Option<UnifiedProvider>, String> {
        let config = self.read_config()?;
        Ok(config.providers.get(id).cloned())
    }
    
    /// 获取所有服务商
    pub fn get_all_providers(&self) -> Result<Vec<UnifiedProvider>, String> {
        let config = self.read_config()?;
        let mut providers: Vec<_> = config.providers.values().cloned().collect();
        // 按排序索引排序
        providers.sort_by(|a, b| {
            let a_idx = a.sort_index.unwrap_or(i32::MAX);
            let b_idx = b.sort_index.unwrap_or(i32::MAX);
            a_idx.cmp(&b_idx)
        });
        Ok(providers)
    }
    
    /// 设置当前激活的服务商
    pub fn set_current_provider(&self, app: &str, provider_id: Option<String>) -> Result<(), String> {
        let mut config = self.read_config()?;
        
        match app {
            "opencode" => config.current.opencode = provider_id,
            "claude" => config.current.claude = provider_id,
            "codex" => config.current.codex = provider_id,
            "gemini" => config.current.gemini = provider_id,
            _ => return Err(format!("未知应用: {}", app)),
        }
        
        self.write_config(&config)
    }
    
    /// 生成 Claude Code 的 settings.json env 配置
    pub fn to_claude_env(&self, provider: &UnifiedProvider) -> serde_json::Value {
        let models = provider.models.claude.as_ref();
        let model = models
            .and_then(|m| m.model.clone())
            .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
        let haiku = models
            .and_then(|m| m.haiku_model.clone())
            .unwrap_or_else(|| model.clone());
        let sonnet = models
            .and_then(|m| m.sonnet_model.clone())
            .unwrap_or_else(|| model.clone());
        let opus = models
            .and_then(|m| m.opus_model.clone())
            .unwrap_or_else(|| model.clone());
        
        serde_json::json!({
            "ANTHROPIC_BASE_URL": provider.base_url,
            "ANTHROPIC_AUTH_TOKEN": provider.api_key,
            "ANTHROPIC_MODEL": model,
            "ANTHROPIC_DEFAULT_HAIKU_MODEL": haiku,
            "ANTHROPIC_DEFAULT_SONNET_MODEL": sonnet,
            "ANTHROPIC_DEFAULT_OPUS_MODEL": opus,
        })
    }
    
    /// 生成 Codex 的 config.toml 内容
    pub fn to_codex_config(&self, provider: &UnifiedProvider) -> String {
        let models = provider.models.codex.as_ref();
        let model = models
            .and_then(|m| m.model.clone())
            .unwrap_or_else(|| "gpt-4o".to_string());
        let reasoning_effort = models
            .and_then(|m| m.reasoning_effort.clone())
            .unwrap_or_else(|| "high".to_string());
        
        // 处理 base_url
        let base_trimmed = provider.base_url.trim_end_matches('/');
        let origin_only = match base_trimmed.split_once("://") {
            Some((_scheme, rest)) => !rest.contains('/'),
            None => !base_trimmed.contains('/'),
        };
        let codex_base_url = if base_trimmed.ends_with("/v1") {
            base_trimmed.to_string()
        } else if origin_only {
            format!("{}/v1", base_trimmed)
        } else {
            base_trimmed.to_string()
        };
        
        format!(
            r#"model_provider = "openswitch"
model = "{model}"
model_reasoning_effort = "{reasoning_effort}"
disable_response_storage = true

[model_providers.openswitch]
name = "{name}"
base_url = "{codex_base_url}"
wire_api = "responses"
requires_openai_auth = true"#,
            model = model,
            reasoning_effort = reasoning_effort,
            name = provider.name,
            codex_base_url = codex_base_url
        )
    }
    
    /// 生成 Gemini 的 .env 配置
    pub fn to_gemini_env(&self, provider: &UnifiedProvider) -> String {
        let models = provider.models.gemini.as_ref();
        let model = models
            .and_then(|m| m.model.clone())
            .unwrap_or_else(|| "gemini-2.5-pro".to_string());
        
        format!(
            r#"GOOGLE_GEMINI_BASE_URL={}
GEMINI_API_KEY={}
GEMINI_MODEL={}"#,
            provider.base_url,
            provider.api_key,
            model
        )
    }
}

/// 从现有 OpenCode Provider 导入
impl UnifiedProvider {
    pub fn from_opencode_provider(
        name: &str,
        base_url: &str,
        api_key: &str,
        npm: Option<String>,
        models: HashMap<String, OpenCodeModelDef>,
    ) -> Self {
        let id = format!("openswitch-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        Self {
            id,
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            apps: ProviderApps {
                opencode: true,
                claude: false,
                codex: false,
                gemini: false,
            },
            models: ProviderModels {
                opencode: Some(OpenCodeModels {
                    npm,
                    models,
                }),
                claude: None,
                codex: None,
                gemini: None,
            },
            website_url: None,
            notes: None,
            icon: None,
            icon_color: None,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            sort_index: None,
        }
    }
    
    /// 创建通用服务商（启用所有应用）
    pub fn new_universal(
        name: &str,
        base_url: &str,
        api_key: &str,
    ) -> Self {
        let id = format!("openswitch-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
        Self {
            id,
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            apps: ProviderApps {
                opencode: true,
                claude: true,
                codex: true,
                gemini: true,
            },
            models: ProviderModels::default(),
            website_url: None,
            notes: None,
            icon: None,
            icon_color: None,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            sort_index: None,
        }
    }
}
