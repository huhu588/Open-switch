//! 请求处理器
//!
//! 处理各种 API 端点的 HTTP 请求

use super::server::ProxyState;
use super::types::*;
use super::usage::{log_usage, TokenUsage};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use serde_json::{json, Value};
use std::time::Instant;

/// 健康检查
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

/// 获取服务状态
pub async fn get_status(State(state): State<ProxyState>) -> Result<Json<ProxyStatus>, StatusCode> {
    let mut status = state.status.read().await.clone();
    
    if let Some(start) = *state.start_time.read().await {
        status.uptime_seconds = start.elapsed().as_secs();
    }
    
    Ok(Json(status))
}

/// 处理 Claude API 请求
pub async fn handle_claude(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

    // 获取 API Key 和 Base URL
    let (api_key, base_url) = get_claude_config(&headers);
    
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    // 构建转发请求
    let client = reqwest::Client::new();
    let target_url = format!("{}/v1/messages", base_url);
    
    let mut req_builder = client.post(&target_url)
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body);

    // 复制其他头部
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !["host", "content-length", "x-api-key", "authorization"].contains(&key_str.as_str()) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(key.as_str(), v);
            }
        }
    }

    // 发送请求
    let response = req_builder.send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("转发请求失败: {e}")))?;

    let status_code = response.status();
    
    // 更新统计
    {
        let mut s = state.status.write().await;
        s.total_requests += 1;
        if status_code.is_success() {
            s.success_requests += 1;
        } else {
            s.failed_requests += 1;
        }
    }

    if is_stream {
        // 流式响应处理
        let stream = response.bytes_stream();
        let _db = state.db.clone();
        let _model_clone = model.clone();
        
        let mapped_stream = stream.map(move |chunk| {
            chunk.map(|bytes| bytes.to_vec())
        });

        // 创建响应
        let body = Body::from_stream(mapped_stream);
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("text/event-stream"));
        response_headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        
        Ok((status_code, response_headers, body).into_response())
    } else {
        // 非流式响应
        let response_body = response.bytes().await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

        // 解析并记录使用量
        if let Ok(json_body) = serde_json::from_slice::<Value>(&response_body) {
            if let Some(usage) = TokenUsage::from_claude_response(&json_body) {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                let _ = log_usage(
                    &state.db,
                    "default",
                    None,
                    AppType::Claude,
                    &model,
                    usage,
                    latency_ms,
                    status_code.as_u16(),
                );
            }
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        
        Ok((status_code, response_headers, response_body.to_vec()).into_response())
    }
}

/// 处理 Codex API 请求 (Chat Completions)
pub async fn handle_codex(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

    // 获取 API Key 和 Base URL
    let (api_key, base_url) = get_openai_config(&headers);
    
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    // 构建转发请求
    let client = reqwest::Client::new();
    let target_url = format!("{}/chat/completions", base_url);
    
    let mut req_builder = client.post(&target_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body);

    // 复制其他头部
    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !["host", "content-length", "authorization"].contains(&key_str.as_str()) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(key.as_str(), v);
            }
        }
    }

    // 发送请求
    let response = req_builder.send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("转发请求失败: {e}")))?;

    let status_code = response.status();
    
    // 更新统计
    {
        let mut s = state.status.write().await;
        s.total_requests += 1;
        if status_code.is_success() {
            s.success_requests += 1;
        } else {
            s.failed_requests += 1;
        }
    }

    if is_stream {
        let stream = response.bytes_stream();
        let mapped_stream = stream.map(move |chunk| {
            chunk.map(|bytes| bytes.to_vec())
        });

        let body = Body::from_stream(mapped_stream);
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("text/event-stream"));
        response_headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        
        Ok((status_code, response_headers, body).into_response())
    } else {
        let response_body = response.bytes().await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

        // 解析并记录使用量
        if let Ok(json_body) = serde_json::from_slice::<Value>(&response_body) {
            if let Some(usage) = TokenUsage::from_openai_response(&json_body) {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                let _ = log_usage(
                    &state.db,
                    "default",
                    None,
                    AppType::Codex,
                    &model,
                    usage,
                    latency_ms,
                    status_code.as_u16(),
                );
            }
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        
        Ok((status_code, response_headers, response_body.to_vec()).into_response())
    }
}

/// 处理 Codex Responses API 请求
pub async fn handle_codex_responses(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

    let (api_key, base_url) = get_openai_config(&headers);
    
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    let client = reqwest::Client::new();
    let target_url = format!("{}/responses", base_url);
    
    let mut req_builder = client.post(&target_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body);

    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !["host", "content-length", "authorization"].contains(&key_str.as_str()) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(key.as_str(), v);
            }
        }
    }

    let response = req_builder.send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("转发请求失败: {e}")))?;

    let status_code = response.status();
    
    {
        let mut s = state.status.write().await;
        s.total_requests += 1;
        if status_code.is_success() {
            s.success_requests += 1;
        } else {
            s.failed_requests += 1;
        }
    }

    if is_stream {
        let stream = response.bytes_stream();
        let mapped_stream = stream.map(move |chunk| {
            chunk.map(|bytes| bytes.to_vec())
        });

        let body = Body::from_stream(mapped_stream);
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("text/event-stream"));
        response_headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        
        Ok((status_code, response_headers, body).into_response())
    } else {
        let response_body = response.bytes().await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

        if let Ok(json_body) = serde_json::from_slice::<Value>(&response_body) {
            if let Some(usage) = TokenUsage::from_codex_response(&json_body) {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                let _ = log_usage(
                    &state.db,
                    "default",
                    None,
                    AppType::Codex,
                    &model,
                    usage,
                    latency_ms,
                    status_code.as_u16(),
                );
            }
        }

        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        
        Ok((status_code, response_headers, response_body.to_vec()).into_response())
    }
}

/// 处理 Gemini API 请求
pub async fn handle_gemini(
    State(state): State<ProxyState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    
    // 从路径提取模型名称
    let model = extract_gemini_model(&path).unwrap_or("unknown".to_string());

    // 获取 API Key 和 Base URL
    let (api_key, base_url) = get_gemini_config(&headers);
    
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    // 构建转发请求
    let client = reqwest::Client::new();
    let target_url = format!("{}/v1beta/{}", base_url, path);
    
    let mut req_builder = client.post(&target_url)
        .header("Content-Type", "application/json")
        .query(&[("key", &api_key)])
        .json(&body);

    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !["host", "content-length"].contains(&key_str.as_str()) {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(key.as_str(), v);
            }
        }
    }

    let response = req_builder.send().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("转发请求失败: {e}")))?;

    let status_code = response.status();
    
    {
        let mut s = state.status.write().await;
        s.total_requests += 1;
        if status_code.is_success() {
            s.success_requests += 1;
        } else {
            s.failed_requests += 1;
        }
    }

    let response_body = response.bytes().await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

    // 解析并记录使用量
    if let Ok(json_body) = serde_json::from_slice::<Value>(&response_body) {
        if let Some(usage) = TokenUsage::from_gemini_response(&json_body) {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            let _ = log_usage(
                &state.db,
                "default",
                None,
                AppType::Gemini,
                &model,
                usage,
                latency_ms,
                status_code.as_u16(),
            );
        }
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    
    Ok((status_code, response_headers, response_body.to_vec()).into_response())
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 从请求头获取 Claude 配置
fn get_claude_config(headers: &HeaderMap) -> (String, String) {
    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    
    let base_url = headers
        .get("x-base-url")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("https://api.anthropic.com")
        .to_string();
    
    (api_key, base_url)
}

/// 从请求头获取 OpenAI 配置
fn get_openai_config(headers: &HeaderMap) -> (String, String) {
    let api_key = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string())
        .unwrap_or_default();
    
    let base_url = headers
        .get("x-base-url")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("https://api.openai.com/v1")
        .to_string();
    
    (api_key, base_url)
}

/// 从请求头获取 Gemini 配置
fn get_gemini_config(headers: &HeaderMap) -> (String, String) {
    let api_key = headers
        .get("x-goog-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    
    let base_url = headers
        .get("x-base-url")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("https://generativelanguage.googleapis.com")
        .to_string();
    
    (api_key, base_url)
}

/// 从 Gemini API 路径提取模型名称
fn extract_gemini_model(path: &str) -> Option<String> {
    // 路径格式: models/{model}:generateContent
    if path.starts_with("models/") {
        let rest = &path[7..]; // 跳过 "models/"
        if let Some(colon_pos) = rest.find(':') {
            return Some(rest[..colon_pos].to_string());
        }
    }
    None
}
