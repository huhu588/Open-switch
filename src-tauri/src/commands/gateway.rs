use crate::modules::gateway;

#[tauri::command]
pub async fn start_gateway() -> Result<String, String> {
    let config = gateway::config::get_gateway_config();
    gateway::start_gateway(config.port).await?;
    Ok(format!("网关已启动，端口: {}", config.port))
}

#[tauri::command]
pub async fn stop_gateway() -> Result<String, String> {
    gateway::stop_gateway().await?;
    Ok("网关已停止".to_string())
}

#[tauri::command]
pub fn get_gateway_status() -> Result<gateway::types::GatewayStatus, String> {
    let config = gateway::config::get_gateway_config();
    let accounts = gateway::db::list_accounts().unwrap_or_default();
    let active_accounts = accounts
        .iter()
        .filter(|a| a.status == gateway::types::AccountStatus::Active)
        .count();
    let api_keys = gateway::db::list_api_keys().unwrap_or_default();
    let summary = gateway::db::get_request_log_summary().unwrap_or(gateway::types::RequestLogSummary {
        total_requests: 0,
        success_count: 0,
        error_count: 0,
        avg_duration_ms: 0.0,
        total_input_tokens: 0,
        total_output_tokens: 0,
    });

    let synced_accounts = accounts
        .iter()
        .filter(|a| a.source.as_deref() == Some("synced"))
        .count();
    let platform_stats = crate::modules::gateway::account_pool_bridge::get_platform_account_stats();

    Ok(gateway::types::GatewayStatus {
        running: gateway::is_gateway_running(),
        port: config.port,
        total_accounts: accounts.len(),
        active_accounts,
        total_api_keys: api_keys.len(),
        total_requests: summary.total_requests,
        uptime_seconds: None,
        synced_accounts,
        platform_stats: if platform_stats.is_empty() {
            None
        } else {
            Some(platform_stats)
        },
    })
}

#[tauri::command]
pub fn get_gateway_config() -> Result<gateway::types::GatewayConfig, String> {
    Ok(gateway::config::get_gateway_config())
}

#[tauri::command]
pub fn save_gateway_config(config: gateway::types::GatewayConfig) -> Result<(), String> {
    gateway::config::save_gateway_config(&config)
}

#[tauri::command]
pub fn list_gateway_accounts() -> Result<Vec<gateway::types::GatewayAccount>, String> {
    gateway::db::list_accounts()
}

#[tauri::command]
pub fn add_gateway_account(
    email: String,
    access_token: String,
    refresh_token: Option<String>,
    tags: Option<String>,
    group_name: Option<String>,
    proxy_url: Option<String>,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    gateway::db::insert_account(
        &id,
        &email,
        &access_token,
        refresh_token.as_deref(),
        tags.as_deref(),
        group_name.as_deref(),
        proxy_url.as_deref(),
        None,
        Some("manual"),
    )
}

#[tauri::command]
pub fn delete_gateway_account(id: String) -> Result<(), String> {
    gateway::db::delete_account(&id)
}

#[tauri::command]
pub fn import_gateway_accounts(
    accounts: Vec<gateway::types::AccountImportPayload>,
) -> Result<usize, String> {
    gateway::account_pool::import_accounts(accounts)
}

#[tauri::command]
pub fn export_gateway_accounts() -> Result<Vec<gateway::types::GatewayAccount>, String> {
    gateway::account_pool::export_accounts()
}

#[tauri::command]
pub fn list_api_keys() -> Result<Vec<gateway::types::GatewayApiKey>, String> {
    gateway::db::list_api_keys()
}

#[tauri::command]
pub fn create_api_key(
    payload: gateway::types::ApiKeyCreatePayload,
) -> Result<(String, gateway::types::GatewayApiKey), String> {
    gateway::api_key::generate_api_key(&payload)
}

#[tauri::command]
pub fn delete_api_key(id: String) -> Result<(), String> {
    gateway::db::delete_api_key(&id)
}

#[tauri::command]
pub fn toggle_api_key(id: String, enabled: bool) -> Result<(), String> {
    gateway::db::toggle_api_key(&id, enabled)
}

#[tauri::command]
pub fn list_request_logs(
    query: gateway::types::RequestLogQuery,
) -> Result<Vec<gateway::types::RequestLogEntry>, String> {
    gateway::request_log::query_logs(&query)
}

#[tauri::command]
pub fn get_request_log_summary() -> Result<gateway::types::RequestLogSummary, String> {
    gateway::request_log::get_summary()
}

#[tauri::command]
pub fn clear_request_logs() -> Result<(), String> {
    gateway::request_log::clear_logs()
}

#[tauri::command]
pub fn sync_accounts_to_gateway() -> Result<crate::modules::gateway::account_pool_bridge::SyncResult, String> {
    crate::modules::gateway::account_pool_bridge::sync_platform_accounts_to_gateway()
}

#[tauri::command]
pub fn get_platform_account_stats() -> Result<std::collections::HashMap<String, usize>, String> {
    Ok(crate::modules::gateway::account_pool_bridge::get_platform_account_stats())
}

#[tauri::command]
pub async fn sync_accounts_to_sub2api() -> Result<crate::modules::subprocess::sub2api_sync::Sub2apiSyncResult, String> {
    let port = crate::modules::subprocess::get_sub2api_port();
    if port == 0 {
        return Err("Sub2api 未在运行".to_string());
    }
    crate::modules::subprocess::sub2api_sync::sync_accounts_to_sub2api(port).await
}

#[tauri::command]
pub fn get_unified_account_pool() -> Result<Vec<gateway::types::GatewayAccount>, String> {
    let mut all_accounts = gateway::db::list_accounts().unwrap_or_default();
    
    let bridge_accounts = crate::modules::gateway::account_pool_bridge::collect_platform_accounts();
    for pa in bridge_accounts {
        if !all_accounts.iter().any(|a| a.id == pa.account.id) {
            all_accounts.push(pa.account);
        }
    }
    
    Ok(all_accounts)
}
