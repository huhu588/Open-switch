// OpenCode 配置管理器
// 负责管理 ~/.Open Switch/opencode.json 和同步到 ~/.config/opencode/package.json

use crate::config::models::{OpenCodeConfig, OpenCodeModelInfo, OpenCodeProvider};
use crate::config::ConfigError;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SYNC_THEME: &str = "tokyonight";

pub struct OpenCodeConfigManager {
    config_dir: PathBuf,
    // ~/.opencode/opencode.json
    home_dir: PathBuf,
    home_json: PathBuf,
    // ~/.config/opencode/package.json
    config_dir_alt: PathBuf,
    config_json_alt: PathBuf,
}

impl OpenCodeConfigManager {
    pub fn new(config_dir: PathBuf) -> Result<Self, ConfigError> {
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // 主路径: ~/.opencode/opencode.json
        let user_home = dirs::home_dir()
            .ok_or_else(|| ConfigError::NotFound {
                name: "用户主目录".to_string(),
            })?;
        
        let home_dir = user_home.join(".opencode");
        let home_json = home_dir.join("opencode.json");

        // 备用路径: ~/.config/opencode/package.json
        let config_dir_alt = user_home.join(".config").join("opencode");
        let config_json_alt = config_dir_alt.join("package.json");

        Ok(Self {
            config_dir,
            home_dir,
            home_json,
            config_dir_alt,
            config_json_alt,
        })
    }

    pub fn read_config(&self) -> Result<OpenCodeConfig, String> {
        let config_path = self.config_dir.join("opencode.json");
        if !config_path.exists() {
            return Ok(OpenCodeConfig::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("解析 opencode.json 失败: {}", e))
    }

    pub fn write_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
        let config_path = self.config_dir.join("opencode.json");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

        fs::write(&config_path, content).map_err(|e| format!("写入 opencode.json 失败: {}", e))
    }

    pub fn get_provider(&self, provider_name: &str) -> Result<Option<OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.get_provider(provider_name).cloned())
    }

    pub fn get_all_providers(&self) -> Result<HashMap<String, OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.provider)
    }

    pub fn add_provider(
        &mut self,
        provider_name: String,
        base_url: String,
        api_key: String,
        npm: Option<String>,
        description: Option<String>,
        model_type: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.get_provider(&provider_name).is_some() {
            return Err(format!("Provider '{}' 已存在", provider_name));
        }

        let provider =
            OpenCodeProvider::new(provider_name.clone(), base_url, api_key, npm, description, model_type);
        config.add_provider(provider_name, provider);

        self.write_config(&config)
    }

    pub fn update_provider_metadata(
        &mut self,
        provider_name: &str,
        base_url: Option<String>,
        api_key: Option<String>,
        npm: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if let Some(url) = base_url {
            provider.set_base_url(url);
        }
        if let Some(key) = api_key {
            provider.set_api_key(key);
        }
        if let Some(npm_val) = npm {
            provider.npm = Some(npm_val);
            provider.update_timestamp();
        }
        if let Some(desc) = description {
            provider.metadata.description = Some(desc);
            provider.update_timestamp();
        }

        self.write_config(&config)
    }

    pub fn delete_provider(&mut self, provider_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.remove_provider(provider_name).is_none() {
            return Err(format!("Provider '{}' 不存在", provider_name));
        }

        self.write_config(&config)
    }

    pub fn get_models(
        &self,
        provider_name: &str,
    ) -> Result<HashMap<String, OpenCodeModelInfo>, String> {
        let config = self.read_config()?;
        let provider = config
            .get_provider(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        Ok(provider.models.clone())
    }

    pub fn add_model(
        &mut self,
        provider_name: &str,
        model_id: String,
        model_info: OpenCodeModelInfo,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if provider.get_model(&model_id).is_some() {
            return Err(format!(
                "模型 '{}' 已存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        provider.add_model(model_id, model_info);

        self.write_config(&config)
    }

    pub fn delete_model(&mut self, provider_name: &str, model_id: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if provider.remove_model(model_id).is_none() {
            return Err(format!(
                "模型 '{}' 不存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        self.write_config(&config)
    }

    pub fn sync_multiple_providers_to_opencode(
        &self,
        provider_names: &[String],
    ) -> Result<(), String> {
        let config = self.read_config()?;
        // 同步到两个路径
        sync_providers(&config, provider_names, &self.home_dir, &self.home_json)?;
        sync_providers(&config, provider_names, &self.config_dir_alt, &self.config_json_alt)
    }

    pub fn sync_multiple_providers_to_project(
        &self,
        provider_names: &[String],
    ) -> Result<(), String> {
        let config = self.read_config()?;
        let (project_dir, project_json) = get_project_opencode_paths()?;
        sync_providers(&config, provider_names, &project_dir, &project_json)
    }
}

fn ensure_dir_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(())
}

fn get_project_opencode_paths() -> Result<(PathBuf, PathBuf), String> {
    let project_dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?
        .join(".opencode");
    let project_json = project_dir.join("opencode.json");
    Ok((project_dir, project_json))
}

fn sync_providers(
    config: &OpenCodeConfig,
    provider_names: &[String],
    dir: &PathBuf,
    json_path: &PathBuf,
) -> Result<(), String> {
    ensure_dir_exists(dir)?;
    sync_providers_to_file(config, provider_names, json_path)
}

fn sync_providers_to_file(
    config: &OpenCodeConfig,
    provider_names: &[String],
    target_path: &PathBuf,
) -> Result<(), String> {
    // 读取现有配置文件（如果存在）
    let mut existing_data: serde_json::Value = if target_path.exists() {
        let content = fs::read_to_string(target_path)
            .map_err(|e| format!("读取现有配置失败: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 确保基本结构存在
    if existing_data.get("$schema").is_none() {
        existing_data["$schema"] = serde_json::json!("https://opencode.ai/config.json");
    }
    if existing_data.get("theme").is_none() {
        existing_data["theme"] = serde_json::json!(SYNC_THEME);
    }
    if existing_data.get("autoupdate").is_none() {
        existing_data["autoupdate"] = serde_json::json!(false);
    }
    if existing_data.get("tools").is_none() {
        existing_data["tools"] = serde_json::json!({ "webfetch": true });
    }

    // 获取要同步的 providers 的 model_type
    let mut target_model_type: Option<String> = None;
    for name in provider_names {
        if let Some(provider) = config.get_provider(name) {
            target_model_type = provider.model_type.clone();
            break;
        }
    }

    // 获取内部配置中所有属于该 model_type 的 provider 名称
    let providers_to_remove: Vec<String> = if let Some(ref mt) = target_model_type {
        config
            .provider
            .iter()
            .filter(|(_, p)| p.model_type.as_ref() == Some(mt))
            .map(|(name, _)| name.clone())
            .collect()
    } else {
        vec![]
    };

    // 从现有配置中获取 providers，保留其他 model_type 的
    let mut new_providers = if let Some(existing_providers) = existing_data
        .get("provider")
        .and_then(|p| p.as_object())
    {
        let mut map = existing_providers.clone();
        // 删除该 model_type 的所有 provider
        for name in &providers_to_remove {
            map.remove(name);
        }
        map
    } else {
        serde_json::Map::new()
    };

    // 添加用户选择的新 provider
    for name in provider_names {
        if let Some(provider) = config.get_provider(name) {
            if let Ok(mut value) = serde_json::to_value(provider) {
                if let Some(obj) = value.as_object_mut() {
                    // 移除 model_type 字段，opencode 不识别这个字段
                    obj.remove("model_type");
                    
                    // Anthropic 协议需要 baseURL 以 /v1 结尾
                    let is_anthropic = obj.get("npm")
                        .and_then(|v| v.as_str())
                        .map(|s| s.contains("anthropic"))
                        .unwrap_or(false);
                    
                    if is_anthropic {
                        if let Some(options) = obj.get_mut("options").and_then(|o| o.as_object_mut()) {
                            if let Some(base_url) = options.get("baseURL").and_then(|u| u.as_str()) {
                                // 如果 baseURL 不以 /v1 结尾，自动添加
                                if !base_url.ends_with("/v1") && !base_url.ends_with("/v1/") {
                                    let new_url = format!("{}/v1", base_url.trim_end_matches('/'));
                                    options.insert("baseURL".to_string(), serde_json::json!(new_url));
                                }
                            }
                        }
                    }
                }
                new_providers.insert(name.clone(), value);
            }
        }
    }

    existing_data["provider"] = serde_json::Value::Object(new_providers);

    let content = serde_json::to_string_pretty(&existing_data)
        .map_err(|e| format!("序列化同步数据失败: {}", e))?;

    backup_existing_file(target_path)?;

    fs::write(target_path, content).map_err(|e| format!("写入失败: {}", e))
}

fn backup_existing_file(target_path: &PathBuf) -> Result<(), String> {
    if target_path.exists() {
        let backup_path = target_path.with_extension("json.bak");
        fs::copy(target_path, &backup_path).map_err(|e| format!("备份文件失败: {}", e))?;
    }
    Ok(())
}
