use super::types::{AccountStatus, GatewayAccount};
use crate::modules::logger;
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Hash, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Antigravity,
    Codex,
    GithubCopilot,
    Windsurf,
    Kiro,
    Cursor,
    Gemini,
    CodeBuddy,
    CodeBuddyCn,
    WorkBuddy,
    Qoder,
    Trae,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Antigravity => write!(f, "antigravity"),
            Platform::Codex => write!(f, "codex"),
            Platform::GithubCopilot => write!(f, "github_copilot"),
            Platform::Windsurf => write!(f, "windsurf"),
            Platform::Kiro => write!(f, "kiro"),
            Platform::Cursor => write!(f, "cursor"),
            Platform::Gemini => write!(f, "gemini"),
            Platform::CodeBuddy => write!(f, "codebuddy"),
            Platform::CodeBuddyCn => write!(f, "codebuddy_cn"),
            Platform::WorkBuddy => write!(f, "workbuddy"),
            Platform::Qoder => write!(f, "qoder"),
            Platform::Trae => write!(f, "trae"),
        }
    }
}

impl Platform {
    pub fn default_upstream_url(&self) -> &'static str {
        match self {
            Platform::Antigravity | Platform::Codex => "https://api.openai.com",
            Platform::GithubCopilot => "https://api.github.com",
            Platform::Windsurf => "https://api.codeium.com",
            Platform::Kiro => "https://api.kiro.dev",
            Platform::Cursor => "https://api2.cursor.sh",
            Platform::Gemini => "https://generativelanguage.googleapis.com",
            Platform::CodeBuddy | Platform::CodeBuddyCn | Platform::WorkBuddy => {
                "https://api.codebuddy.ai"
            }
            Platform::Qoder => "https://api.qoder.ai",
            Platform::Trae => "https://api.trae.ai",
        }
    }

    pub fn all() -> Vec<Platform> {
        vec![
            Platform::Antigravity,
            Platform::Codex,
            Platform::GithubCopilot,
            Platform::Windsurf,
            Platform::Kiro,
            Platform::Cursor,
            Platform::Gemini,
            Platform::CodeBuddy,
            Platform::CodeBuddyCn,
            Platform::WorkBuddy,
            Platform::Qoder,
            Platform::Trae,
        ]
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlatformAccount {
    pub account: GatewayAccount,
    pub platform: Platform,
    pub source: AccountSource,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountSource {
    Manual,
    Synced,
}

pub fn collect_platform_accounts() -> Vec<PlatformAccount> {
    let mut result = Vec::new();
    let data_dir = match dirs::data_dir() {
        Some(d) => d.join("com.jlcodes.ai-switch"),
        None => return result,
    };

    for platform in Platform::all() {
        match load_accounts_for_platform(&data_dir, &platform) {
            Ok(accounts) => {
                logger::log_info(&format!(
                    "[AccountBridge] {} 平台加载了 {} 个账号",
                    platform,
                    accounts.len()
                ));
                result.extend(accounts);
            }
            Err(e) => {
                logger::log_warn(&format!(
                    "[AccountBridge] {} 平台加载失败: {}",
                    platform, e
                ));
            }
        }
    }

    result
}

fn load_accounts_for_platform(
    data_dir: &std::path::Path,
    platform: &Platform,
) -> Result<Vec<PlatformAccount>, String> {
    let platform_dir = data_dir.join(platform.to_string());
    let index_path = platform_dir.join("accounts_index.json");

    if !index_path.exists() {
        return Ok(Vec::new());
    }

    let content =
        std::fs::read_to_string(&index_path).map_err(|e| format!("读取账号索引失败: {}", e))?;

    let index: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析账号索引失败: {}", e))?;

    let mut accounts = Vec::new();

    if let Some(account_list) = index.get("accounts").and_then(|a| a.as_array()) {
        for summary in account_list {
            let account_id = summary.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if account_id.is_empty() {
                continue;
            }

            let account_file = platform_dir.join(format!("{}.json", account_id));
            if !account_file.exists() {
                continue;
            }

            match load_single_account(&account_file, account_id, platform) {
                Ok(pa) => accounts.push(pa),
                Err(e) => {
                    logger::log_warn(&format!(
                        "[AccountBridge] 加载账号 {} 失败: {}",
                        account_id, e
                    ));
                }
            }
        }
    }

    Ok(accounts)
}

fn load_single_account(
    path: &std::path::Path,
    account_id: &str,
    platform: &Platform,
) -> Result<PlatformAccount, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("读取账号文件失败: {}", e))?;
    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析账号文件失败: {}", e))?;

    let email = json
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let access_token = json
        .get("auth")
        .and_then(|a| a.get("access_token"))
        .and_then(|v| v.as_str())
        .or_else(|| json.get("access_token").and_then(|v| v.as_str()))
        .or_else(|| json.get("copilot_token").and_then(|v| v.as_str()))
        .or_else(|| json.get("github_access_token").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    if access_token.is_empty() {
        return Err("无可用的 access_token".to_string());
    }

    let refresh_token = json
        .get("auth")
        .and_then(|a| a.get("refresh_token"))
        .and_then(|v| v.as_str())
        .or_else(|| json.get("refresh_token").and_then(|v| v.as_str()))
        .map(|s| s.to_string());

    let disabled = json
        .get("disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let now = chrono::Utc::now().timestamp();
    let gateway_account = GatewayAccount {
        id: format!("bridge_{}_{}", platform, account_id),
        email,
        access_token,
        refresh_token,
        token_expires_at: None,
        status: if disabled {
            AccountStatus::Inactive
        } else {
            AccountStatus::Active
        },
        tags: Some(platform.to_string()),
        group_name: Some(platform.to_string()),
        proxy_url: None,
        created_at: now,
        updated_at: now,
        last_used_at: None,
        cooldown_until: None,
        error_count: 0,
        platform: Some(platform.to_string()),
        source: Some("synced".to_string()),
    };

    Ok(PlatformAccount {
        account: gateway_account,
        platform: platform.clone(),
        source: AccountSource::Synced,
    })
}

pub fn sync_platform_accounts_to_gateway() -> Result<SyncResult, String> {
    let platform_accounts = collect_platform_accounts();
    let existing = super::db::list_accounts().unwrap_or_default();

    let mut added = 0;
    let mut updated = 0;
    let mut skipped = 0;

    for pa in &platform_accounts {
        let existing_match = existing.iter().find(|e| e.id == pa.account.id);

        if let Some(existing_account) = existing_match {
            if existing_account.access_token != pa.account.access_token {
                super::db::insert_account(
                    &pa.account.id,
                    &pa.account.email,
                    &pa.account.access_token,
                    pa.account.refresh_token.as_deref(),
                    pa.account.tags.as_deref(),
                    pa.account.group_name.as_deref(),
                    pa.account.proxy_url.as_deref(),
                    pa.account.platform.as_deref(),
                    pa.account.source.as_deref(),
                )?;
                updated += 1;
            } else {
                skipped += 1;
            }
        } else {
            super::db::insert_account(
                &pa.account.id,
                &pa.account.email,
                &pa.account.access_token,
                pa.account.refresh_token.as_deref(),
                pa.account.tags.as_deref(),
                pa.account.group_name.as_deref(),
                pa.account.proxy_url.as_deref(),
                pa.account.platform.as_deref(),
                pa.account.source.as_deref(),
            )?;
            added += 1;
        }
    }

    logger::log_info(&format!(
        "[AccountBridge] 同步完成: 新增 {}, 更新 {}, 跳过 {}",
        added, updated, skipped
    ));

    Ok(SyncResult {
        total_platforms: Platform::all().len(),
        total_accounts: platform_accounts.len(),
        added,
        updated,
        skipped,
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncResult {
    pub total_platforms: usize,
    pub total_accounts: usize,
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

pub fn get_platform_account_stats() -> HashMap<String, usize> {
    let accounts = collect_platform_accounts();
    let mut stats = HashMap::new();
    for pa in &accounts {
        *stats.entry(pa.platform.to_string()).or_insert(0) += 1;
    }
    stats
}
