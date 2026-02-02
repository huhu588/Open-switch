// 应用设置命令模块
// 处理关闭窗口行为、自动启动等应用设置

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;
use tauri_plugin_autostart::ManagerExt;
use std::sync::{Arc, Mutex};

use crate::config::ConfigManager;
use crate::database::Database;

/// 关闭窗口时的行为
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    /// 每次询问
    Ask,
    /// 最小化到托盘
    Tray,
    /// 直接退出
    Quit,
}

impl Default for CloseAction {
    fn default() -> Self {
        CloseAction::Ask
    }
}

impl std::fmt::Display for CloseAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloseAction::Ask => write!(f, "ask"),
            CloseAction::Tray => write!(f, "tray"),
            CloseAction::Quit => write!(f, "quit"),
        }
    }
}

impl std::str::FromStr for CloseAction {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ask" => Ok(CloseAction::Ask),
            "tray" => Ok(CloseAction::Tray),
            "quit" => Ok(CloseAction::Quit),
            _ => Err(format!("Unknown close action: {}", s)),
        }
    }
}

/// 日志保留策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRetention {
    /// 保留 30 天
    Days30,
    /// 永久保留
    Permanent,
}

impl Default for LogRetention {
    fn default() -> Self {
        LogRetention::Permanent // 默认永久保留
    }
}

impl std::fmt::Display for LogRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogRetention::Days30 => write!(f, "days30"),
            LogRetention::Permanent => write!(f, "permanent"),
        }
    }
}

impl std::str::FromStr for LogRetention {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "days30" | "30days" | "30" => Ok(LogRetention::Days30),
            "permanent" | "forever" | "all" => Ok(LogRetention::Permanent),
            _ => Err(format!("Unknown log retention: {}", s)),
        }
    }
}

/// 应用设置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// 关闭窗口时的行为
    pub close_action: CloseAction,
    /// 日志保留策略
    #[serde(default)]
    pub log_retention: LogRetention,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_action: CloseAction::Ask,
            log_retention: LogRetention::Permanent, // 默认永久保留
        }
    }
}

const SETTINGS_STORE_KEY: &str = "app_settings";

/// 获取应用设置
#[tauri::command]
pub async fn get_app_settings(
    app: tauri::AppHandle,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<AppSettings, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value(value.clone())
            .map_err(|e| e.to_string())
    } else {
        Ok(AppSettings::default())
    }
}

/// 保存应用设置
#[tauri::command]
pub async fn save_app_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_STORE_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 获取关闭行为设置
#[tauri::command]
pub async fn get_close_action(
    app: tauri::AppHandle,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    Ok(settings.close_action.to_string())
}

/// 设置关闭行为
#[tauri::command]
pub async fn set_close_action(
    app: tauri::AppHandle,
    action: String,
    _config: State<'_, Mutex<ConfigManager>>,
) -> Result<(), String> {
    let close_action: CloseAction = action.parse()?;
    
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    // 读取现有设置或创建默认设置
    let mut settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    settings.close_action = close_action;
    
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_STORE_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 获取日志保留设置
#[tauri::command]
pub async fn get_log_retention(
    app: tauri::AppHandle,
) -> Result<String, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    Ok(settings.log_retention.to_string())
}

/// 设置日志保留策略
#[tauri::command]
pub async fn set_log_retention(
    app: tauri::AppHandle,
    retention: String,
) -> Result<(), String> {
    let log_retention: LogRetention = retention.parse()?;
    
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    // 读取现有设置或创建默认设置
    let mut settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).map_err(|e| {
            eprintln!("读取 settings.json 失败: {}", e);
            format!("读取 settings.json 失败: {}", e)
        })?
    } else {
        AppSettings::default()
    };
    
    settings.log_retention = log_retention;
    
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set(SETTINGS_STORE_KEY, value);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 清理过期日志
#[tauri::command]
pub async fn cleanup_old_logs(
    app: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
) -> Result<u32, String> {
    // 获取当前保留设置
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    
    let settings = if let Some(value) = store.get(SETTINGS_STORE_KEY) {
        serde_json::from_value::<AppSettings>(value.clone()).unwrap_or_default()
    } else {
        AppSettings::default()
    };
    
    // 如果是永久保留，不删除任何记录
    if settings.log_retention == LogRetention::Permanent {
        return Ok(0);
    }
    
    // 计算 30 天前的时间戳
    let cutoff_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
        - (30 * 24 * 3600); // 30 天
    
    // 删除过期记录
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let deleted = conn
        .execute(
            "DELETE FROM proxy_request_logs WHERE created_at < ?1",
            rusqlite::params![cutoff_time],
        )
        .map_err(|e| format!("清理过期日志失败: {e}"))?;
    
    Ok(deleted as u32)
}

/// 处理用户的关闭选择（统一由后端处理窗口操作）
#[tauri::command]
pub async fn handle_close_choice(
    app: tauri::AppHandle,
    choice: String,
) -> Result<(), String> {
    match choice.as_str() {
        "tray" => {
            // 隐藏窗口到托盘
            if let Some(window) = app.get_webview_window("main") {
                window.hide().map_err(|e| e.to_string())?;
            }
        }
        "quit" => {
            // 退出应用
            app.exit(0);
        }
        _ => {
            return Err(format!("Unknown choice: {}", choice));
        }
    }
    Ok(())
}

// ============== 自动启动设置 ==============

/// 获取自动启动状态
#[tauri::command]
pub async fn get_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autostart_manager = app.autolaunch();
    autostart_manager.is_enabled().map_err(|e| e.to_string())
}

/// 设置自动启动
#[tauri::command]
pub async fn set_autostart_enabled(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let autostart_manager = app.autolaunch();
    
    if enabled {
        autostart_manager.enable().map_err(|e| e.to_string())?;
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// ============== 环境变量冲突检测 ==============

/// 冲突来源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictSource {
    pub app: String,         // "claude", "codex", "gemini"
    pub value: String,       // 脱敏后的值
    pub config_path: String, // 配置文件路径
}

/// 环境变量冲突
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConflict {
    pub variable: String,
    pub sources: Vec<ConflictSource>,
}

/// 脱敏处理：只显示前4个字符和后4个字符
fn mask_value(value: &str) -> String {
    if value.len() <= 12 {
        "*".repeat(value.len().min(8))
    } else {
        format!("{}...{}", &value[..4], &value[value.len()-4..])
    }
}

/// 刷新托盘菜单（当 Provider 列表变化时调用）
#[tauri::command]
pub fn refresh_tray_menu(app: tauri::AppHandle) -> Result<(), String> {
    crate::refresh_tray_menu(&app);
    Ok(())
}

/// 检测环境变量冲突
#[tauri::command]
pub async fn detect_env_conflicts() -> Result<Vec<EnvConflict>, String> {
    use std::collections::HashMap;
    
    let mut env_map: HashMap<String, Vec<ConflictSource>> = HashMap::new();
    
    // 1. 读取 Claude Code 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let claude_path = std::path::Path::new(&home).join(".claude").join("settings.json");
        if claude_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&claude_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(env) = json.get("env").and_then(|v| v.as_object()) {
                        for (key, value) in env {
                            if let Some(val_str) = value.as_str() {
                                env_map.entry(key.clone()).or_default().push(ConflictSource {
                                    app: "Claude Code".to_string(),
                                    value: mask_value(val_str),
                                    config_path: claude_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 2. 读取 Codex 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let codex_auth_path = std::path::Path::new(&home).join(".codex").join("auth.json");
        if codex_auth_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&codex_auth_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = json.as_object() {
                        for (key, value) in obj {
                            if let Some(val_str) = value.as_str() {
                                env_map.entry(key.clone()).or_default().push(ConflictSource {
                                    app: "Codex".to_string(),
                                    value: mask_value(val_str),
                                    config_path: codex_auth_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // 读取 config.toml 中的 env_key
        let codex_config_path = std::path::Path::new(&home).join(".codex").join("config.toml");
        if codex_config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&codex_config_path) {
                // 简单解析 env_key = "XXX" 行
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("env_key") {
                        if let Some(value) = line.split('=').nth(1) {
                            let key = value.trim().trim_matches('"').trim_matches('\'');
                            if !key.is_empty() {
                                env_map.entry(key.to_string()).or_default().push(ConflictSource {
                                    app: "Codex".to_string(),
                                    value: "(env_key reference)".to_string(),
                                    config_path: codex_config_path.display().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 3. 读取 Gemini 配置
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let gemini_env_path = std::path::Path::new(&home).join(".gemini").join(".env");
        if gemini_env_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gemini_env_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"').trim_matches('\'');
                        env_map.entry(key.to_string()).or_default().push(ConflictSource {
                            app: "Gemini".to_string(),
                            value: mask_value(value),
                            config_path: gemini_env_path.display().to_string(),
                        });
                    }
                }
            }
        }
    }
    
    // 4. 筛选出有冲突的变量（被多个工具使用）
    let conflicts: Vec<EnvConflict> = env_map
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(variable, sources)| EnvConflict { variable, sources })
        .collect();
    
    Ok(conflicts)
}

// ============== 外部工具配置读取 ==============

/// 外部工具服务商项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcSwitchProviderItem {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,  // API Key（用于导入时自动填充）
    pub model_count: i32,
    pub source: String,
    pub tool: Option<String>,
    pub inferred_model_type: Option<String>,
    pub current_model: Option<String>,
}

/// 读取外部工具配置的服务商列表
/// 支持读取 SQLite 数据库和 JSON 配置文件格式
#[tauri::command]
pub async fn get_cc_switch_providers() -> Result<Vec<CcSwitchProviderItem>, String> {
    let mut providers = Vec::new();
    
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "无法获取用户目录".to_string())?;
    
    // 1. 先读取 Ai Switch 自己的配置 (~/.ai-switch/config.json)
    let open_switch_path = std::path::Path::new(&home).join(".ai-switch").join("config.json");
    if open_switch_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&open_switch_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(unified_providers) = json.get("providers").and_then(|v| v.as_object()) {
                    for (_id, provider) in unified_providers {
                        let name = provider.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown")
                            .to_string();
                        
                        let base_url = provider.get("baseUrl")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        
                        // 检查启用的应用
                        let apps = provider.get("apps");
                        let opencode_enabled = apps.and_then(|a| a.get("opencode")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let claude_enabled = apps.and_then(|a| a.get("claude")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let codex_enabled = apps.and_then(|a| a.get("codex")).and_then(|v| v.as_bool()).unwrap_or(false);
                        let gemini_enabled = apps.and_then(|a| a.get("gemini")).and_then(|v| v.as_bool()).unwrap_or(false);
                        
                        // 推断模型类型
                        let inferred_type = if opencode_enabled {
                            "opencode"
                        } else if claude_enabled {
                            "claude"
                        } else if codex_enabled {
                            "codex"
                        } else if gemini_enabled {
                            "gemini"
                        } else {
                            "codex"
                        };
                        
                        // 获取应用信息
                        let mut apps_list = Vec::new();
                        if opencode_enabled { apps_list.push("OpenCode"); }
                        if claude_enabled { apps_list.push("Claude"); }
                        if codex_enabled { apps_list.push("Codex"); }
                        if gemini_enabled { apps_list.push("Gemini"); }
                        let apps_info = apps_list.join(" ");
                        
                        providers.push(CcSwitchProviderItem {
                            name: if apps_info.is_empty() { name } else { format!("{} ({})", name, apps_info) },
                            base_url,
                            api_key: None, // Ai Switch 配置不包含 API key
                            model_count: -1,
                            source: "open_switch".to_string(),
                            tool: Some("open_switch".to_string()),
                            inferred_model_type: Some(inferred_type.to_string()),
                            current_model: None,
                        });
                    }
                }
            }
        }
    }
    
    // 2. 读取外部工具配置
    // 优先读取 SQLite 数据库，然后尝试 JSON 文件
    let db_path = std::path::Path::new(&home).join(".cc-switch").join("cc-switch.db");
    let config_path = std::path::Path::new(&home).join(".cc-switch").join("config.json");
    
    // 2.1 尝试读取 SQLite 数据库
    // 表结构: providers(id, app_type, name, settings_config, website_url, notes, is_current, ...)
    if db_path.exists() {
        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
            // 读取 providers 表 - 使用正确的列名
            if let Ok(mut stmt) = conn.prepare("SELECT id, app_type, name, settings_config, website_url, notes, is_current FROM providers") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0).unwrap_or_default(),  // id
                        row.get::<_, String>(1).unwrap_or_default(),  // app_type (claude/codex/gemini)
                        row.get::<_, String>(2).unwrap_or_default(),  // name
                        row.get::<_, String>(3).unwrap_or_default(),  // settings_config (JSON)
                        row.get::<_, String>(4).unwrap_or_default(),  // website_url
                        row.get::<_, String>(5).unwrap_or_default(),  // notes
                        row.get::<_, i32>(6).unwrap_or(0),            // is_current
                    ))
                }) {
                    for row in rows.flatten() {
                        let (id, app_type, name, settings_config, website_url, _notes, is_current) = row;
                        
                        // 解析 settings_config JSON
                        let config: Option<serde_json::Value> = serde_json::from_str(&settings_config).ok();
                        
                        // 获取 base_url
                        let base_url = config.as_ref()
                            .and_then(|c| c.get("env"))
                            .and_then(|env| {
                                env.get("ANTHROPIC_BASE_URL")
                                    .or_else(|| env.get("GOOGLE_GEMINI_BASE_URL"))
                                    .or_else(|| env.get("OPENAI_BASE_URL"))
                            })
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| website_url.clone());
                        
                        // 获取 API key (尝试多个可能的 key 名称)
                        let api_key = config.as_ref()
                            .and_then(|c| c.get("env"))
                            .and_then(|env| {
                                env.get("ANTHROPIC_AUTH_TOKEN")
                                    .or_else(|| env.get("ANTHROPIC_API_KEY"))
                                    .or_else(|| env.get("GOOGLE_GEMINI_API_KEY"))
                                    .or_else(|| env.get("GEMINI_API_KEY"))
                                    .or_else(|| env.get("OPENAI_API_KEY"))
                            })
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        
                        // 根据 app_type 推断模型类型
                        let inferred_type = match app_type.as_str() {
                            "claude" => "claude",
                            "codex" => "codex",
                            "gemini" => "gemini",
                            _ => "claude" // 默认 Claude
                        };
                        
                        // 显示名称
                        let display_name = format!("{} (cc-switch)", name);
                        
                        providers.push(CcSwitchProviderItem {
                            name: display_name,
                            base_url,
                            api_key,
                            model_count: if is_current == 1 { 1 } else { 0 },
                            source: format!("cc_switch_db_{}", id),
                            tool: Some("cc_switch".to_string()),
                            inferred_model_type: Some(inferred_type.to_string()),
                            current_model: None,
                        });
                    }
                }
            }
        }
        // 如果 SQLite 读取成功，返回结果
        if !providers.is_empty() || !config_path.exists() {
            return Ok(providers);
        }
    }
    
    // 2.2 尝试读取 JSON 配置文件
    if !config_path.exists() {
        // 外部工具未安装或未配置
        return Ok(providers);
    }
    
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取外部工具配置失败: {}", e))?;
    
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析外部工具配置失败: {}", e))?;
    
    // JSON 配置格式:
    // { "claude": { "providers": { ... } }, "codex": { "providers": { ... } }, "gemini": { "providers": { ... } } }
    
    // 读取 claude.providers（Claude Code 的供应商）
    if let Some(claude_providers) = json.get("claude").and_then(|v| v.get("providers")) {
        if let Some(obj) = claude_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                // 从 settingsConfig.env 提取 base_url
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("ANTHROPIC_BASE_URL"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://api.anthropic.com")
                    .to_string();
                
                // 从 settingsConfig.env 提取 api_key
                let api_key = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| {
                        // 尝试多个可能的 key 名称
                        env.get("ANTHROPIC_AUTH_TOKEN")
                            .or_else(|| env.get("ANTHROPIC_API_KEY"))
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                // 从 settingsConfig.env 提取 model
                let model = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("ANTHROPIC_MODEL"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name: format!("{} (cc-switch)", name),
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_claude".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("claude".to_string()),
                    current_model: model,
                });
            }
        }
    }
    
    // 读取 codex.providers（Codex CLI 的供应商）
    if let Some(codex_providers) = json.get("codex").and_then(|v| v.get("providers")) {
        if let Some(obj) = codex_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                // Codex 的 base_url 在 settingsConfig.config 中的 TOML 字符串里
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("config"))
                    .and_then(|v| v.as_str())
                    .and_then(|toml| {
                        for line in toml.lines() {
                            let line = line.trim();
                            if line.starts_with("base_url") {
                                if let Some(url) = line.split('=').nth(1) {
                                    return Some(url.trim().trim_matches('"').to_string());
                                }
                            }
                        }
                        None
                    })
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                
                // Codex 的 API key 可能在 env 或 config 中
                let api_key = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("OPENAI_API_KEY"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name: format!("{} (cc-switch)", name),
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_codex".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("codex".to_string()),
                    current_model: None,
                });
            }
        }
    }
    
    // 读取 gemini.providers（Gemini CLI 的供应商）
    if let Some(gemini_providers) = json.get("gemini").and_then(|v| v.get("providers")) {
        if let Some(obj) = gemini_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("GOOGLE_GEMINI_BASE_URL"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://generativelanguage.googleapis.com")
                    .to_string();
                
                let api_key = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| {
                        env.get("GOOGLE_GEMINI_API_KEY")
                            .or_else(|| env.get("GEMINI_API_KEY"))
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let model = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("GEMINI_MODEL"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name: format!("{} (cc-switch)", name),
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_gemini".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("gemini".to_string()),
                    current_model: model,
                });
            }
        }
    }
    
    // 兼容旧格式：读取 universalProviders（跨应用通用供应商）
    if let Some(universal_providers) = json.get("universalProviders") {
        if let Some(arr) = universal_providers.as_array() {
            for provider in arr {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let base_url = provider.get("baseUrl")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                let apps = provider.get("apps");
                let claude_enabled = apps.and_then(|a| a.get("claude")).and_then(|v| v.as_bool()).unwrap_or(false);
                let codex_enabled = apps.and_then(|a| a.get("codex")).and_then(|v| v.as_bool()).unwrap_or(false);
                let gemini_enabled = apps.and_then(|a| a.get("gemini")).and_then(|v| v.as_bool()).unwrap_or(false);
                
                let inferred_type = if claude_enabled {
                    "claude"
                } else if codex_enabled {
                    "codex"
                } else if gemini_enabled {
                    "gemini"
                } else {
                    "codex"
                };
                
                let apps_info = format!(
                    "{}{}{}",
                    if claude_enabled { "Claude " } else { "" },
                    if codex_enabled { "Codex " } else { "" },
                    if gemini_enabled { "Gemini" } else { "" }
                ).trim().to_string();
                
                let api_key = provider.get("apiKey")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name: format!("{} ({}) (cc-switch)", name, apps_info),
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_universal".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some(inferred_type.to_string()),
                    current_model: None,
                });
            }
        }
    }
    
    // 兼容旧格式：读取 claudeProviders
    if let Some(claude_providers) = json.get("claudeProviders").and_then(|v| v.get("providers")) {
        if let Some(obj) = claude_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("ANTHROPIC_BASE_URL"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://api.anthropic.com")
                    .to_string();
                
                let api_key = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| {
                        env.get("ANTHROPIC_AUTH_TOKEN")
                            .or_else(|| env.get("ANTHROPIC_API_KEY"))
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let model = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("ANTHROPIC_MODEL"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name: format!("{} (cc-switch)", name),
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_claude".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("claude".to_string()),
                    current_model: model,
                });
            }
        }
    }
    
    // 兼容旧格式：读取 codexProviders
    if let Some(codex_providers) = json.get("codexProviders").and_then(|v| v.get("providers")) {
        if let Some(obj) = codex_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("config"))
                    .and_then(|v| v.as_str())
                    .and_then(|toml| {
                        for line in toml.lines() {
                            let line = line.trim();
                            if line.starts_with("base_url") {
                                if let Some(url) = line.split('=').nth(1) {
                                    return Some(url.trim().trim_matches('"').to_string());
                                }
                            }
                        }
                        None
                    })
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                
                providers.push(CcSwitchProviderItem {
                    name,
                    base_url,
                    api_key: None,
                    model_count: 0,
                    source: "cc_switch_codex".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("codex".to_string()),
                    current_model: None,
                });
            }
        }
    }
    
    // 读取 geminiProviders（Gemini CLI 的供应商）
    if let Some(gemini_providers) = json.get("geminiProviders").and_then(|v| v.get("providers")) {
        if let Some(obj) = gemini_providers.as_object() {
            for (_id, provider) in obj {
                let name = provider.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                
                // 从 settingsConfig.env 提取 base_url
                let base_url = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("GOOGLE_GEMINI_BASE_URL"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("https://generativelanguage.googleapis.com")
                    .to_string();
                
                let api_key = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| {
                        env.get("GOOGLE_GEMINI_API_KEY")
                            .or_else(|| env.get("GEMINI_API_KEY"))
                    })
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                let model = provider
                    .get("settingsConfig")
                    .and_then(|sc| sc.get("env"))
                    .and_then(|env| env.get("GEMINI_MODEL"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                
                providers.push(CcSwitchProviderItem {
                    name,
                    base_url,
                    api_key,
                    model_count: 0,
                    source: "cc_switch_gemini".to_string(),
                    tool: Some("cc_switch".to_string()),
                    inferred_model_type: Some("gemini".to_string()),
                    current_model: model,
                });
            }
        }
    }
    
    Ok(providers)
}

/// 删除 cc-switch 服务商
/// 支持从 SQLite 数据库和 JSON 配置文件中删除
#[tauri::command]
pub async fn delete_cc_switch_provider(name: String, source: String) -> Result<(), String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "无法获取用户目录".to_string())?;
    
    // 从名称中提取原始名称
    // 名称可能的格式：
    // - "Name (cc-switch)" -> "Name"
    // - "Name (cc-switch) (App1 App2)" -> "Name"
    // - "Name (App1) (cc-switch)" -> "Name"
    let original_name = {
        let mut n = name.clone();
        // 先去掉末尾的应用标签，如 " (OpenCode Claude)"
        if let Some(pos) = n.rfind(" (cc-switch)") {
            // 如果 "(cc-switch)" 后面还有内容，说明有应用标签
            let after_cc = &n[pos + 12..];
            if !after_cc.is_empty() {
                // 去掉应用标签
                n = n[..pos + 12].to_string();
            }
        }
        // 去掉 " (cc-switch)" 后缀
        n.trim_end_matches(" (cc-switch)").to_string()
    };
    
    // 如果来源是 SQLite 数据库
    if source.starts_with("cc_switch_db_") {
        let db_id = source.strip_prefix("cc_switch_db_").unwrap_or("");
        let db_path = std::path::Path::new(&home).join(".cc-switch").join("cc-switch.db");
        
        if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("打开数据库失败: {}", e))?;
            
            // 尝试通过 ID 删除
            let deleted = conn.execute(
                "DELETE FROM providers WHERE id = ?1",
                rusqlite::params![db_id],
            ).map_err(|e| format!("删除失败: {}", e))?;
            
            if deleted == 0 {
                // 如果通过 ID 删除失败，尝试通过名称删除
                conn.execute(
                    "DELETE FROM providers WHERE name = ?1",
                    rusqlite::params![original_name],
                ).map_err(|e| format!("删除失败: {}", e))?;
            }
            
            return Ok(());
        }
    }
    
    // 如果来源是 JSON 配置文件
    let config_path = std::path::Path::new(&home).join(".cc-switch").join("config.json");
    
    if !config_path.exists() {
        return Err("cc-switch 配置文件不存在".to_string());
    }
    
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    let mut deleted = false;
    
    // 根据来源确定要删除的位置
    let sections = match source.as_str() {
        "cc_switch_claude" => vec![("claude", "providers")],
        "cc_switch_codex" => vec![("codex", "providers"), ("codexProviders", "providers")],
        "cc_switch_gemini" => vec![("gemini", "providers"), ("geminiProviders", "providers")],
        "cc_switch_universal" => vec![
            ("claude", "providers"),
            ("codex", "providers"),
            ("gemini", "providers"),
            ("codexProviders", "providers"),
            ("geminiProviders", "providers"),
        ],
        _ => vec![
            ("claude", "providers"),
            ("codex", "providers"),
            ("gemini", "providers"),
            ("codexProviders", "providers"),
            ("geminiProviders", "providers"),
        ],
    };
    
    for (section, key) in sections {
        if let Some(providers) = json.get_mut(section)
            .and_then(|v| v.get_mut(key))
            .and_then(|v| v.as_object_mut())
        {
            // 找到并删除匹配名称的服务商
            let keys_to_remove: Vec<String> = providers.iter()
                .filter(|(_k, v)| {
                    v.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n == original_name)
                        .unwrap_or(false)
                })
                .map(|(k, _)| k.clone())
                .collect();
            
            for key in keys_to_remove {
                providers.remove(&key);
                deleted = true;
            }
        }
    }
    
    if !deleted {
        return Err(format!("未找到名为 '{}' 的服务商", original_name));
    }
    
    // 写回配置文件
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    Ok(())
}

/// 删除 Ai Switch 统一配置中的服务商
#[tauri::command]
pub async fn delete_open_switch_provider(name: String) -> Result<(), String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "无法获取用户目录".to_string())?;
    
    let config_path = std::path::Path::new(&home).join(".ai-switch").join("config.json");
    
    if !config_path.exists() {
        return Err("Ai Switch 配置文件不存在".to_string());
    }
    
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    // 从名称中提取原始名称（去掉应用标签后缀，如 "(OpenCode Claude Codex Gemini)"）
    let original_name = if let Some(pos) = name.rfind(" (") {
        name[..pos].to_string()
    } else {
        name.clone()
    };
    
    let providers = json.get_mut("providers")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| "配置文件中不存在 providers 字段".to_string())?;
    
    // 找到并删除匹配名称的服务商
    let keys_to_remove: Vec<String> = providers.iter()
        .filter(|(_k, v)| {
            v.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n == original_name)
                .unwrap_or(false)
        })
        .map(|(k, _)| k.clone())
        .collect();
    
    if keys_to_remove.is_empty() {
        return Err(format!("未找到名为 '{}' 的服务商", original_name));
    }
    
    for key in keys_to_remove {
        providers.remove(&key);
    }
    
    // 写回配置文件
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    Ok(())
}
