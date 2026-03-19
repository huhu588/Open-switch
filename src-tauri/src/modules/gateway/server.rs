use super::router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub async fn run_gateway_server(
    port: u16,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) -> Result<(), String> {
    let app = router::create_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| format!("绑定端口 {} 失败: {}", port, e))?;

    tracing::info!("[Gateway] HTTP 服务器启动于 {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
            tracing::info!("[Gateway] 收到关闭信号，正在优雅停止");
        })
        .await
        .map_err(|e| format!("服务器错误: {}", e))?;

    tracing::info!("[Gateway] HTTP 服务器已停止");
    Ok(())
}
