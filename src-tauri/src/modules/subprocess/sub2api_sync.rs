use crate::modules::gateway::account_pool_bridge;
use crate::modules::logger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sub2apiSyncResult {
    pub total_accounts: usize,
    pub synced: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// 将平台账号同步到 Sub2api（通过 HTTP API）
pub async fn sync_accounts_to_sub2api(sub2api_port: u16) -> Result<Sub2apiSyncResult, String> {
    let platform_accounts = account_pool_bridge::collect_platform_accounts();
    
    if platform_accounts.is_empty() {
        return Ok(Sub2apiSyncResult {
            total_accounts: 0,
            synced: 0,
            failed: 0,
            errors: vec!["没有可用的平台账号".to_string()],
        });
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let base_url = format!("http://localhost:{}", sub2api_port);
    
    // 检查 Sub2api 是否在运行
    let health_url = format!("{}/health", base_url);
    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {}
        _ => return Err("Sub2api 未运行或无法连接".to_string()),
    }

    let mut synced = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for pa in &platform_accounts {
        let payload = serde_json::json!({
            "email": pa.account.email,
            "access_token": pa.account.access_token,
            "refresh_token": pa.account.refresh_token,
            "platform": pa.platform.to_string(),
            "source": "ai-switch",
        });

        let sync_url = format!("{}/api/admin/accounts/sync", base_url);
        match client.post(&sync_url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                synced += 1;
            }
            Ok(resp) => {
                let msg = format!(
                    "同步账号 {} 失败: HTTP {}",
                    pa.account.email,
                    resp.status()
                );
                logger::log_warn(&format!("[Sub2apiSync] {}", msg));
                errors.push(msg);
                failed += 1;
            }
            Err(e) => {
                let msg = format!("同步账号 {} 失败: {}", pa.account.email, e);
                logger::log_warn(&format!("[Sub2apiSync] {}", msg));
                errors.push(msg);
                failed += 1;
            }
        }
    }

    logger::log_info(&format!(
        "[Sub2apiSync] 同步完成: 共 {} 个, 成功 {}, 失败 {}",
        platform_accounts.len(), synced, failed
    ));

    Ok(Sub2apiSyncResult {
        total_accounts: platform_accounts.len(),
        synced,
        failed,
        errors,
    })
}
