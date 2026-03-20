pub mod server;
pub mod router;
pub mod proxy;
pub mod account_pool;
pub mod account_pool_bridge;
pub mod api_key;
pub mod request_log;
pub mod protocol_adapter;
pub mod db;
pub mod config;
pub mod types;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tokio::sync::Mutex;

static GATEWAY_RUNNING: AtomicBool = AtomicBool::new(false);
static GATEWAY_SHUTDOWN: OnceLock<Mutex<Option<tokio::sync::oneshot::Sender<()>>>> =
    OnceLock::new();

pub fn is_gateway_running() -> bool {
    GATEWAY_RUNNING.load(Ordering::Relaxed)
}

pub async fn start_gateway(port: u16) -> Result<(), String> {
    if GATEWAY_RUNNING.load(Ordering::Relaxed) {
        return Err("网关已在运行".to_string());
    }

    db::init_gateway_db().map_err(|e| format!("初始化网关数据库失败: {}", e))?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let shutdown_mutex = GATEWAY_SHUTDOWN.get_or_init(|| Mutex::new(None));
    *shutdown_mutex.lock().await = Some(shutdown_tx);

    GATEWAY_RUNNING.store(true, Ordering::Relaxed);

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run_gateway_server(port, shutdown_rx).await {
            tracing::error!("[Gateway] 服务器异常退出: {}", e);
        }
        GATEWAY_RUNNING.store(false, Ordering::Relaxed);
    });

    tracing::info!("[Gateway] 网关服务已启动，监听端口: {}", port);

    tokio::spawn(async move {
        let _ = server_handle.await;
        tracing::info!("[Gateway] 网关服务已停止");
    });

    Ok(())
}

pub async fn stop_gateway() -> Result<(), String> {
    if !GATEWAY_RUNNING.load(Ordering::Relaxed) {
        return Err("网关未在运行".to_string());
    }

    let shutdown_mutex = GATEWAY_SHUTDOWN
        .get()
        .ok_or("网关未初始化")?;

    let sender = shutdown_mutex.lock().await.take();
    if let Some(tx) = sender {
        let _ = tx.send(());
    }

    GATEWAY_RUNNING.store(false, Ordering::Relaxed);
    tracing::info!("[Gateway] 网关服务正在关闭");
    Ok(())
}
