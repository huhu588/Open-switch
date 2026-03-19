
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

use crate::modules::opencode_config::{ConfigManager, McpServer, McpServerType};
use crate::modules::opencode_config::codex_manager::CodexConfigManager;
use crate::modules::opencode_config::gemini_manager::GeminiConfigManager;
use crate::opencode_error::AppError;
use super::model::build_variants;

const BACKUP_VERSION: &str = "1.2.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedProvider {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub npm: Option<String>,
    pub description: Option<String>,
    pub model_type: Option<String>,
    pub enabled: bool,
    pub models: Vec<ExportedModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedModel {
    pub id: String,
    pub name: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedOAuthConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedMcpServer {
    pub name: String,
    pub server_type: String,
    pub enabled: bool,
    pub timeout: Option<u32>,
    pub command: Option<Vec<String>>,
    pub environment: Option<HashMap<String, String>>,
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<ExportedOAuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedRule {
    pub name: String,
    pub location: String,
    pub rule_type: String,
    pub content: String,
    #[serde(default = "default_file_ext")]
    pub file_ext: String,
}

fn default_file_ext() -> String {
    "md".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSkills {
    pub name: String,
    pub location: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCodexProvider {
    pub name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_openai_auth: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCodexMcpServer {
    pub name: String,
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedCodexConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_providers: Vec<ExportedCodexProvider>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<ExportedCodexMcpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedGeminiEnv {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_gemini_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_gemini_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedGeminiMcpServer {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportedGeminiConfig {
    #[serde(default)]
    pub env: ExportedGeminiEnv,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<ExportedGeminiMcpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedUsageRecord {
    pub session_id: String,
    pub timestamp: i64,
    pub model: String,
    pub source: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_read_tokens: u32,
    #[serde(default)]
    pub cache_creation_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedChatConversation {
    pub messages: Vec<ExportedChatMessage>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedDevEnv {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub version: String,
    pub created_at: String,
    pub app_name: String,
    pub providers: Vec<ExportedProvider>,
    pub mcp_servers: Vec<ExportedMcpServer>,
    pub rules: Vec<ExportedRule>,
    pub skills: Vec<ExportedSkills>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_config: Option<ExportedCodexConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gemini_config: Option<ExportedGeminiConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_stats: Option<Vec<ExportedUsageRecord>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_conversations: Option<Vec<ExportedChatConversation>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev_envs: Option<Vec<ExportedDevEnv>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportStats {
    pub providers: usize,
    pub models: usize,
    pub mcp_servers: usize,
    pub rules: usize,
    pub skills: usize,
    pub codex_providers: usize,
    pub codex_mcp_servers: usize,
    pub gemini_configured: bool,
    pub gemini_mcp_servers: usize,
    #[serde(default)]
    pub usage_records: usize,
    #[serde(default)]
    pub chat_conversations: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExportOptions {
    #[serde(default = "default_true")]
    pub include_providers: bool,
    #[serde(default = "default_true")]
    pub include_mcp: bool,
    #[serde(default = "default_true")]
    pub include_rules: bool,
    #[serde(default = "default_true")]
    pub include_skills: bool,
    #[serde(default = "default_true")]
    pub include_codex: bool,
    #[serde(default = "default_true")]
    pub include_gemini: bool,
    #[serde(default)]
    pub include_usage_stats: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
pub struct FilteredExportOptions {
    #[serde(default)]
    pub provider_names: Vec<String>,
    #[serde(default)]
    pub mcp_names: Vec<String>,
    #[serde(default)]
    pub rule_ids: Vec<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub codex_provider_names: Vec<String>,
    #[serde(default)]
    pub codex_mcp_names: Vec<String>,
    #[serde(default)]
    pub include_gemini_env: bool,
    #[serde(default)]
    pub gemini_mcp_names: Vec<String>,
    #[serde(default)]
    pub usage_sources: Vec<String>,
    #[serde(default)]
    pub dev_envs: Vec<ExportedDevEnv>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportOptions {
    pub import_providers: bool,
    pub import_mcp: bool,
    pub import_rules: bool,
    pub import_skills: bool,
    pub overwrite_existing: bool,
    #[serde(default)]
    pub import_codex: bool,
    #[serde(default)]
    pub import_gemini: bool,
    #[serde(default)]
    pub import_usage_stats: bool,
    #[serde(default)]
    pub import_chat_conversations: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub providers_imported: usize,
    pub providers_skipped: usize,
    pub mcp_imported: usize,
    pub mcp_skipped: usize,
    pub rules_imported: usize,
    pub rules_skipped: usize,
    pub skills_imported: usize,
    pub skills_skipped: usize,
    pub codex_imported: usize,
    pub codex_skipped: usize,
    pub gemini_imported: usize,
    pub gemini_skipped: usize,
    #[serde(default)]
    pub usage_imported: usize,
    #[serde(default)]
    pub usage_skipped: usize,
    #[serde(default)]
    pub chat_conversations_imported: usize,
    #[serde(default)]
    pub chat_conversations_skipped: usize,
    pub errors: Vec<String>,
}

fn get_skills_paths() -> Vec<(PathBuf, String)> {
    let mut paths = Vec::new();
    if let Some(home_dir) = dirs::home_dir() {
        paths.push((
            home_dir.join(".config").join("opencode").join("skills"),
            "global_opencode".to_string(),
        ));
        paths.push((
            home_dir.join(".claude").join("skills"),
            "global_claude".to_string(),
        ));
        paths.push((
            home_dir.join(".codex").join("skills"),
            "global_codex".to_string(),
        ));
        paths.push((
            home_dir.join(".gemini").join("skills"),
            "global_gemini".to_string(),
        ));
    }
    paths
}

fn get_rule_paths() -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();
    if let Some(home) = dirs::home_dir() {
        paths.insert("global_opencode".to_string(), home.join(".config").join("opencode"));
        paths.insert("global_claude".to_string(), home.join(".claude"));
        paths.insert("global_codex".to_string(), home.join(".codex"));
        paths.insert("global_gemini".to_string(), home.join(".gemini"));
    }
    paths
}

fn create_backup_internal(manager: &ConfigManager) -> Result<BackupData, AppError> {
    let providers_map = manager.opencode().get_all_providers()?;
    let mut providers: Vec<ExportedProvider> = Vec::new();
    
    for (name, provider) in providers_map {
        let models: Vec<ExportedModel> = provider.models
            .iter()
            .map(|(id, info)| ExportedModel {
                id: id.clone(),
                name: info.name.clone(),
                reasoning_effort: info.reasoning_effort.clone(),
            })
            .collect();
        
        providers.push(ExportedProvider {
            name,
            base_url: provider.options.base_url.clone(),
            api_key: provider.options.api_key.clone(),
            npm: provider.npm.clone(),
            description: provider.metadata.description.clone(),
            model_type: provider.model_type.clone(),
            enabled: provider.enabled,
            models,
        });
    }
    
    let mcp_config = manager.mcp().read_config()?;
    let mcp_servers: Vec<ExportedMcpServer> = mcp_config.servers
        .iter()
        .map(|(name, server)| {
            let oauth = server.oauth.as_ref().map(|o| ExportedOAuthConfig {
                client_id: o.client_id.clone(),
                client_secret: o.client_secret.clone(),
                scope: o.scope.clone(),
            });
            
            ExportedMcpServer {
                name: name.clone(),
                server_type: server.server_type.to_string(),
                enabled: server.enabled,
                timeout: server.timeout,
                command: server.command.clone(),
                environment: if server.environment.is_empty() { None } else { Some(server.environment.clone()) },
                url: server.url.clone(),
                headers: if server.headers.is_empty() { None } else { Some(server.headers.clone()) },
                oauth,
            }
        })
        .collect();
    
    let mut rules: Vec<ExportedRule> = Vec::new();
    let rule_paths = get_rule_paths();
    
    for (location_key, base_path) in &rule_paths {
        if location_key == "global_opencode" || location_key == "global_codex" {
            let agents_path = base_path.join("AGENTS.md");
            if agents_path.exists() {
                if let Ok(content) = fs::read_to_string(&agents_path) {
                    rules.push(ExportedRule {
                        name: "AGENTS.md".to_string(),
                        location: location_key.clone(),
                        rule_type: "agents_md".to_string(),
                        content,
                        file_ext: "md".to_string(),
                    });
                }
            }
        }
        
        if location_key == "global_gemini" {
            let gemini_md_path = base_path.join("GEMINI.md");
            if gemini_md_path.exists() {
                if let Ok(content) = fs::read_to_string(&gemini_md_path) {
                    rules.push(ExportedRule {
                        name: "GEMINI.md".to_string(),
                        location: location_key.clone(),
                        rule_type: "gemini_md".to_string(),
                        content,
                        file_ext: "md".to_string(),
                    });
                }
            }
        }
        
        let rules_dir = base_path.join("rules");
        if rules_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&rules_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("md");
                        if ext == "md" || ext == "mdc" {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let name = path.file_stem()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                rules.push(ExportedRule {
                                    name,
                                    location: location_key.clone(),
                                    rule_type: "rules_dir".to_string(),
                                    content,
                                    file_ext: ext.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut skills: Vec<ExportedSkills> = Vec::new();
    for (base_path, location) in get_skills_paths() {
        if !base_path.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skills_file = path.join("SKILL.md");
                    if skills_file.exists() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        if let Ok(content) = fs::read_to_string(&skills_file) {
                            skills.push(ExportedSkills {
                                name,
                                location: location.clone(),
                                content,
                            });
                        }
                    }
                }
            }
        }
    }
    
    let codex_config = read_codex_config();
    
    let gemini_config = read_gemini_config();
    
    let chat_conversations = read_migrated_conversations_for_backup();

    Ok(BackupData {
        version: BACKUP_VERSION.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        app_name: "Ai Switch".to_string(),
        providers,
        mcp_servers,
        rules,
        skills,
        codex_config,
        gemini_config,
        usage_stats: None,
        chat_conversations,
        dev_envs: None,
    })
}

fn read_codex_config() -> Option<ExportedCodexConfig> {
    let codex_manager = CodexConfigManager::new().ok()?;
    
    let model_providers: Vec<ExportedCodexProvider> = codex_manager
        .get_model_providers()
        .ok()
        .map(|providers| {
            providers
                .into_iter()
                .map(|(name, provider)| ExportedCodexProvider {
                    name,
                    base_url: provider.base_url,
                    env_key: provider.env_key,
                    requires_openai_auth: provider.requires_openai_auth,
                })
                .collect()
        })
        .unwrap_or_default();
    
    let mcp_servers: Vec<ExportedCodexMcpServer> = codex_manager
        .get_mcp_servers()
        .ok()
        .map(|servers| {
            servers
                .into_iter()
                .map(|(name, server)| ExportedCodexMcpServer {
                    name,
                    command: server.command,
                    env: server.env,
                })
                .collect()
        })
        .unwrap_or_default();
    
    if model_providers.is_empty() && mcp_servers.is_empty() {
        return None;
    }
    
    Some(ExportedCodexConfig {
        model_providers,
        mcp_servers,
    })
}

fn read_gemini_config() -> Option<ExportedGeminiConfig> {
    let gemini_manager = GeminiConfigManager::new().ok()?;
    
    let env = gemini_manager.read_env().ok()
        .map(|env_config| ExportedGeminiEnv {
            gemini_api_key: env_config.gemini_api_key,
            google_gemini_api_key: env_config.google_gemini_api_key,
            google_gemini_base_url: env_config.google_gemini_base_url,
            gemini_model: env_config.gemini_model,
        })
        .unwrap_or_default();
    
    let mcp_servers: Vec<ExportedGeminiMcpServer> = gemini_manager
        .get_mcp_servers()
        .ok()
        .map(|servers| {
            servers
                .into_iter()
                .map(|(name, server)| ExportedGeminiMcpServer {
                    name,
                    command: server.command,
                    args: server.args,
                    env: server.env,
                    url: server.url,
                })
                .collect()
        })
        .unwrap_or_default();
    
    let has_env = env.gemini_api_key.is_some() 
        || env.google_gemini_api_key.is_some() 
        || env.google_gemini_base_url.is_some()
        || env.gemini_model.is_some();
    
    if !has_env && mcp_servers.is_empty() {
        return None;
    }
    
    Some(ExportedGeminiConfig {
        env,
        mcp_servers,
    })
}

#[tauri::command]
pub fn create_backup(
    config_manager: State<'_, Mutex<ConfigManager>>,
) -> Result<BackupData, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    create_backup_internal(&manager)
}

#[tauri::command]
pub fn export_backup(
    file_path: String,
    options: Option<ExportOptions>,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::modules::opencode_db::Database>>,
) -> Result<ExportStats, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut backup = create_backup_internal(&manager)?;

    let opts = options.unwrap_or(ExportOptions {
        include_providers: true, include_mcp: true, include_rules: true,
        include_skills: true, include_codex: true, include_gemini: true, include_usage_stats: false,
    });

    if !opts.include_providers { backup.providers.clear(); }
    if !opts.include_mcp { backup.mcp_servers.clear(); }
    if !opts.include_rules { backup.rules.clear(); }
    if !opts.include_skills { backup.skills.clear(); }
    if !opts.include_codex { backup.codex_config = None; }
    if !opts.include_gemini { backup.gemini_config = None; }

    if opts.include_usage_stats {
        let conn = db.conn.lock().map_err(|e| AppError::Custom(format!("DB lock failed: {}", e)))?;
        let mut usage_records = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost FROM proxy_request_logs ORDER BY created_at"
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(ExportedUsageRecord {
                    session_id: row.get::<_, String>(0).unwrap_or_default(),
                    timestamp: row.get::<_, i64>(1).unwrap_or(0),
                    model: row.get::<_, String>(2).unwrap_or_default(),
                    source: row.get::<_, String>(3).unwrap_or_default(),
                    input_tokens: row.get::<_, u32>(4).unwrap_or(0),
                    output_tokens: row.get::<_, u32>(5).unwrap_or(0),
                    cache_read_tokens: row.get::<_, u32>(6).unwrap_or(0),
                    cache_creation_tokens: row.get::<_, u32>(7).unwrap_or(0),
                    cost: row.get::<_, f64>(8).ok(),
                })
            }) {
                for r in rows.flatten() { usage_records.push(r); }
            }
        }
        backup.usage_stats = Some(usage_records);
    }

    let stats = ExportStats {
        providers: backup.providers.len(),
        models: backup.providers.iter().map(|p| p.models.len()).sum(),
        mcp_servers: backup.mcp_servers.len(),
        rules: backup.rules.len(),
        skills: backup.skills.len(),
        codex_providers: backup.codex_config.as_ref().map(|c| c.model_providers.len()).unwrap_or(0),
        codex_mcp_servers: backup.codex_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        gemini_configured: backup.gemini_config.is_some(),
        gemini_mcp_servers: backup.gemini_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        usage_records: backup.usage_stats.as_ref().map(|r| r.len()).unwrap_or(0),
        chat_conversations: 0,
    };
    
    let content = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Custom(format!("Failed to serialize: {}", e)))?;
    
    fs::write(&file_path, content)
        .map_err(|e| AppError::Custom(format!("Failed to write file: {}", e)))?;
    
    Ok(stats)
}

#[tauri::command]
pub fn export_backup_filtered(
    file_path: String,
    options: FilteredExportOptions,
    chat_conversations: Option<Vec<ExportedChatConversation>>,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::modules::opencode_db::Database>>,
) -> Result<ExportStats, AppError> {
    let manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    let mut backup = create_backup_internal(&manager)?;

    backup.providers.retain(|p| options.provider_names.contains(&p.name));
    
    backup.mcp_servers.retain(|m| options.mcp_names.contains(&m.name));
    
    backup.rules.retain(|r| {
        let id = format!("{}|{}", r.name, r.location);
        options.rule_ids.contains(&id)
    });
    
    backup.skills.retain(|s| {
        let id = format!("{}|{}", s.name, s.location);
        options.skill_ids.contains(&id)
    });
    
    if let Some(ref mut codex) = backup.codex_config {
        codex.model_providers.retain(|p| options.codex_provider_names.contains(&p.name));
        codex.mcp_servers.retain(|m| options.codex_mcp_names.contains(&m.name));
        if codex.model_providers.is_empty() && codex.mcp_servers.is_empty() {
            backup.codex_config = None;
        }
    }
    if let Some(ref mut gemini) = backup.gemini_config {
        if !options.include_gemini_env {
            gemini.env = ExportedGeminiEnv {
                gemini_api_key: None,
                google_gemini_api_key: None,
                google_gemini_base_url: None,
                gemini_model: None,
            };
        }
        gemini.mcp_servers.retain(|m| options.gemini_mcp_names.contains(&m.name));
        let has_env = options.include_gemini_env;
        if !has_env && gemini.mcp_servers.is_empty() {
            backup.gemini_config = None;
        }
    }
    
    if !options.dev_envs.is_empty() {
        backup.dev_envs = Some(options.dev_envs.clone());
    }

    if !options.usage_sources.is_empty() {
        let conn = db.conn.lock().map_err(|e| AppError::Custom(format!("DB lock failed: {}", e)))?;
        let mut usage_records = Vec::new();
        let placeholders: Vec<&str> = options.usage_sources.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost FROM proxy_request_logs WHERE app_type IN ({}) ORDER BY created_at",
            placeholders.join(", ")
        );
        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map(rusqlite::params_from_iter(options.usage_sources.iter()), |row| {
                Ok(ExportedUsageRecord {
                    session_id: row.get::<_, String>(0).unwrap_or_default(),
                    timestamp: row.get::<_, i64>(1).unwrap_or(0),
                    model: row.get::<_, String>(2).unwrap_or_default(),
                    source: row.get::<_, String>(3).unwrap_or_default(),
                    input_tokens: row.get::<_, u32>(4).unwrap_or(0),
                    output_tokens: row.get::<_, u32>(5).unwrap_or(0),
                    cache_read_tokens: row.get::<_, u32>(6).unwrap_or(0),
                    cache_creation_tokens: row.get::<_, u32>(7).unwrap_or(0),
                    cost: row.get::<_, f64>(8).ok(),
                })
            }) {
                for r in rows.flatten() { usage_records.push(r); }
            }
        }
        backup.usage_stats = Some(usage_records);
    }
    
    backup.chat_conversations = chat_conversations;
    
    let stats = ExportStats {
        providers: backup.providers.len(),
        models: backup.providers.iter().map(|p| p.models.len()).sum(),
        mcp_servers: backup.mcp_servers.len(),
        rules: backup.rules.len(),
        skills: backup.skills.len(),
        codex_providers: backup.codex_config.as_ref().map(|c| c.model_providers.len()).unwrap_or(0),
        codex_mcp_servers: backup.codex_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        gemini_configured: backup.gemini_config.is_some(),
        gemini_mcp_servers: backup.gemini_config.as_ref().map(|c| c.mcp_servers.len()).unwrap_or(0),
        usage_records: backup.usage_stats.as_ref().map(|r| r.len()).unwrap_or(0),
        chat_conversations: backup.chat_conversations.as_ref().map(|c| c.len()).unwrap_or(0),
    };
    
    let content = serde_json::to_string_pretty(&backup)
        .map_err(|e| AppError::Custom(format!("Failed to serialize: {}", e)))?;
    
    fs::write(&file_path, content)
        .map_err(|e| AppError::Custom(format!("Failed to write file: {}", e)))?;
    
    Ok(stats)
}

#[tauri::command]
pub fn preview_backup(file_path: String) -> Result<BackupData, AppError> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| AppError::Custom(format!("Failed to read file: {}", e)))?;
    
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Custom(format!("Failed to parse file: {}", e)))?;
    
    Ok(backup)
}

#[tauri::command]
pub fn import_backup(
    file_path: String,
    options: ImportOptions,
    config_manager: State<'_, Mutex<ConfigManager>>,
    db: State<'_, std::sync::Arc<crate::modules::opencode_db::Database>>,
) -> Result<ImportResult, AppError> {
    let content = fs::read_to_string(&file_path)
        .map_err(|e| AppError::Custom(format!("Failed to read file: {}", e)))?;
    
    let backup: BackupData = serde_json::from_str(&content)
        .map_err(|e| AppError::Custom(format!("Failed to parse file: {}", e)))?;
    
    let mut result = ImportResult {
        success: true,
        providers_imported: 0,
        providers_skipped: 0,
        mcp_imported: 0,
        mcp_skipped: 0,
        rules_imported: 0,
        rules_skipped: 0,
        skills_imported: 0,
        skills_skipped: 0,
        codex_imported: 0,
        codex_skipped: 0,
        gemini_imported: 0,
        gemini_skipped: 0,
        usage_imported: 0,
        usage_skipped: 0,
        chat_conversations_imported: 0,
        chat_conversations_skipped: 0,
        errors: Vec::new(),
    };
    
    let mut manager = config_manager.lock().map_err(|e| AppError::Custom(e.to_string()))?;
    
    if options.import_providers {
        let existing = match manager.opencode().get_all_providers() {
            Ok(map) => Some(map),
            Err(e) => {
                result.errors.push(format!("读取现有 Provider 失败: {}", e));
                None
            }
        };
        
        if let Some(existing) = existing {
            for provider in &backup.providers {
                let exists = existing.contains_key(&provider.name);
            
                if exists && !options.overwrite_existing {
                    result.providers_skipped += 1;
                    continue;
                }
            
                if exists && options.overwrite_existing {
                    if let Err(e) = manager.opencode_mut().delete_provider(&provider.name) {
                        result.errors.push(format!("删除 Provider '{}' 失败: {}", provider.name, e));
                        continue;
                    }
                }
            
                let model_type = provider.model_type.clone().unwrap_or_else(|| "claude".to_string());
                let variants = build_variants(&model_type);
            
                match manager.opencode_mut().add_provider(
                    provider.name.clone(),
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    provider.npm.clone(),
                    provider.description.clone(),
                    provider.model_type.clone(),
                    true,
                ) {
                    Ok(_) => {
                        for model in &provider.models {
                            let model_info = crate::modules::opencode_config::OpenCodeModelInfo {
                                id: model.id.clone(),
                                name: model.name.clone(),
                                limit: None,
                                reasoning: Some(true),
                                variants: Some(variants.clone()),
                                options: None,
                                reasoning_effort: model.reasoning_effort.clone(),
                                thinking_budget: None,
                                model_detection: None,
                            };
                            if let Err(e) = manager.opencode_mut().add_model(&provider.name, model.id.clone(), model_info) {
                                result.errors.push(format!(
                                    "Provider '{}' 添加模型 '{}' 失败: {}",
                                    provider.name, model.id, e
                                ));
                            }
                        }
                        if let Err(e) = manager.opencode_mut().toggle_provider(&provider.name, provider.enabled) {
                            result.errors.push(format!(
                                "Provider '{}' 更新启用状态失败: {}",
                                provider.name, e
                            ));
                        }
                        result.providers_imported += 1;
                    }
                    Err(e) => {
                        result.errors.push(format!("Provider '{}': {}", provider.name, e));
                    }
                }
            }
        }
    }
    
    if options.import_mcp {
        let existing = match manager.mcp().read_config().map(|c| c.servers) {
            Ok(map) => Some(map),
            Err(e) => {
                result.errors.push(format!("读取现有 MCP 配置失败: {}", e));
                None
            }
        };
        
        if let Some(existing) = existing {
            for mcp in &backup.mcp_servers {
                let exists = existing.contains_key(&mcp.name);
            
                if exists && !options.overwrite_existing {
                    result.mcp_skipped += 1;
                    continue;
                }
            
                if exists && options.overwrite_existing {
                    if let Err(e) = manager.mcp_mut().delete_server(&mcp.name) {
                        result.errors.push(format!("删除 MCP '{}' 失败: {}", mcp.name, e));
                        continue;
                    }
                }
            
                let oauth = mcp.oauth.as_ref().map(|o| crate::modules::opencode_config::McpOAuthConfig {
                    client_id: o.client_id.clone(),
                    client_secret: o.client_secret.clone(),
                    scope: o.scope.clone(),
                });
            
                let server = if mcp.server_type == "local" {
                    McpServer {
                        server_type: McpServerType::Local,
                        enabled: mcp.enabled,
                        timeout: mcp.timeout,
                        command: mcp.command.clone(),
                        environment: mcp.environment.clone().unwrap_or_default(),
                        url: None,
                        headers: HashMap::new(),
                        oauth: None,
                        metadata: Default::default(),
                    }
                } else {
                    McpServer {
                        server_type: McpServerType::Remote,
                        enabled: mcp.enabled,
                        timeout: mcp.timeout,
                        command: None,
                        environment: HashMap::new(),
                        url: mcp.url.clone(),
                        headers: mcp.headers.clone().unwrap_or_default(),
                        oauth,
                        metadata: Default::default(),
                    }
                };
            
                match manager.mcp_mut().save_server(&mcp.name, server) {
                    Ok(_) => result.mcp_imported += 1,
                    Err(e) => result.errors.push(format!("MCP '{}': {}", mcp.name, e)),
                }
            }
        }
        if let Err(e) = manager.mcp().sync_to_opencode(None) {
            result.errors.push(format!("同步 MCP 配置失败: {}", e));
        }
    }
    
    if options.import_rules {
        let rule_paths = get_rule_paths();
        
        for rule in &backup.rules {
            if let Some(base_path) = rule_paths.get(&rule.location) {
                if let Err(e) = fs::create_dir_all(base_path) {
                    result.errors.push(format!("创建目录失败: {}", e));
                    continue;
                }
                
                let target_path = if rule.rule_type == "agents_md" {
                    base_path.join("AGENTS.md")
                } else if rule.rule_type == "gemini_md" {
                    base_path.join("GEMINI.md")
                } else {
                    let rules_dir = base_path.join("rules");
                    if let Err(e) = fs::create_dir_all(&rules_dir) {
                        result.errors.push(format!("Create dir failed: {}", e));
                        continue;
                    }
                    let ext = if rule.file_ext.is_empty() { "md" } else { &rule.file_ext };
                    rules_dir.join(format!("{}.{}", rule.name, ext))
                };
                
                if target_path.exists() && !options.overwrite_existing {
                    result.rules_skipped += 1;
                    continue;
                }
                
                match fs::write(&target_path, &rule.content) {
                    Ok(_) => result.rules_imported += 1,
                    Err(e) => result.errors.push(format!("Rule '{}': {}", rule.name, e)),
                }
            }
        }
    }
    
    if options.import_skills {
        for skills in &backup.skills {
            let base_path = match skills.location.as_str() {
                "global_opencode" => dirs::home_dir().map(|d| d.join(".config").join("opencode").join("skills")),
                "global_claude" => dirs::home_dir().map(|d| d.join(".claude").join("skills")),
                "global_codex" => dirs::home_dir().map(|d| d.join(".codex").join("skills")),
                "global_gemini" => dirs::home_dir().map(|d| d.join(".gemini").join("skills")),
                _ => None,
            };
            
            if let Some(base) = base_path {
                let skills_dir = base.join(&skills.name);
                if let Err(e) = fs::create_dir_all(&skills_dir) {
                    result.errors.push(format!("Create dir failed: {}", e));
                    continue;
                }
                
                let skills_file = skills_dir.join("SKILL.md");
                if skills_file.exists() && !options.overwrite_existing {
                    result.skills_skipped += 1;
                    continue;
                }
                
                match fs::write(&skills_file, &skills.content) {
                    Ok(_) => result.skills_imported += 1,
                    Err(e) => result.errors.push(format!("skills '{}': {}", skills.name, e)),
                }
            }
        }
    }
    
    if options.import_codex {
        if let Some(ref codex_config) = backup.codex_config {
            import_codex_config(codex_config, &options, &mut result);
        }
    }
    
    if options.import_gemini {
        if let Some(ref gemini_config) = backup.gemini_config {
            import_gemini_config(gemini_config, &options, &mut result);
        }
    }
    
    if options.import_usage_stats {
        if let Some(records) = &backup.usage_stats {
            if let Ok(conn) = db.conn.lock() {
                let mut existing_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
                if let Ok(mut stmt) = conn.prepare("SELECT session_id, created_at FROM proxy_request_logs") {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        let sid: String = row.get(0)?;
                        let ts: i64 = row.get(1)?;
                        Ok(format!("{}|{}", sid, ts))
                    }) {
                        for key in rows.flatten() { existing_keys.insert(key); }
                    }
                }

                let _ = conn.execute_batch("BEGIN IMMEDIATE");
                for record in records {
                    let dedup_key = format!("{}|{}", record.session_id, record.timestamp);
                    if existing_keys.contains(&dedup_key) {
                        result.usage_skipped += 1;
                        continue;
                    }
                    let insert_result = conn.execute(
                        "INSERT INTO proxy_request_logs (session_id, created_at, model, app_type, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, cost, success) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1)",
                        rusqlite::params![
                            record.session_id, record.timestamp, record.model, record.source,
                            record.input_tokens, record.output_tokens, record.cache_read_tokens,
                            record.cache_creation_tokens, record.cost.unwrap_or(0.0)
                        ],
                    );
                    match insert_result {
                        Ok(_) => { result.usage_imported += 1; }
                        Err(_) => { result.usage_skipped += 1; }
                    }
                }
                let _ = conn.execute_batch("COMMIT");
            }
        }
    }

    if options.import_chat_conversations {
        if let Some(conversations) = &backup.chat_conversations {
            import_chat_conversations_to_store(conversations, &mut result);
        }
    }

    result.success = result.errors.is_empty();
    Ok(result)
}

fn get_migration_store_path_internal() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".ai-switch").join("migrated_conversations.jsonl"))
}

fn read_migrated_conversations_for_backup() -> Option<Vec<ExportedChatConversation>> {
    let store_path = get_migration_store_path_internal()?;
    if !store_path.exists() { return None; }
    let content = fs::read_to_string(&store_path).ok()?;
    let mut conversations = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() { continue; }
        if let Ok(conv) = serde_json::from_str::<ExportedChatConversation>(line) {
            conversations.push(conv);
        }
    }
    if conversations.is_empty() { None } else { Some(conversations) }
}

fn import_chat_conversations_to_store(
    conversations: &[ExportedChatConversation],
    result: &mut ImportResult,
) {
    let Some(store_path) = get_migration_store_path_internal() else {
        result.errors.push("无法获取对话存储路径".to_string());
        return;
    };

    let mut existing_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut existing_lines: Vec<String> = Vec::new();
    if store_path.exists() {
        if let Ok(content) = fs::read_to_string(&store_path) {
            for line in content.lines() {
                if line.trim().is_empty() { continue; }
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    existing_keys.insert(chat_dedup_key(&json));
                    existing_lines.push(line.to_string());
                }
            }
        }
    }

    let mut new_lines: Vec<String> = Vec::new();
    for conv in conversations {
        if let Ok(json) = serde_json::to_value(conv) {
            let key = chat_dedup_key(&json);
            if existing_keys.contains(&key) {
                result.chat_conversations_skipped += 1;
            } else {
                existing_keys.insert(key);
                if let Ok(line) = serde_json::to_string(conv) {
                    new_lines.push(line);
                    result.chat_conversations_imported += 1;
                }
            }
        }
    }

    if !new_lines.is_empty() {
        existing_lines.extend(new_lines);
        if let Some(parent) = store_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&store_path, existing_lines.join("\n")) {
            result.errors.push(format!("写入对话记录失败: {}", e));
        }
    }
}

fn chat_dedup_key(json: &serde_json::Value) -> String {
    let source = json.get("source").and_then(|v| v.as_str()).unwrap_or("");
    let session_id = json.get("sessionId")
        .or_else(|| json.get("session_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let first_content = json.get("messages")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|msg| msg.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let content_preview = &first_content[..first_content.len().min(100)];
    format!("{}|{}|{:x}", source, session_id, simple_hash_backup(content_preview))
}

fn simple_hash_backup(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u64); }
    h
}

fn import_codex_config(
    config: &ExportedCodexConfig,
    options: &ImportOptions,
    result: &mut ImportResult,
) {
    let codex_manager = match CodexConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(format!("初始化 Codex 配置管理器失败: {}", e));
            return;
        }
    };
    
    for provider in &config.model_providers {
        let existing = codex_manager.get_model_providers().ok();
        let exists = existing
            .as_ref()
            .map(|p| p.contains_key(&provider.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.codex_skipped += 1;
            continue;
        }
        
        let codex_provider = crate::modules::opencode_config::codex_manager::CodexModelProvider {
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            env_key: provider.env_key.clone(),
            requires_openai_auth: provider.requires_openai_auth,
        };
        
        match codex_manager.add_model_provider(&provider.name, codex_provider) {
            Ok(_) => result.codex_imported += 1,
            Err(e) => result.errors.push(format!("Codex Provider '{}': {}", provider.name, e)),
        }
    }
    
    for server in &config.mcp_servers {
        let existing = codex_manager.get_mcp_servers().ok();
        let exists = existing
            .as_ref()
            .map(|s| s.contains_key(&server.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.codex_skipped += 1;
            continue;
        }
        
        let codex_server = crate::modules::opencode_config::codex_manager::CodexMcpServer {
            command: server.command.clone(),
            env: server.env.clone(),
        };
        
        match codex_manager.add_mcp_server(&server.name, codex_server) {
            Ok(_) => result.codex_imported += 1,
            Err(e) => result.errors.push(format!("Codex MCP '{}': {}", server.name, e)),
        }
    }
}

fn import_gemini_config(
    config: &ExportedGeminiConfig,
    options: &ImportOptions,
    result: &mut ImportResult,
) {
    let gemini_manager = match GeminiConfigManager::new() {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(format!("初始化 Gemini 配置管理器失败: {}", e));
            return;
        }
    };
    
    let has_env = config.env.gemini_api_key.is_some() 
        || config.env.google_gemini_api_key.is_some() 
        || config.env.google_gemini_base_url.is_some()
        || config.env.gemini_model.is_some();
    
    if has_env {
        let existing_env = gemini_manager.read_env().ok();
        let env_exists = existing_env
            .as_ref()
            .map(|e| e.gemini_api_key.is_some() || e.google_gemini_api_key.is_some())
            .unwrap_or(false);
        
        if env_exists && !options.overwrite_existing {
            result.gemini_skipped += 1;
        } else {
            let env = crate::modules::opencode_config::gemini_manager::GeminiEnv {
                gemini_api_key: config.env.gemini_api_key.clone(),
                google_gemini_api_key: config.env.google_gemini_api_key.clone(),
                google_gemini_base_url: config.env.google_gemini_base_url.clone(),
                gemini_model: config.env.gemini_model.clone(),
                other: std::collections::HashMap::new(),
            };
            
            match gemini_manager.write_env(&env) {
                Ok(_) => result.gemini_imported += 1,
                Err(e) => result.errors.push(format!("Gemini ENV: {}", e)),
            }
        }
    }
    
    for server in &config.mcp_servers {
        let existing = gemini_manager.get_mcp_servers().ok();
        let exists = existing
            .as_ref()
            .map(|s| s.contains_key(&server.name))
            .unwrap_or(false);
        
        if exists && !options.overwrite_existing {
            result.gemini_skipped += 1;
            continue;
        }
        
        let gemini_server = crate::modules::opencode_config::gemini_manager::GeminiMcpServer {
            command: server.command.clone(),
            args: server.args.clone(),
            env: server.env.clone(),
            url: server.url.clone(),
        };
        
        match gemini_manager.add_mcp_server(&server.name, gemini_server) {
            Ok(_) => result.gemini_imported += 1,
            Err(e) => result.errors.push(format!("Gemini MCP '{}': {}", server.name, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn test_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("ai_switch_test").join(name);
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn make_test_conversation(source: &str, session_id: &str, content: &str) -> ExportedChatConversation {
        ExportedChatConversation {
            messages: vec![ExportedChatMessage {
                role: "user".to_string(),
                content: content.to_string(),
                model: None,
                timestamp: None,
                tool_use: None,
            }],
            source: source.to_string(),
            session_id: Some(session_id.to_string()),
            name: Some(format!("Test {}", session_id)),
            created_at: Some(1738000000000.0),
        }
    }

    #[test]
    fn test_chat_dedup_key_consistency() {
        let conv = make_test_conversation("claude", "abc123", "Hello world");
        let json = serde_json::to_value(&conv).unwrap();
        let key1 = chat_dedup_key(&json);
        let key2 = chat_dedup_key(&json);
        assert_eq!(key1, key2, "相同数据应产生相同的 dedup key");
        assert!(key1.starts_with("claude|abc123|"), "前缀应包含 source 和 sessionId");
    }

    #[test]
    fn test_chat_dedup_key_different_for_different_data() {
        let conv1 = make_test_conversation("claude", "abc", "Hello");
        let conv2 = make_test_conversation("cursor", "abc", "Hello");
        let conv3 = make_test_conversation("claude", "abc", "World");
        let j1 = serde_json::to_value(&conv1).unwrap();
        let j2 = serde_json::to_value(&conv2).unwrap();
        let j3 = serde_json::to_value(&conv3).unwrap();
        assert_ne!(chat_dedup_key(&j1), chat_dedup_key(&j2), "不同 source 应产生不同 key");
        assert_ne!(chat_dedup_key(&j1), chat_dedup_key(&j3), "不同 content 应产生不同 key");
    }

    #[test]
    fn test_import_chat_conversations_to_store_basic() {
        let temp_dir = test_temp_dir("import_chat");
        let store_path = temp_dir.join("migrated_conversations.jsonl");
        let _ = fs::remove_file(&store_path);

        let conversations = vec![
            make_test_conversation("claude", "s1", "Hello"),
            make_test_conversation("cursor", "s2", "World"),
        ];

        let mut result = ImportResult {
            success: true,
            providers_imported: 0, providers_skipped: 0,
            mcp_imported: 0, mcp_skipped: 0,
            rules_imported: 0, rules_skipped: 0,
            skills_imported: 0, skills_skipped: 0,
            codex_imported: 0, codex_skipped: 0,
            gemini_imported: 0, gemini_skipped: 0,
            usage_imported: 0, usage_skipped: 0,
            chat_conversations_imported: 0, chat_conversations_skipped: 0,
            errors: Vec::new(),
        };

        let mut existing_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut lines: Vec<String> = Vec::new();
        for conv in &conversations {
            if let Ok(json) = serde_json::to_value(conv) {
                let key = chat_dedup_key(&json);
                if !existing_keys.contains(&key) {
                    existing_keys.insert(key);
                    if let Ok(line) = serde_json::to_string(conv) {
                        lines.push(line);
                        result.chat_conversations_imported += 1;
                    }
                }
            }
        }
        fs::write(&store_path, lines.join("\n")).unwrap();

        assert_eq!(result.chat_conversations_imported, 2);
        assert_eq!(result.chat_conversations_skipped, 0);

        let content = fs::read_to_string(&store_path).unwrap();
        let file_lines: Vec<&str> = content.lines().collect();
        assert_eq!(file_lines.len(), 2);

        let mut result2 = result.clone();
        result2.chat_conversations_imported = 0;
        result2.chat_conversations_skipped = 0;
        let mut existing_keys2: std::collections::HashSet<String> = std::collections::HashSet::new();
        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                existing_keys2.insert(chat_dedup_key(&json));
            }
        }
        for conv in &conversations {
            if let Ok(json) = serde_json::to_value(conv) {
                let key = chat_dedup_key(&json);
                if existing_keys2.contains(&key) {
                    result2.chat_conversations_skipped += 1;
                }
            }
        }
        assert_eq!(result2.chat_conversations_skipped, 2, "重复导入应全部跳过");
    }

    #[test]
    fn test_read_migrated_conversations_from_jsonl() {
        let temp_dir = test_temp_dir("read_jsonl");
        let store_path = temp_dir.join("test.jsonl");
        let _ = fs::remove_file(&store_path);

        let conv1 = make_test_conversation("claude", "s1", "Test content 1");
        let conv2 = make_test_conversation("cursor", "s2", "Test content 2");
        let mut file = fs::File::create(&store_path).unwrap();
        writeln!(file, "{}", serde_json::to_string(&conv1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&conv2).unwrap()).unwrap();

        let content = fs::read_to_string(&store_path).unwrap();
        let mut conversations: Vec<ExportedChatConversation> = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() { continue; }
            if let Ok(conv) = serde_json::from_str::<ExportedChatConversation>(line) {
                conversations.push(conv);
            }
        }
        assert_eq!(conversations.len(), 2);
        assert_eq!(conversations[0].source, "claude");
        assert_eq!(conversations[0].session_id.as_deref(), Some("s1"));
        assert_eq!(conversations[1].source, "cursor");
        assert_eq!(conversations[1].messages[0].content, "Test content 2");
    }

    #[test]
    fn test_serialization_compatibility() {
        let json_str = r#"{"messages":[{"role":"user","content":"hello"}],"source":"claude-code","sessionId":"abc","name":"Test","createdAt":1738000000}"#;
        let conv: ExportedChatConversation = serde_json::from_str(json_str).unwrap();
        assert_eq!(conv.source, "claude-code");
        assert_eq!(conv.session_id.as_deref(), Some("abc"));
        assert_eq!(conv.messages.len(), 1);
        assert_eq!(conv.created_at, Some(1738000000.0));
    }
}
