use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;

static DB_CONNECTION: Mutex<Option<Connection>> = Mutex::new(None);

fn db_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("gateway.db")
}

pub fn init_gateway_db() -> Result<(), String> {
    let mut guard = DB_CONNECTION.lock().unwrap();
    if guard.is_some() {
        return Ok(());
    }

    let path = db_path();
    let conn = Connection::open(&path).map_err(|e| format!("打开数据库失败: {}", e))?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("设置 PRAGMA 失败: {}", e))?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS gateway_accounts (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            token_expires_at INTEGER,
            status TEXT NOT NULL DEFAULT 'active',
            tags TEXT,
            group_name TEXT,
            proxy_url TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_used_at INTEGER,
            cooldown_until INTEGER,
            error_count INTEGER NOT NULL DEFAULT 0,
            platform TEXT,
            source TEXT DEFAULT 'manual'
        );

        CREATE TABLE IF NOT EXISTS gateway_api_keys (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            key_hash TEXT NOT NULL UNIQUE,
            key_prefix TEXT NOT NULL,
            allowed_models TEXT,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            last_used_at INTEGER,
            usage_count INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS gateway_request_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trace_id TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            method TEXT NOT NULL,
            path TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            account_email TEXT,
            model TEXT,
            input_tokens INTEGER,
            output_tokens INTEGER,
            error_message TEXT,
            api_key_prefix TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_request_logs_timestamp ON gateway_request_logs(timestamp);
        CREATE INDEX IF NOT EXISTS idx_request_logs_status ON gateway_request_logs(status_code);
        CREATE INDEX IF NOT EXISTS idx_accounts_status ON gateway_accounts(status);
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON gateway_api_keys(key_hash);
        ",
    )
    .map_err(|e| format!("创建表失败: {}", e))?;

    *guard = Some(conn);

    Ok(())
}

pub fn with_db<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&Connection) -> Result<R, rusqlite::Error>,
{
    init_gateway_db()?;
    let guard = DB_CONNECTION.lock().unwrap();
    let conn = guard.as_ref().ok_or("网关数据库未初始化")?;
    f(conn).map_err(|e| format!("数据库操作失败: {}", e))
}

pub fn insert_account(
    id: &str,
    email: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    tags: Option<&str>,
    group_name: Option<&str>,
    proxy_url: Option<&str>,
    platform: Option<&str>,
    source: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    with_db(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO gateway_accounts (id, email, access_token, refresh_token, status, tags, group_name, proxy_url, created_at, updated_at, platform, source)
             VALUES (?1, ?2, ?3, ?4, 'active', ?5, ?6, ?7, ?8, ?8, ?9, ?10)",
            params![id, email, access_token, refresh_token, tags, group_name, proxy_url, now, platform, source],
        )?;
        Ok(())
    })
}

pub fn list_accounts() -> Result<Vec<super::types::GatewayAccount>, String> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, email, access_token, refresh_token, token_expires_at, status, tags, group_name, proxy_url, created_at, updated_at, last_used_at, cooldown_until, error_count, platform, source
             FROM gateway_accounts ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(super::types::GatewayAccount {
                id: row.get(0)?,
                email: row.get(1)?,
                access_token: row.get(2)?,
                refresh_token: row.get(3)?,
                token_expires_at: row.get(4)?,
                status: match row.get::<_, String>(5)?.as_str() {
                    "inactive" => super::types::AccountStatus::Inactive,
                    "cooldown" => super::types::AccountStatus::Cooldown,
                    "error" => super::types::AccountStatus::Error,
                    "expired" => super::types::AccountStatus::Expired,
                    _ => super::types::AccountStatus::Active,
                },
                tags: row.get(6)?,
                group_name: row.get(7)?,
                proxy_url: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                last_used_at: row.get(11)?,
                cooldown_until: row.get(12)?,
                error_count: row.get(13)?,
                platform: row.get(14).ok().unwrap_or(None),
                source: row.get(15).ok().unwrap_or(None),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

pub fn delete_account(id: &str) -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM gateway_accounts WHERE id = ?1", params![id])?;
        Ok(())
    })
}

pub fn update_account_status(id: &str, status: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    with_db(|conn| {
        conn.execute(
            "UPDATE gateway_accounts SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;
        Ok(())
    })
}

pub fn mark_account_used(id: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    with_db(|conn| {
        conn.execute(
            "UPDATE gateway_accounts SET last_used_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    })
}

pub fn get_active_accounts() -> Result<Vec<super::types::GatewayAccount>, String> {
    with_db(|conn| {
        let now = chrono::Utc::now().timestamp();
        let mut stmt = conn.prepare(
            "SELECT id, email, access_token, refresh_token, token_expires_at, status, tags, group_name, proxy_url, created_at, updated_at, last_used_at, cooldown_until, error_count, platform, source
             FROM gateway_accounts
             WHERE status = 'active' AND (cooldown_until IS NULL OR cooldown_until < ?1)
             ORDER BY last_used_at ASC NULLS FIRST",
        )?;
        let rows = stmt.query_map(params![now], |row| {
            Ok(super::types::GatewayAccount {
                id: row.get(0)?,
                email: row.get(1)?,
                access_token: row.get(2)?,
                refresh_token: row.get(3)?,
                token_expires_at: row.get(4)?,
                status: super::types::AccountStatus::Active,
                tags: row.get(6)?,
                group_name: row.get(7)?,
                proxy_url: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
                last_used_at: row.get(11)?,
                cooldown_until: row.get(12)?,
                error_count: row.get(13)?,
                platform: row.get(14).ok().unwrap_or(None),
                source: row.get(15).ok().unwrap_or(None),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

pub fn insert_api_key(
    id: &str,
    name: &str,
    key_hash: &str,
    key_prefix: &str,
    allowed_models: Option<&str>,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    with_db(|conn| {
        conn.execute(
            "INSERT INTO gateway_api_keys (id, name, key_hash, key_prefix, allowed_models, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, key_hash, key_prefix, allowed_models, now],
        )?;
        Ok(())
    })
}

pub fn list_api_keys() -> Result<Vec<super::types::GatewayApiKey>, String> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, key_hash, key_prefix, allowed_models, enabled, created_at, last_used_at, usage_count
             FROM gateway_api_keys ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(super::types::GatewayApiKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key_hash: row.get(2)?,
                key_prefix: row.get(3)?,
                allowed_models: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
                last_used_at: row.get(7)?,
                usage_count: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

pub fn validate_api_key(key_hash: &str) -> Result<Option<super::types::GatewayApiKey>, String> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, key_hash, key_prefix, allowed_models, enabled, created_at, last_used_at, usage_count
             FROM gateway_api_keys WHERE key_hash = ?1 AND enabled = 1",
        )?;
        let mut rows = stmt.query(params![key_hash])?;
        if let Some(row) = rows.next()? {
            Ok(Some(super::types::GatewayApiKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key_hash: row.get(2)?,
                key_prefix: row.get(3)?,
                allowed_models: row.get(4)?,
                enabled: true,
                created_at: row.get(6)?,
                last_used_at: row.get(7)?,
                usage_count: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    })
}

pub fn delete_api_key(id: &str) -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM gateway_api_keys WHERE id = ?1", params![id])?;
        Ok(())
    })
}

pub fn toggle_api_key(id: &str, enabled: bool) -> Result<(), String> {
    with_db(|conn| {
        conn.execute(
            "UPDATE gateway_api_keys SET enabled = ?1 WHERE id = ?2",
            params![enabled as i32, id],
        )?;
        Ok(())
    })
}

pub fn insert_request_log(entry: &super::types::RequestLogEntry) -> Result<(), String> {
    with_db(|conn| {
        conn.execute(
            "INSERT INTO gateway_request_logs (trace_id, timestamp, method, path, status_code, duration_ms, account_email, model, input_tokens, output_tokens, error_message, api_key_prefix)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.trace_id,
                entry.timestamp,
                entry.method,
                entry.path,
                entry.status_code,
                entry.duration_ms,
                entry.account_email,
                entry.model,
                entry.input_tokens,
                entry.output_tokens,
                entry.error_message,
                entry.api_key_prefix,
            ],
        )?;
        Ok(())
    })
}

pub fn query_request_logs(
    query: &super::types::RequestLogQuery,
) -> Result<Vec<super::types::RequestLogEntry>, String> {
    with_db(|conn| {
        let limit = query.limit.unwrap_or(100);
        let offset = query.offset.unwrap_or(0);

        let mut sql = String::from(
            "SELECT id, trace_id, timestamp, method, path, status_code, duration_ms, account_email, model, input_tokens, output_tokens, error_message, api_key_prefix
             FROM gateway_request_logs WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(code) = query.status_code {
            sql.push_str(&format!(
                " AND status_code = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(code as i32));
        }
        if let Some(ref model) = query.model {
            sql.push_str(&format!(" AND model = ?{}", param_values.len() + 1));
            param_values.push(Box::new(model.clone()));
        }
        if let Some(start) = query.start_time {
            sql.push_str(&format!(" AND timestamp >= ?{}", param_values.len() + 1));
            param_values.push(Box::new(start));
        }
        if let Some(end) = query.end_time {
            sql.push_str(&format!(" AND timestamp <= ?{}", param_values.len() + 1));
            param_values.push(Box::new(end));
        }

        sql.push_str(&format!(
            " ORDER BY timestamp DESC LIMIT ?{} OFFSET ?{}",
            param_values.len() + 1,
            param_values.len() + 2
        ));
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_ref.as_slice(), |row| {
            Ok(super::types::RequestLogEntry {
                id: row.get(0)?,
                trace_id: row.get(1)?,
                timestamp: row.get(2)?,
                method: row.get(3)?,
                path: row.get(4)?,
                status_code: row.get::<_, i32>(5)? as u16,
                duration_ms: row.get(6)?,
                account_email: row.get(7)?,
                model: row.get(8)?,
                input_tokens: row.get(9)?,
                output_tokens: row.get(10)?,
                error_message: row.get(11)?,
                api_key_prefix: row.get(12)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })
}

pub fn get_request_log_summary() -> Result<super::types::RequestLogSummary, String> {
    with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT COUNT(*), SUM(CASE WHEN status_code < 400 THEN 1 ELSE 0 END),
                    SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END),
                    AVG(duration_ms), COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0)
             FROM gateway_request_logs",
        )?;
        let summary = stmt.query_row([], |row| {
            Ok(super::types::RequestLogSummary {
                total_requests: row.get(0)?,
                success_count: row.get(1)?,
                error_count: row.get(2)?,
                avg_duration_ms: row.get::<_, f64>(3).unwrap_or(0.0),
                total_input_tokens: row.get(4)?,
                total_output_tokens: row.get(5)?,
            })
        })?;
        Ok(summary)
    })
}

pub fn clear_request_logs() -> Result<(), String> {
    with_db(|conn| {
        conn.execute("DELETE FROM gateway_request_logs", [])?;
        Ok(())
    })
}
