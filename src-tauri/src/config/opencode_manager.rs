// OpenCode 配置管理器
// 直接管理 OpenCode 官方配置文件：~/.config/opencode/opencode.json
// 所有读写操作都直接作用于 OpenCode 的配置文件

use crate::config::models::{OpenCodeConfig, OpenCodeModelInfo, OpenCodeProvider};
use crate::config::ConfigError;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SYNC_THEME: &str = "tokyonight";

/// Provider 元数据（Open Switch 特有字段，不同步到 opencode.json）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderMetadataStorage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_type: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_enabled() -> bool {
    true
}

pub struct OpenCodeConfigManager {
    // OpenCode 官方主配置文件：~/.config/opencode/opencode.json（所有读写操作的目标）
    opencode_config_json: PathBuf,
    // Open Switch 元数据文件：~/.config/opencode/switch_metadata.json
    metadata_json: PathBuf,
    // 备份路径：~/.opencode/opencode.json（仅用于备份）
    home_dir: PathBuf,
    home_json: PathBuf,
    // 旧的 package.json 路径：~/.config/opencode/package.json（已废弃，保留以兼容旧版本）
    #[allow(dead_code)]
    config_json_alt: PathBuf,
}

impl OpenCodeConfigManager {
    pub fn new(_config_dir: PathBuf) -> Result<Self, ConfigError> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| ConfigError::NotFound {
                name: "用户主目录".to_string(),
            })?;
        
        // OpenCode 官方主配置路径：~/.config/opencode/opencode.json
        let opencode_config_dir = user_home.join(".config").join("opencode");
        let opencode_config_json = opencode_config_dir.join("opencode.json");
        let metadata_json = opencode_config_dir.join("switch_metadata.json");

        // 确保 OpenCode 配置目录存在
        if !opencode_config_dir.exists() {
            fs::create_dir_all(&opencode_config_dir)?;
        }

        // 备份路径（仅用于备份）
        let home_dir = user_home.join(".opencode");
        let home_json = home_dir.join("opencode.json");
        let config_json_alt = opencode_config_dir.join("package.json");

        Ok(Self {
            opencode_config_json,
            metadata_json,
            home_dir,
            home_json,
            config_json_alt,
        })
    }

    /// 读取元数据存储
    fn read_metadata(&self) -> Result<HashMap<String, ProviderMetadataStorage>, String> {
        if !self.metadata_json.exists() {
            return Ok(HashMap::new());
        }
        
        let content = fs::read_to_string(&self.metadata_json)
            .map_err(|e| format!("读取元数据文件失败: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("解析元数据文件失败: {}", e))
    }
    
    /// 写入元数据存储
    fn write_metadata(&self, metadata: &HashMap<String, ProviderMetadataStorage>) -> Result<(), String> {
        let content = serde_json::to_string_pretty(metadata)
            .map_err(|e| format!("序列化元数据失败: {}", e))?;
        
        fs::write(&self.metadata_json, content)
            .map_err(|e| format!("写入元数据文件失败: {}", e))
    }

    /// 直接读取 OpenCode 官方配置文件：~/.config/opencode/opencode.json
    /// 同时从元数据文件中恢复 model_type、enabled 等字段
    pub fn read_config(&self) -> Result<OpenCodeConfig, String> {
        let config_path = &self.opencode_config_json;
        if !config_path.exists() {
            return Ok(OpenCodeConfig::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

        // 移除 UTF-8 BOM（如果存在），PowerShell 生成的文件可能包含 BOM
        let content = content.trim_start_matches('\u{feff}');

        // opencode.json 中 models 通常使用 HashMap key 作为模型 id，不会在 value 内重复存储 id
        // 为兼容旧文件/外部编辑，允许缺失 id，并在读取后用 key 回填
        let mut config: OpenCodeConfig = serde_json::from_str(content)
            .map_err(|e| format!("解析 opencode.json 失败: {}", e))?;

        for (_provider_name, provider) in config.provider.iter_mut() {
            for (model_id, model_info) in provider.models.iter_mut() {
                if model_info.id.is_empty() {
                    model_info.id = model_id.clone();
                }
            }
        }
        
        // 从元数据文件中恢复 model_type、enabled 等字段
        let metadata = self.read_metadata().unwrap_or_default();
        for (provider_name, provider) in config.provider.iter_mut() {
            if let Some(meta) = metadata.get(provider_name) {
                provider.model_type = meta.model_type.clone();
                provider.enabled = meta.enabled;
                if meta.description.is_some() {
                    provider.metadata.description = meta.description.clone();
                }
            }
        }

        Ok(config)
    }

    /// 直接写入 OpenCode 官方配置文件：~/.config/opencode/opencode.json
    /// 同时将 model_type、enabled 等字段保存到元数据文件中
    pub fn write_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
        let config_path = &self.opencode_config_json;
        
        // 保存元数据到独立文件
        let mut metadata = HashMap::new();
        for (provider_name, provider) in &config.provider {
            metadata.insert(
                provider_name.clone(),
                ProviderMetadataStorage {
                    model_type: provider.model_type.clone(),
                    enabled: provider.enabled,
                    description: provider.metadata.description.clone(),
                },
            );
        }
        self.write_metadata(&metadata)?;
        
        // 先序列化为 JSON Value，然后移除 OpenCode 不支持的字段
        let mut value = serde_json::to_value(config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        // 移除所有 provider 中的工具内部字段
        if let Some(providers) = value.get_mut("provider").and_then(|p| p.as_object_mut()) {
            for (_name, provider) in providers.iter_mut() {
                if let Some(provider_obj) = provider.as_object_mut() {
                    // 移除 OpenCode 不识别的字段
                    provider_obj.remove("model_type");
                    provider_obj.remove("enabled");
                    provider_obj.remove("auto_add_v1_suffix");
                    provider_obj.remove("metadata");
                }
            }
        }
        
        let content = serde_json::to_string_pretty(&value)
            .map_err(|e| format!("序列化 JSON 失败: {}", e))?;

        fs::write(config_path, content)
            .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;
        
        // 可选：同步备份到 ~/.opencode/opencode.json
        if let Err(e) = self.backup_to_home() {
            eprintln!("备份到 ~/.opencode/opencode.json 失败: {}", e);
        }
        
        Ok(())
    }

    /// 备份当前配置到 ~/.opencode/opencode.json
    fn backup_to_home(&self) -> Result<(), String> {
        if !self.home_dir.exists() {
            fs::create_dir_all(&self.home_dir)
                .map_err(|e| format!("创建备份目录失败: {}", e))?;
        }
        
        fs::copy(&self.opencode_config_json, &self.home_json)
            .map_err(|e| format!("备份配置文件失败: {}", e))?;
        
        Ok(())
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
        auto_add_v1_suffix: bool,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.get_provider(&provider_name).is_some() {
            return Err(format!("Provider '{}' 已存在", provider_name));
        }

        let provider = OpenCodeProvider::new_with_v1_suffix(
            provider_name.clone(),
            base_url,
            api_key,
            npm,
            description,
            model_type,
            auto_add_v1_suffix,
        );
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
        model_type: Option<String>,
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
        if let Some(mt) = model_type {
            provider.model_type = Some(mt);
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

    pub fn toggle_provider(&mut self, provider_name: &str, enabled: bool) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        provider.enabled = enabled;
        provider.update_timestamp();

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

    pub fn update_model(
        &mut self,
        provider_name: &str,
        model_id: &str,
        model_info: OpenCodeModelInfo,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if provider.get_model(model_id).is_none() {
            return Err(format!(
                "模型 '{}' 不存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        // 更新模型信息
        provider.models.insert(model_id.to_string(), model_info);

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

    /// 已废弃：配置已经直接在 OpenCode 主配置文件中
    /// 保留此方法以保持向后兼容，但实际不执行任何操作
    pub fn sync_multiple_providers_to_opencode(
        &self,
        _provider_names: &[String],
    ) -> Result<(), String> {
        // 由于现在直接读写 OpenCode 配置文件，无需同步
        Ok(())
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
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?;
    // 开发模式下 Tauri 的 current_dir 往往是 src-tauri，需要回退到项目根目录
    let project_root = match current_dir.file_name().and_then(|n| n.to_str()) {
        Some("src-tauri") => current_dir.parent().unwrap_or(&current_dir).to_path_buf(),
        _ => current_dir.clone(),
    };
    let project_dir = project_root.join(".opencode");
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
                    // 移除 model_type 和 enabled 字段，opencode 不识别这些字段
                    obj.remove("model_type");
                    obj.remove("enabled");
                    
                    // 根据 auto_add_v1_suffix 和协议类型决定是否添加 /v1 后缀
                    let is_anthropic = obj.get("npm")
                        .and_then(|v| v.as_str())
                        .map(|s| s.contains("anthropic"))
                        .unwrap_or(false);
                    
                    // 只有当 auto_add_v1_suffix 为 true 且使用 Anthropic 协议时才添加 /v1
                    if is_anthropic && provider.auto_add_v1_suffix {
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

/// 已部署的服务商信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeployedProviderInfo {
    pub name: String,
    pub base_url: String,
    pub model_count: usize,
    pub source: String, // "global" 或 "project"
}

impl OpenCodeConfigManager {
    /// 读取已部署到 opencode 的全局配置
    pub fn read_deployed_global_config(&self) -> Result<serde_json::Value, String> {
        // 优先读取主配置文件：~/.config/opencode/opencode.json
        if self.opencode_config_json.exists() {
            let content = fs::read_to_string(&self.opencode_config_json)
                .map_err(|e| format!("读取全局配置失败: {}", e))?;
            return serde_json::from_str(&content)
                .map_err(|e| format!("解析全局配置失败: {}", e));
        }
        
        // 备用：~/.opencode/opencode.json（备份路径）
        if self.home_json.exists() {
            let content = fs::read_to_string(&self.home_json)
                .map_err(|e| format!("读取全局配置失败: {}", e))?;
            return serde_json::from_str(&content)
                .map_err(|e| format!("解析全局配置失败: {}", e));
        }
        
        Ok(serde_json::json!({}))
    }

    /// 获取已部署到 opencode 的服务商列表
    pub fn get_deployed_providers(&self) -> Result<Vec<DeployedProviderInfo>, String> {
        let mut result = Vec::new();
        
        // 读取全局配置
        let global_config = self.read_deployed_global_config()?;
        if let Some(providers) = global_config.get("provider").and_then(|p| p.as_object()) {
            for (name, value) in providers {
                let base_url = value
                    .get("options")
                    .and_then(|o| o.get("baseURL"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let model_count = value
                    .get("models")
                    .and_then(|m| m.as_object())
                    .map(|m| m.len())
                    .unwrap_or(0);
                
                result.push(DeployedProviderInfo {
                    name: name.clone(),
                    base_url,
                    model_count,
                    source: "global".to_string(),
                });
            }
        }
        
        // 读取项目配置
        if let Ok((_, project_json)) = get_project_opencode_paths() {
            if project_json.exists() {
                if let Ok(content) = fs::read_to_string(&project_json) {
                    if let Ok(project_config) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(providers) = project_config.get("provider").and_then(|p| p.as_object()) {
                            for (name, value) in providers {
                                // 检查是否已存在于全局配置中
                                if result.iter().any(|p| p.name == *name) {
                                    continue;
                                }
                                
                                let base_url = value
                                    .get("options")
                                    .and_then(|o| o.get("baseURL"))
                                    .and_then(|u| u.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let model_count = value
                                    .get("models")
                                    .and_then(|m| m.as_object())
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                
                                result.push(DeployedProviderInfo {
                                    name: name.clone(),
                                    base_url,
                                    model_count,
                                    source: "project".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // 按名称排序
        result.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(result)
    }

    /// 从已部署的 opencode 配置中删除服务商
    pub fn remove_deployed_provider(&self, provider_name: &str, from_global: bool, from_project: bool) -> Result<(), String> {
        if from_global {
            // 从 ~/.opencode/opencode.json 删除
            self.remove_provider_from_file(&self.home_json, provider_name)?;
            // 从 ~/.config/opencode/package.json 删除
            self.remove_provider_from_file(&self.config_json_alt, provider_name)?;
        }
        
        if from_project {
            if let Ok((_, project_json)) = get_project_opencode_paths() {
                self.remove_provider_from_file(&project_json, provider_name)?;
            }
        }
        
        Ok(())
    }

    /// 从指定配置文件中删除服务商
    fn remove_provider_from_file(&self, file_path: &PathBuf, provider_name: &str) -> Result<(), String> {
        if !file_path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
        let mut config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;
        
        if let Some(providers) = config.get_mut("provider").and_then(|p| p.as_object_mut()) {
            if providers.remove(provider_name).is_some() {
                // 备份原文件
                backup_existing_file(file_path)?;
                
                // 写入更新后的配置
                let new_content = serde_json::to_string_pretty(&config)
                    .map_err(|e| format!("序列化配置失败: {}", e))?;
                fs::write(file_path, new_content)
                    .map_err(|e| format!("写入配置文件失败: {}", e))?;
            }
        }
        
        Ok(())
    }
}
