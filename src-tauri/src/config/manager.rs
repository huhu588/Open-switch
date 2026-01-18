// 核心配置管理器
// 负责管理全局 config.json 和协调各供应商配置管理器

use crate::config::mcp_manager::McpConfigManager;
use crate::config::models::{GlobalConfig, OpenCodeActiveConfig, OpenCodeActiveReference};
use crate::config::opencode_manager::OpenCodeConfigManager;
use crate::config::ConfigError;
use std::fs;
use std::path::PathBuf;

/// 核心配置管理器
pub struct ConfigManager {
    global_config_file: PathBuf, // ~/.Open Switch/config.json
    opencode_manager: OpenCodeConfigManager,
    mcp_manager: McpConfigManager,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, ConfigError> {
        let home_dir = dirs::home_dir().ok_or_else(|| ConfigError::NotFound {
            name: "用户主目录".to_string(),
        })?;
        let config_dir = home_dir.join(".Open Switch");
        let global_config_file = config_dir.join("config.json");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir)?;

        // 初始化 OpenCode 配置管理器
        let opencode_manager = OpenCodeConfigManager::new(config_dir.clone())?;

        // 初始化 MCP 配置管理器
        let mcp_manager = McpConfigManager::new(config_dir)?;

        Ok(Self {
            global_config_file,
            opencode_manager,
            mcp_manager,
        })
    }

    // ========================================================================
    // 全局配置管理 (config.json)
    // ========================================================================

    /// 读取全局配置
    pub fn read_global_config(&self) -> Result<GlobalConfig, ConfigError> {
        if !self.global_config_file.exists() {
            // 如果文件不存在，返回新配置
            return Ok(GlobalConfig::new());
        }

        let content = fs::read_to_string(&self.global_config_file)?;

        Ok(serde_json::from_str(&content)?)
    }

    /// 写入全局配置
    pub fn write_global_config(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(config)?;

        Ok(fs::write(&self.global_config_file, content)?)
    }

    // ========================================================================
    // OpenCode 配置管理
    // ========================================================================

    /// 获取 OpenCode 配置管理器引用
    pub fn opencode(&self) -> &OpenCodeConfigManager {
        &self.opencode_manager
    }

    /// 获取 OpenCode 配置管理器可变引用
    pub fn opencode_mut(&mut self) -> &mut OpenCodeConfigManager {
        &mut self.opencode_manager
    }

    // ========================================================================
    // MCP 配置管理
    // ========================================================================

    /// 获取 MCP 配置管理器引用
    pub fn mcp(&self) -> &McpConfigManager {
        &self.mcp_manager
    }

    /// 获取 MCP 配置管理器可变引用
    pub fn mcp_mut(&mut self) -> &mut McpConfigManager {
        &mut self.mcp_manager
    }

    /// 获取当前激活的 OpenCode 配置
    pub fn get_active_opencode_config(&self) -> Result<Option<OpenCodeActiveConfig>, ConfigError> {
        let global_config = self.read_global_config()?;

        if let Some(ref reference) = global_config.active.opencode {
            let opencode_config = self.opencode_manager.read_config()?;

            let active_config = OpenCodeActiveConfig::from_reference(reference, &opencode_config)?;
            Ok(Some(active_config))
        } else {
            Ok(None)
        }
    }

    /// 验证所有 Provider 是否存在
    fn validate_providers_exist(
        &self,
        provider_names: &[String],
        opencode_config: &crate::config::models::OpenCodeConfig,
    ) -> Result<(), ConfigError> {
        for provider_name in provider_names {
            if opencode_config.get_provider(provider_name).is_none() {
                return Err(ConfigError::NotFound {
                    name: format!("Provider '{}'", provider_name),
                });
            }
        }
        Ok(())
    }

    /// 应用多个 OpenCode Provider 配置到全局
    pub fn apply_multiple_opencode_to_global(
        &mut self,
        provider_names: &[String],
    ) -> Result<(), ConfigError> {
        let opencode_config = self.opencode_manager.read_config()?;
        self.validate_providers_exist(provider_names, &opencode_config)?;

        if let Some(first_provider) = provider_names.first() {
            let reference = OpenCodeActiveReference {
                provider: first_provider.clone(),
            };

            let mut global_config = self.read_global_config()?;
            global_config.active.opencode = Some(reference);
            global_config.update_timestamp();
            self.write_global_config(&global_config)?;
        }

        self.opencode_manager
            .sync_multiple_providers_to_opencode(provider_names)?;

        Ok(())
    }

    /// 应用多个 OpenCode Provider 配置到项目级
    pub fn apply_multiple_opencode_to_project(
        &mut self,
        provider_names: &[String],
    ) -> Result<(), ConfigError> {
        let opencode_config = self.opencode_manager.read_config()?;
        self.validate_providers_exist(provider_names, &opencode_config)?;

        self.opencode_manager
            .sync_multiple_providers_to_project(provider_names)?;

        Ok(())
    }

    /// 检查 Provider 是否已应用到全局/项目配置
    /// 返回 (in_global, in_project)
    pub fn check_provider_applied(&self, provider_name: &str) -> Result<(bool, bool), ConfigError> {
        // 检查全局配置 ~/.config/opencode/package.json
        let home_dir = dirs::home_dir().ok_or_else(|| ConfigError::NotFound {
            name: "用户主目录".to_string(),
        })?;
        let global_config_path = home_dir.join(".config").join("opencode").join("package.json");
        let in_global = Self::check_provider_in_config(&global_config_path, provider_name);

        // 检查项目配置 ./.opencode/opencode.json
        let project_config_path = std::env::current_dir()
            .map(|p| p.join(".opencode").join("opencode.json"))
            .unwrap_or_default();
        let in_project = Self::check_provider_in_config(&project_config_path, provider_name);

        Ok((in_global, in_project))
    }

    /// 检查配置文件中是否包含指定的 provider
    fn check_provider_in_config(config_path: &PathBuf, provider_name: &str) -> bool {
        if !config_path.exists() {
            return false;
        }

        let content = match fs::read_to_string(config_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(_) => return false,
        };

        // 检查 provider 字段中是否包含指定的 provider
        if let Some(providers) = json.get("provider").and_then(|v| v.as_object()) {
            return providers.contains_key(provider_name);
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_global_config_read_write() {
        let manager = ConfigManager::new().unwrap();
        let config = GlobalConfig::new();

        let result = manager.write_global_config(&config);
        assert!(result.is_ok());

        let read_config = manager.read_global_config().unwrap();
        assert_eq!(read_config.version, "3.0.0");
    }
}
