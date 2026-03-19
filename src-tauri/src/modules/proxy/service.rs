//! 代理服务
//!
//! 提供代理服务器的启动、停止和配置接管管理

use super::{ProxyConfig, ProxyServer, ProxyServerInfo, ProxyStatus, ProxyTakeoverStatus};
use crate::modules::opencode_db::Database;
use crate::opencode_error::AppError;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 代理接管模式下的占位符 Token
const PROXY_TOKEN_PLACEHOLDER: &str = "PROXY_MANAGED";

/// 代理服务
pub struct ProxyService {
    db: Arc<Database>,
    server: Arc<RwLock<Option<ProxyServer>>>,
}

impl ProxyService {
    /// 创建代理服务
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            server: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动代理服务器
    pub async fn start(&self) -> Result<ProxyServerInfo, AppError> {
        let config_db = self.db.get_proxy_config()?;
        
        let config = ProxyConfig {
            listen_address: config_db.listen_address.clone(),
            listen_port: config_db.listen_port,
            enable_logging: true,
        };

        let server = ProxyServer::new(config, self.db.clone());
        let info = server.start().await?;
        
        *self.server.write().await = Some(server);
        
        // 更新数据库中的代理启用状态
        let mut updated_config = config_db;
        updated_config.proxy_enabled = true;
        self.db.update_proxy_config(&updated_config)?;
        
        Ok(info)
    }

    /// 停止代理服务器
    pub async fn stop(&self) -> Result<(), AppError> {
        if let Some(server) = self.server.write().await.take() {
            server.stop().await?;
        }
        
        // 更新数据库中的代理启用状态
        let mut config = self.db.get_proxy_config()?;
        config.proxy_enabled = false;
        self.db.update_proxy_config(&config)?;
        
        Ok(())
    }

    /// 获取代理状态
    pub async fn get_status(&self) -> ProxyStatus {
        if let Some(server) = self.server.read().await.as_ref() {
            server.get_status().await
        } else {
            ProxyStatus::default()
        }
    }

    /// 检查是否运行中
    pub async fn is_running(&self) -> bool {
        if let Some(server) = self.server.read().await.as_ref() {
            server.is_running().await
        } else {
            false
        }
    }

    /// 启动代理并接管配置
    pub async fn start_with_takeover(&self, apps: &[&str]) -> Result<ProxyServerInfo, AppError> {
        // 1. 备份各应用的配置
        for app in apps {
            self.backup_live_config(app)?;
        }

        // 2. 启动代理服务器
        let info = self.start().await?;

        // 3. 接管各应用的配置
        let proxy_url = format!("http://{}:{}", info.address, info.port);
        for app in apps {
            if let Err(e) = self.takeover_live_config(app, &proxy_url) {
                // 接管失败，恢复配置
                for app in apps {
                    let _ = self.restore_live_config(app);
                }
                let _ = self.stop().await;
                return Err(e);
            }
        }

        // 4. 更新接管状态
        let mut config = self.db.get_proxy_config()?;
        for app in apps {
            match *app {
                "claude" => config.takeover_claude = true,
                "codex" => config.takeover_codex = true,
                "gemini" => config.takeover_gemini = true,
                _ => {}
            }
        }
        self.db.update_proxy_config(&config)?;

        Ok(info)
    }

    /// 停止代理并恢复配置
    pub async fn stop_with_restore(&self) -> Result<(), AppError> {
        let config = self.db.get_proxy_config()?;

        // 恢复各应用的配置
        if config.takeover_claude {
            let _ = self.restore_live_config("claude");
        }
        if config.takeover_codex {
            let _ = self.restore_live_config("codex");
        }
        if config.takeover_gemini {
            let _ = self.restore_live_config("gemini");
        }

        // 停止代理服务器
        self.stop().await?;

        // 清除接管状态
        let mut updated_config = self.db.get_proxy_config()?;
        updated_config.takeover_claude = false;
        updated_config.takeover_codex = false;
        updated_config.takeover_gemini = false;
        self.db.update_proxy_config(&updated_config)?;

        // 删除备份
        let _ = self.db.delete_live_backup("claude");
        let _ = self.db.delete_live_backup("codex");
        let _ = self.db.delete_live_backup("gemini");

        Ok(())
    }

    /// 获取接管状态
    pub fn get_takeover_status(&self) -> Result<ProxyTakeoverStatus, AppError> {
        let config = self.db.get_proxy_config()?;
        Ok(ProxyTakeoverStatus {
            claude: config.takeover_claude,
            codex: config.takeover_codex,
            gemini: config.takeover_gemini,
        })
    }

    /// 为指定应用开启/关闭接管
    pub async fn set_takeover_for_app(&self, app_type: &str, enabled: bool) -> Result<(), AppError> {
        let mut config = self.db.get_proxy_config()?;

        if enabled {
            // 确保代理服务器正在运行
            if !self.is_running().await {
                self.start().await?;
            }

            // 备份并接管配置
            self.backup_live_config(app_type)?;
            
            let status = self.get_status().await;
            let proxy_url = format!("http://{}:{}", status.address, status.port);
            self.takeover_live_config(app_type, &proxy_url)?;

            // 更新接管状态
            match app_type {
                "claude" => config.takeover_claude = true,
                "codex" => config.takeover_codex = true,
                "gemini" => config.takeover_gemini = true,
                _ => {}
            }
        } else {
            // 恢复配置
            let _ = self.restore_live_config(app_type);
            let _ = self.db.delete_live_backup(app_type);

            // 更新接管状态
            match app_type {
                "claude" => config.takeover_claude = false,
                "codex" => config.takeover_codex = false,
                "gemini" => config.takeover_gemini = false,
                _ => {}
            }

            // 如果没有任何应用被接管，停止代理
            if !config.takeover_claude && !config.takeover_codex && !config.takeover_gemini {
                let _ = self.stop().await;
            }
        }

        self.db.update_proxy_config(&config)?;
        Ok(())
    }

    // ==================== 配置备份/接管/恢复 ====================

    /// 备份应用的配置
    fn backup_live_config(&self, app_type: &str) -> Result<(), AppError> {
        let config = match app_type {
            "claude" => self.read_claude_live()?,
            "codex" => self.read_codex_live()?,
            "gemini" => self.read_gemini_live()?,
            _ => return Err(AppError::Proxy(format!("未知的应用类型: {app_type}"))),
        };

        let json_str = serde_json::to_string(&config)
            .map_err(|e| AppError::Proxy(format!("序列化配置失败: {e}")))?;
        
        self.db.save_live_backup(app_type, &json_str)?;
        Ok(())
    }

    /// 接管应用的配置
    fn takeover_live_config(&self, app_type: &str, proxy_url: &str) -> Result<(), AppError> {
        match app_type {
            "claude" => self.takeover_claude_config(proxy_url),
            "codex" => self.takeover_codex_config(proxy_url),
            "gemini" => self.takeover_gemini_config(proxy_url),
            _ => Err(AppError::Proxy(format!("未知的应用类型: {app_type}"))),
        }
    }

    /// 恢复应用的配置
    fn restore_live_config(&self, app_type: &str) -> Result<(), AppError> {
        let backup = self.db.get_live_backup(app_type)?;
        
        if let Some(backup_str) = backup {
            let config: Value = serde_json::from_str(&backup_str)
                .map_err(|e| AppError::Proxy(format!("解析备份失败: {e}")))?;
            
            match app_type {
                "claude" => self.write_claude_live(&config)?,
                "codex" => self.write_codex_live(&config)?,
                "gemini" => self.write_gemini_live(&config)?,
                _ => return Err(AppError::Proxy(format!("未知的应用类型: {app_type}"))),
            }
        }
        
        Ok(())
    }

    // ==================== Claude 配置处理 ====================

    fn read_claude_live(&self) -> Result<Value, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        let path = home.join(".claude").join("settings.json");
        
        if !path.exists() {
            return Ok(json!({}));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Proxy(format!("读取 Claude 配置失败: {e}")))?;
        
        serde_json::from_str(&content)
            .map_err(|e| AppError::Proxy(format!("解析 Claude 配置失败: {e}")))
    }

    fn write_claude_live(&self, config: &Value) -> Result<(), AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        let dir = home.join(".claude");
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Proxy(format!("创建目录失败: {e}")))?;
        
        let path = dir.join("settings.json");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::Proxy(format!("序列化配置失败: {e}")))?;
        
        std::fs::write(&path, content)
            .map_err(|e| AppError::Proxy(format!("写入 Claude 配置失败: {e}")))
    }

    fn takeover_claude_config(&self, proxy_url: &str) -> Result<(), AppError> {
        let mut config = self.read_claude_live()?;
        
        let env = config.get_mut("env")
            .and_then(|v| v.as_object_mut())
            .map(|obj| obj.clone())
            .unwrap_or_default();
        
        let mut new_env = env.clone();
        new_env.insert("ANTHROPIC_BASE_URL".to_string(), json!(proxy_url));
        
        // 使用占位符替换 Token
        for key in ["ANTHROPIC_AUTH_TOKEN", "ANTHROPIC_API_KEY"] {
            if new_env.contains_key(key) {
                new_env.insert(key.to_string(), json!(PROXY_TOKEN_PLACEHOLDER));
            }
        }
        
        if let Some(obj) = config.as_object_mut() {
            obj.insert("env".to_string(), json!(new_env));
        } else {
            config = json!({ "env": new_env });
        }
        
        self.write_claude_live(&config)
    }

    // ==================== Codex 配置处理 ====================

    fn read_codex_live(&self) -> Result<Value, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        
        let auth_path = home.join(".codex").join("auth.json");
        let config_path = home.join(".codex").join("config.toml");
        
        let auth: Value = if auth_path.exists() {
            let content = std::fs::read_to_string(&auth_path)
                .map_err(|e| AppError::Proxy(format!("读取 Codex auth 失败: {e}")))?;
            serde_json::from_str(&content).unwrap_or(json!({}))
        } else {
            json!({})
        };
        
        let config_str = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .map_err(|e| AppError::Proxy(format!("读取 Codex config 失败: {e}")))?
        } else {
            String::new()
        };
        
        Ok(json!({
            "auth": auth,
            "config": config_str
        }))
    }

    fn write_codex_live(&self, config: &Value) -> Result<(), AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        let dir = home.join(".codex");
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Proxy(format!("创建目录失败: {e}")))?;
        
        if let Some(auth) = config.get("auth") {
            let auth_path = dir.join("auth.json");
            let content = serde_json::to_string_pretty(auth)
                .map_err(|e| AppError::Proxy(format!("序列化 auth 失败: {e}")))?;
            std::fs::write(&auth_path, content)
                .map_err(|e| AppError::Proxy(format!("写入 Codex auth 失败: {e}")))?;
        }
        
        if let Some(config_str) = config.get("config").and_then(|v| v.as_str()) {
            let config_path = dir.join("config.toml");
            std::fs::write(&config_path, config_str)
                .map_err(|e| AppError::Proxy(format!("写入 Codex config 失败: {e}")))?;
        }
        
        Ok(())
    }

    fn takeover_codex_config(&self, proxy_url: &str) -> Result<(), AppError> {
        let mut config = self.read_codex_live()?;
        
        // 修改 auth.json
        if let Some(auth) = config.get_mut("auth").and_then(|v| v.as_object_mut()) {
            auth.insert("OPENAI_API_KEY".to_string(), json!(PROXY_TOKEN_PLACEHOLDER));
        }
        
        // 修改 config.toml 中的 base_url
        if let Some(config_str) = config.get("config").and_then(|v| v.as_str()) {
            let new_config = self.update_codex_toml_base_url(config_str, &format!("{}/v1", proxy_url));
            config["config"] = json!(new_config);
        }
        
        self.write_codex_live(&config)
    }

    fn update_codex_toml_base_url(&self, toml_str: &str, new_url: &str) -> String {
        // 简单的 TOML 处理：查找或添加 base_url
        let mut lines: Vec<String> = toml_str.lines().map(|s| s.to_string()).collect();
        let mut found = false;
        
        for line in &mut lines {
            if line.trim().starts_with("base_url") {
                *line = format!("base_url = \"{}\"", new_url);
                found = true;
                break;
            }
        }
        
        if !found {
            lines.push(format!("base_url = \"{}\"", new_url));
        }
        
        lines.join("\n")
    }

    // ==================== Gemini 配置处理 ====================

    fn read_gemini_live(&self) -> Result<Value, AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        let env_path = home.join(".gemini").join(".env");
        
        if !env_path.exists() {
            return Ok(json!({ "env": {} }));
        }
        
        let content = std::fs::read_to_string(&env_path)
            .map_err(|e| AppError::Proxy(format!("读取 Gemini .env 失败: {e}")))?;
        
        let mut env_map = serde_json::Map::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                env_map.insert(key.trim().to_string(), json!(value.trim().trim_matches('"')));
            }
        }
        
        Ok(json!({ "env": env_map }))
    }

    fn write_gemini_live(&self, config: &Value) -> Result<(), AppError> {
        let home = dirs::home_dir()
            .ok_or_else(|| AppError::Proxy("无法获取用户目录".to_string()))?;
        let dir = home.join(".gemini");
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Proxy(format!("创建目录失败: {e}")))?;
        
        if let Some(env) = config.get("env").and_then(|v| v.as_object()) {
            let mut lines = Vec::new();
            for (key, value) in env {
                if let Some(v) = value.as_str() {
                    lines.push(format!("{}=\"{}\"", key, v));
                }
            }
            
            let env_path = dir.join(".env");
            std::fs::write(&env_path, lines.join("\n"))
                .map_err(|e| AppError::Proxy(format!("写入 Gemini .env 失败: {e}")))?;
        }
        
        Ok(())
    }

    fn takeover_gemini_config(&self, proxy_url: &str) -> Result<(), AppError> {
        let mut config = self.read_gemini_live()?;
        
        if let Some(env) = config.get_mut("env").and_then(|v| v.as_object_mut()) {
            env.insert("GOOGLE_GEMINI_BASE_URL".to_string(), json!(proxy_url));
            env.insert("GEMINI_API_KEY".to_string(), json!(PROXY_TOKEN_PLACEHOLDER));
        } else {
            config["env"] = json!({
                "GOOGLE_GEMINI_BASE_URL": proxy_url,
                "GEMINI_API_KEY": PROXY_TOKEN_PLACEHOLDER
            });
        }
        
        self.write_gemini_live(&config)
    }
}
