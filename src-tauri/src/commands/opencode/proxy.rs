//! 代理服务器相关命令

use crate::modules::opencode_db::schema::{ModelTrendData, ProviderStats, UsageSummary, UsageTrend};
use crate::modules::opencode_db::Database;
use crate::modules::proxy::{ProxyServerInfo, ProxyService, ProxyStatus, ProxyTakeoverStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// 代理服务器状态
pub struct ProxyServiceState(pub Arc<RwLock<Option<ProxyService>>>);

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfigResponse {
    pub proxy_enabled: bool,
    pub listen_address: String,
    pub listen_port: u16,
    pub takeover_claude: bool,
    pub takeover_codex: bool,
    pub takeover_gemini: bool,
}

/// 初始化代理服务
#[tauri::command]
pub async fn init_proxy_service(
    db: State<'_, Arc<Database>>,
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let service = ProxyService::new(db.inner().clone());
    *proxy_state.0.write().await = Some(service);
    Ok(())
}

/// 启动代理服务器
#[tauri::command]
pub async fn start_proxy(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<ProxyServerInfo, String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    service.start().await.map_err(|e| e.to_string())
}

/// 停止代理服务器
#[tauri::command]
pub async fn stop_proxy(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    service.stop().await.map_err(|e| e.to_string())
}

/// 获取代理状态
#[tauri::command]
pub async fn get_proxy_status(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<ProxyStatus, String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    Ok(service.get_status().await)
}

/// 检查代理是否运行
#[tauri::command]
pub async fn is_proxy_running(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<bool, String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    Ok(service.is_running().await)
}

/// 启动代理并接管配置
#[tauri::command]
pub async fn start_proxy_with_takeover(
    apps: Vec<String>,
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<ProxyServerInfo, String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    
    let apps_refs: Vec<&str> = apps.iter().map(|s| s.as_str()).collect();
    service.start_with_takeover(&apps_refs).await.map_err(|e| e.to_string())
}

/// 停止代理并恢复配置
#[tauri::command]
pub async fn stop_proxy_with_restore(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    service.stop_with_restore().await.map_err(|e| e.to_string())
}

/// 获取接管状态
#[tauri::command]
pub async fn get_takeover_status(
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<ProxyTakeoverStatus, String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    service.get_takeover_status().map_err(|e| e.to_string())
}

/// 为指定应用设置接管
#[tauri::command]
pub async fn set_takeover_for_app(
    app_type: String,
    enabled: bool,
    proxy_state: State<'_, ProxyServiceState>,
) -> Result<(), String> {
    let guard = proxy_state.0.read().await;
    let service = guard.as_ref().ok_or("代理服务未初始化")?;
    service.set_takeover_for_app(&app_type, enabled).await.map_err(|e| e.to_string())
}

/// 获取代理配置
#[tauri::command]
pub async fn get_proxy_config(
    db: State<'_, Arc<Database>>,
) -> Result<ProxyConfigResponse, String> {
    let config = db.get_proxy_config().map_err(|e| e.to_string())?;
    Ok(ProxyConfigResponse {
        proxy_enabled: config.proxy_enabled,
        listen_address: config.listen_address,
        listen_port: config.listen_port,
        takeover_claude: config.takeover_claude,
        takeover_codex: config.takeover_codex,
        takeover_gemini: config.takeover_gemini,
    })
}

/// 更新代理配置
#[tauri::command]
pub async fn update_proxy_config(
    config: ProxyConfigResponse,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    let config_db = crate::modules::opencode_db::schema::ProxyConfigDb {
        proxy_enabled: config.proxy_enabled,
        listen_address: config.listen_address,
        listen_port: config.listen_port,
        takeover_claude: config.takeover_claude,
        takeover_codex: config.takeover_codex,
        takeover_gemini: config.takeover_gemini,
    };
    db.update_proxy_config(&config_db).map_err(|e| e.to_string())
}

// ==================== 统计查询命令 ====================

/// 获取使用量摘要
#[tauri::command]
pub async fn get_proxy_usage_summary(
    period: String,
    db: State<'_, Arc<Database>>,
) -> Result<UsageSummary, String> {
    let (start_ts, end_ts) = get_time_range(&period);
    db.get_usage_summary(start_ts, end_ts).map_err(|e| e.to_string())
}

/// 获取使用趋势
#[tauri::command]
pub async fn get_proxy_usage_trend(
    period: String,
    provider_id: Option<String>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<UsageTrend>, String> {
    let (start_ts, end_ts) = get_time_range(&period);
    db.get_usage_trend(start_ts, end_ts, &period, provider_id.as_deref())
        .map_err(|e| e.to_string())
}

/// 获取按模型分组的使用趋势（用于堆叠柱形图）
#[tauri::command]
pub async fn get_proxy_usage_trend_by_model(
    period: String,
    provider_id: Option<String>,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ModelTrendData>, String> {
    let (start_ts, end_ts) = get_time_range(&period);
    db.get_usage_trend_by_model(start_ts, end_ts, &period, provider_id.as_deref())
        .map_err(|e| e.to_string())
}

/// 获取各服务商统计
#[tauri::command]
pub async fn get_provider_stats(
    period: String,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ProviderStats>, String> {
    let (start_ts, end_ts) = get_time_range(&period);
    db.get_provider_stats(start_ts, end_ts).map_err(|e| e.to_string())
}

/// 清空使用统计
#[tauri::command]
pub async fn clear_proxy_usage_stats(
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.clear_usage_stats().map_err(|e| e.to_string())
}

// ==================== 辅助函数 ====================

/// 计算时间范围
fn get_time_range(period: &str) -> (Option<i64>, Option<i64>) {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    
    let start = match period {
        "24h" => Some(now - 24 * 3600),
        "7d" => Some(now - 7 * 24 * 3600),
        "30d" => Some(now - 30 * 24 * 3600),
        "all" => None,
        _ => None,
    };
    
    (start, Some(now))
}
