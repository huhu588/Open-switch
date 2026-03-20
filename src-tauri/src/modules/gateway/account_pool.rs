use super::db;
use super::account_pool_bridge;
use super::types::{GatewayAccount, RouteStrategy};
use std::sync::atomic::{AtomicUsize, Ordering};

static ROUND_ROBIN_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn select_account(strategy: &RouteStrategy) -> Result<GatewayAccount, String> {
    let mut accounts = db::get_active_accounts()?;

    if accounts.is_empty() {
        tracing::info!("[AccountPool] 数据库无可用账号，尝试从平台同步");
        if let Ok(sync_result) = account_pool_bridge::sync_platform_accounts_to_gateway() {
            tracing::info!(
                "[AccountPool] 自动同步完成: 新增 {} 个账号",
                sync_result.added
            );
            accounts = db::get_active_accounts()?;
        }
    }

    if accounts.is_empty() {
        return Err("没有可用的账号（数据库和平台账号池均为空）".to_string());
    }

    let account = match strategy {
        RouteStrategy::RoundRobin => {
            let idx = ROUND_ROBIN_INDEX.fetch_add(1, Ordering::Relaxed) % accounts.len();
            accounts[idx].clone()
        }
        RouteStrategy::LeastUsed | RouteStrategy::QuotaAware => {
            accounts
                .iter()
                .min_by_key(|a| a.last_used_at.unwrap_or(0))
                .unwrap()
                .clone()
        }
        RouteStrategy::Random => {
            use rand::Rng;
            let idx = rand::thread_rng().gen_range(0..accounts.len());
            accounts[idx].clone()
        }
        RouteStrategy::Priority => accounts[0].clone(),
    };

    db::mark_account_used(&account.id)?;
    Ok(account)
}

pub fn cooldown_account(id: &str, duration_seconds: u32) -> Result<(), String> {
    let cooldown_until = chrono::Utc::now().timestamp() + duration_seconds as i64;
    db::with_db(|conn| {
        conn.execute(
            "UPDATE gateway_accounts SET cooldown_until = ?1, status = 'cooldown', updated_at = ?2 WHERE id = ?3",
            rusqlite::params![cooldown_until, chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    })
}

pub fn report_account_error(id: &str) -> Result<(), String> {
    db::with_db(|conn| {
        conn.execute(
            "UPDATE gateway_accounts SET error_count = error_count + 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp(), id],
        )?;

        let error_count: i32 = conn.query_row(
            "SELECT error_count FROM gateway_accounts WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ).unwrap_or(0);

        if error_count >= 5 {
            conn.execute(
                "UPDATE gateway_accounts SET status = 'error' WHERE id = ?1",
                rusqlite::params![id],
            )?;
        }

        Ok(())
    })
}

pub fn reset_account_errors(id: &str) -> Result<(), String> {
    db::with_db(|conn| {
        conn.execute(
            "UPDATE gateway_accounts SET error_count = 0, status = 'active', cooldown_until = NULL, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp(), id],
        )?;
        Ok(())
    })
}

pub fn import_accounts(accounts: Vec<super::types::AccountImportPayload>) -> Result<usize, String> {
    let mut count = 0;
    for account in accounts {
        let id = uuid::Uuid::new_v4().to_string();
        db::insert_account(
            &id,
            &account.email,
            &account.access_token,
            account.refresh_token.as_deref(),
            account.tags.as_deref(),
            account.group_name.as_deref(),
            account.proxy_url.as_deref(),
            None,
            Some("manual"),
        )?;
        count += 1;
    }
    Ok(count)
}

pub fn export_accounts() -> Result<Vec<GatewayAccount>, String> {
    db::list_accounts()
}
