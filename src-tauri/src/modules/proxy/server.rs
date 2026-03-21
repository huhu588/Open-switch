//! HTTP 代理服务器
//!
//! 基于 Axum 的 HTTP 服务器，处理代理请求

use super::{handlers, types::*, ProxyConfig};
use crate::modules::opencode_db::Database;
use crate::opencode_error::AppError;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{oneshot, RwLock};

/// 代理服务器状态（共享）
#[derive(Clone)]
pub struct ProxyState {
    pub db: Arc<Database>,
    pub config: Arc<RwLock<ProxyConfig>>,
    pub status: Arc<RwLock<ProxyStatus>>,
    pub start_time: Arc<RwLock<Option<Instant>>>,
}

/// 代理 HTTP 服务器
pub struct ProxyServer {
    config: ProxyConfig,
    state: ProxyState,
    shutdown_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
}

impl ProxyServer {
    /// 创建新的代理服务器
    pub fn new(config: ProxyConfig, db: Arc<Database>) -> Self {
        let state = ProxyState {
            db,
            config: Arc::new(RwLock::new(config.clone())),
            status: Arc::new(RwLock::new(ProxyStatus::default())),
            start_time: Arc::new(RwLock::new(None)),
        };

        Self {
            config,
            state,
            shutdown_tx: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动代理服务器
    pub async fn start(&self) -> Result<ProxyServerInfo, AppError> {
        // 检查是否已在运行
        if self.shutdown_tx.read().await.is_some() {
            return Err(AppError::Proxy("代理服务器已在运行".to_string()));
        }

        let addr: SocketAddr = format!("{}:{}", self.config.listen_address, self.config.listen_port)
            .parse()
            .map_err(|e| AppError::Proxy(format!("无效的地址: {e}")))?;

        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // 构建路由
        let app = self.build_router();

        // 绑定监听器
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| AppError::Proxy(format!("绑定端口失败: {e}")))?;

        // 保存关闭句柄
        *self.shutdown_tx.write().await = Some(shutdown_tx);

        // 更新状态
        {
            let mut status = self.state.status.write().await;
            status.running = true;
            status.address = self.config.listen_address.clone();
            status.port = self.config.listen_port;
        }

        // 记录启动时间
        *self.state.start_time.write().await = Some(Instant::now());

        // 启动服务器
        let state = self.state.clone();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();

            // 服务器停止后更新状态
            state.status.write().await.running = false;
            *state.start_time.write().await = None;
        });

        Ok(ProxyServerInfo {
            address: self.config.listen_address.clone(),
            port: self.config.listen_port,
            started_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 停止代理服务器
    pub async fn stop(&self) -> Result<(), AppError> {
        if let Some(tx) = self.shutdown_tx.write().await.take() {
            let _ = tx.send(());
            Ok(())
        } else {
            Err(AppError::Proxy("代理服务器未运行".to_string()))
        }
    }

    /// 获取服务器状态
    pub async fn get_status(&self) -> ProxyStatus {
        let mut status = self.state.status.read().await.clone();

        // 计算运行时间
        if let Some(start) = *self.state.start_time.read().await {
            status.uptime_seconds = start.elapsed().as_secs();
        }

        status
    }

    /// 检查服务器是否运行中
    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.read().await.is_some()
    }

    /// 构建路由
    fn build_router(&self) -> Router {
        use tower_http::cors::{Any, CorsLayer};

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            // 健康检查
            .route("/health", get(handlers::health_check))
            .route("/status", get(handlers::get_status))
            // Claude API
            .route("/v1/messages", post(handlers::handle_claude))
            .route("/claude/v1/messages", post(handlers::handle_claude))
            // Codex API (OpenAI Chat Completions)
            .route("/v1/chat/completions", post(handlers::handle_codex))
            .route("/codex/v1/chat/completions", post(handlers::handle_codex))
            // Codex API (OpenAI Responses)
            .route("/v1/responses", post(handlers::handle_codex_responses))
            .route("/codex/v1/responses", post(handlers::handle_codex_responses))
            // Gemini API
            .route("/v1beta/*path", post(handlers::handle_gemini))
            .route("/gemini/v1beta/*path", post(handlers::handle_gemini))
            // Cursor Welfare API (直接 OpenAI 格式转发)
            .route("/cursor-welfare/v1/chat/completions", post(handlers::handle_cursor_welfare))
            // Cursor Welfare Claude 协议适配（Anthropic -> OpenAI -> cursor2api-go）
            .route("/cursor-welfare/v1/messages", post(handlers::handle_cursor_welfare_claude_compat))
            // Cursor Welfare Gemini 协议适配（Gemini -> OpenAI -> cursor2api-go）
            .route("/cursor-welfare/v1beta/*path", post(handlers::handle_cursor_welfare_gemini_compat))
            // 提高请求体大小限制
            .layer(DefaultBodyLimit::max(200 * 1024 * 1024))
            .layer(cors)
            .with_state(self.state.clone())
    }
}
