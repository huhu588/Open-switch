use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenClawStatus {
    pub installed: bool,
    pub config_dir: String,
    pub has_agents_md: bool,
    pub has_soul_md: bool,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenClawProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

pub struct OpenClawConfigManager {
    config_dir: PathBuf,
}

impl OpenClawConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home =
            dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;

        let config_dir = user_home.join(".openclaw");

        Ok(Self { config_dir })
    }

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    pub fn get_config_path(&self) -> String {
        self.config_dir.to_string_lossy().to_string()
    }

    pub fn is_installed(&self) -> bool {
        self.config_dir.exists()
    }

    pub fn get_status(&self) -> OpenClawStatus {
        let config_dir_str = self.config_dir.to_string_lossy().to_string();
        let has_agents_md = self.config_dir.join("AGENTS.md").exists()
            || std::env::current_dir()
                .map(|cwd| cwd.join("AGENTS.md").exists())
                .unwrap_or(false);
        let has_soul_md = self.config_dir.join("SOUL.md").exists()
            || std::env::current_dir()
                .map(|cwd| cwd.join("SOUL.md").exists())
                .unwrap_or(false);

        let version = self.detect_version();

        OpenClawStatus {
            installed: self.is_installed(),
            config_dir: config_dir_str,
            has_agents_md,
            has_soul_md,
            version,
        }
    }

    pub fn get_agents_content(&self) -> Result<String, String> {
        let agents_path = self.config_dir.join("AGENTS.md");
        if agents_path.exists() {
            return fs::read_to_string(&agents_path)
                .map_err(|e| format!("读取 AGENTS.md 失败: {}", e));
        }

        if let Ok(cwd) = std::env::current_dir() {
            let project_agents = cwd.join("AGENTS.md");
            if project_agents.exists() {
                return fs::read_to_string(&project_agents)
                    .map_err(|e| format!("读取项目 AGENTS.md 失败: {}", e));
            }
        }

        Err("AGENTS.md 未找到".to_string())
    }

    pub fn save_agents_content(&self, content: &str) -> Result<(), String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("创建 OpenClaw 配置目录失败: {}", e))?;
        }

        let agents_path = self.config_dir.join("AGENTS.md");
        fs::write(&agents_path, content)
            .map_err(|e| format!("写入 AGENTS.md 失败: {}", e))
    }

    pub fn get_soul_content(&self) -> Result<String, String> {
        let soul_path = self.config_dir.join("SOUL.md");
        if soul_path.exists() {
            return fs::read_to_string(&soul_path)
                .map_err(|e| format!("读取 SOUL.md 失败: {}", e));
        }
        Err("SOUL.md 未找到".to_string())
    }

    pub fn save_soul_content(&self, content: &str) -> Result<(), String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("创建 OpenClaw 配置目录失败: {}", e))?;
        }

        let soul_path = self.config_dir.join("SOUL.md");
        fs::write(&soul_path, content)
            .map_err(|e| format!("写入 SOUL.md 失败: {}", e))
    }

    /// 应用 Provider 配置到 OpenClaw
    /// OpenClaw 使用环境变量 ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL 或
    /// OPENAI_API_KEY / OPENAI_BASE_URL，通过 .env 文件或 AGENTS.md 中的指令配置
    pub fn apply_provider(
        &self,
        provider: &OpenClawProvider,
    ) -> Result<(), String> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .map_err(|e| format!("创建 OpenClaw 配置目录失败: {}", e))?;
        }

        let env_path = self.config_dir.join(".env");
        let mut env_content = if env_path.exists() {
            fs::read_to_string(&env_path)
                .map_err(|e| format!("读取 .env 失败: {}", e))?
        } else {
            String::new()
        };

        let updates = vec![
            ("OPENAI_API_KEY", provider.api_key.as_deref()),
            ("OPENAI_BASE_URL", Some(provider.base_url.as_str())),
        ];

        for (key, value) in updates {
            if let Some(val) = value {
                let line = format!("{}={}", key, val);
                let pattern = format!("{}=", key);
                if let Some(pos) = env_content.find(&pattern) {
                    let end = env_content[pos..]
                        .find('\n')
                        .map(|i| pos + i)
                        .unwrap_or(env_content.len());
                    env_content.replace_range(pos..end, &line);
                } else {
                    if !env_content.is_empty() && !env_content.ends_with('\n') {
                        env_content.push('\n');
                    }
                    env_content.push_str(&line);
                    env_content.push('\n');
                }
            }
        }

        fs::write(&env_path, &env_content)
            .map_err(|e| format!("写入 .env 失败: {}", e))?;

        tracing::info!(
            "[OpenClaw] Provider 配置已应用: base_url={}",
            provider.base_url
        );

        Ok(())
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("openclaw")
            .arg("--version")
            .output()
            .ok()?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let trimmed = version_str.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        None
    }
}
