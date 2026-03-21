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

/// 处理 Cursor Welfare 请求（OpenAI chat/completions 转发到 cursor2api-go）
pub async fn handle_cursor_welfare(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let is_stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

    let (api_key, base_url) = get_cursor_welfare_config(&headers);
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    let client = reqwest::Client::new();
    let target_url = format!("{}/v1/chat/completions", base_url);

    let req_builder = client
        .post(&target_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body);

    let response = req_builder
        .send()
        .await
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
        let mapped_stream = stream.map(move |chunk| chunk.map(|bytes| bytes.to_vec()));

        let body = Body::from_stream(mapped_stream);
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("text/event-stream"));
        response_headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        Ok((status_code, response_headers, body).into_response())
    } else {
        let response_body = response
            .bytes()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

        if let Ok(json_body) = serde_json::from_slice::<Value>(&response_body) {
            if let Some(usage) = TokenUsage::from_openai_response(&json_body) {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                let _ = log_usage(
                    &state.db,
                    "cursor_welfare",
                    Some("Cursor Welfare"),
                    AppType::CursorWelfare,
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

/// 处理 Claude -> CursorWelfare 协议适配
/// 接收 Anthropic Messages 格式，转为 OpenAI chat/completions 发到 cursor2api-go，
/// 再把 OpenAI 响应转回 Anthropic Messages 格式
pub async fn handle_cursor_welfare_claude_compat(
    State(state): State<ProxyState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let model = body.get("model").and_then(|v| v.as_str()).unwrap_or("claude-sonnet-4.6").to_string();

    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    if api_key.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "缺少 API Key".to_string()));
    }

    let cursor_welfare_port = crate::modules::subprocess::get_cursor_welfare_port();
    if cursor_welfare_port == 0 {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Cursor 福利服务未运行".to_string(),
        ));
    }
    let upstream = format!("http://localhost:{}", cursor_welfare_port);

    let openai_body = anthropic_to_openai(&body, &model);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", upstream))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&openai_body)
        .send()
        .await
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

    let response_body = response
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

    if let Ok(openai_resp) = serde_json::from_slice::<Value>(&response_body) {
        if let Some(usage) = TokenUsage::from_openai_response(&openai_resp) {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            let _ = log_usage(
                &state.db,
                "cursor_welfare",
                Some("Cursor Welfare"),
                AppType::CursorWelfare,
                &model,
                usage,
                latency_ms,
                status_code.as_u16(),
            );
        }
        let anthropic_resp = openai_to_anthropic(&openai_resp, &model);
        let resp_bytes = serde_json::to_vec(&anthropic_resp).unwrap_or_default();

        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok((status_code, response_headers, resp_bytes).into_response())
    } else {
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok((status_code, response_headers, response_body.to_vec()).into_response())
    }
}

/// 处理 Gemini -> CursorWelfare 协议适配
pub async fn handle_cursor_welfare_gemini_compat(
    State(state): State<ProxyState>,
    Path(path): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let start_time = Instant::now();
    let model = extract_gemini_model(&path).unwrap_or("unknown".to_string());

    let api_key = headers
        .get("x-goog-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let cursor_welfare_port = crate::modules::subprocess::get_cursor_welfare_port();
    if cursor_welfare_port == 0 {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Cursor 福利服务未运行".to_string(),
        ));
    }
    let upstream = format!("http://localhost:{}", cursor_welfare_port);

    let openai_body = gemini_to_openai(&body, &model);
    let cw_api_key = if api_key.is_empty() {
        "cursor-welfare".to_string()
    } else {
        api_key
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", upstream))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", cw_api_key))
        .json(&openai_body)
        .send()
        .await
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

    let response_body = response
        .bytes()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("读取响应失败: {e}")))?;

    if let Ok(openai_resp) = serde_json::from_slice::<Value>(&response_body) {
        if let Some(usage) = TokenUsage::from_openai_response(&openai_resp) {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            let _ = log_usage(
                &state.db,
                "cursor_welfare",
                Some("Cursor Welfare"),
                AppType::CursorWelfare,
                &model,
                usage,
                latency_ms,
                status_code.as_u16(),
            );
        }
        let gemini_resp = openai_to_gemini(&openai_resp, &model);
        let resp_bytes = serde_json::to_vec(&gemini_resp).unwrap_or_default();

        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok((status_code, response_headers, resp_bytes).into_response())
    } else {
        let mut response_headers = HeaderMap::new();
        response_headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        Ok((status_code, response_headers, response_body.to_vec()).into_response())
    }
}

// ============================================================================
// 协议转换辅助函数
// ============================================================================

/// Anthropic Messages -> OpenAI chat/completions 请求
fn anthropic_to_openai(body: &Value, model: &str) -> Value {
    let mut messages = Vec::new();

    if let Some(system) = body.get("system") {
        if let Some(s) = system.as_str() {
            messages.push(json!({"role": "system", "content": s}));
        } else if let Some(arr) = system.as_array() {
            let text: Vec<String> = arr
                .iter()
                .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                .map(|s| s.to_string())
                .collect();
            if !text.is_empty() {
                messages.push(json!({"role": "system", "content": text.join("\n")}));
            }
        }
    }

    if let Some(msgs) = body.get("messages").and_then(|v| v.as_array()) {
        for msg in msgs {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let openai_role = match role {
                "assistant" => "assistant",
                _ => "user",
            };
            if let Some(content) = msg.get("content") {
                if content.is_string() {
                    messages.push(json!({"role": openai_role, "content": content}));
                } else if let Some(arr) = content.as_array() {
                    let text: Vec<String> = arr
                        .iter()
                        .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                        .map(|s| s.to_string())
                        .collect();
                    messages.push(json!({"role": openai_role, "content": text.join("\n")}));
                }
            }
        }
    }

    let max_tokens = body
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(8192);

    json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
    })
}

/// OpenAI chat/completions 响应 -> Anthropic Messages 响应
fn openai_to_anthropic(resp: &Value, model: &str) -> Value {
    let content_text = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let stop_reason = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(|r| r.as_str())
        .map(|r| match r {
            "stop" => "end_turn",
            "length" => "max_tokens",
            _ => "end_turn",
        })
        .unwrap_or("end_turn");

    let usage = resp.get("usage");
    let input_tokens = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    json!({
        "id": resp.get("id").and_then(|v| v.as_str()).unwrap_or("msg_cursor_welfare"),
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": [{"type": "text", "text": content_text}],
        "stop_reason": stop_reason,
        "stop_sequence": null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    })
}

/// Gemini generateContent 请求 -> OpenAI chat/completions 请求
fn gemini_to_openai(body: &Value, model: &str) -> Value {
    let mut messages = Vec::new();

    if let Some(contents) = body.get("contents").and_then(|v| v.as_array()) {
        for content in contents {
            let role = content.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let openai_role = match role {
                "model" => "assistant",
                _ => "user",
            };
            if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                let text: Vec<String> = parts
                    .iter()
                    .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                    .map(|s| s.to_string())
                    .collect();
                if !text.is_empty() {
                    messages.push(json!({"role": openai_role, "content": text.join("\n")}));
                }
            }
        }
    }

    if let Some(sys_inst) = body.get("systemInstruction") {
        if let Some(parts) = sys_inst.get("parts").and_then(|p| p.as_array()) {
            let text: Vec<String> = parts
                .iter()
                .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                .map(|s| s.to_string())
                .collect();
            if !text.is_empty() {
                messages.insert(0, json!({"role": "system", "content": text.join("\n")}));
            }
        }
    }

    json!({
        "model": model,
        "messages": messages,
        "max_tokens": 8192,
    })
}

/// OpenAI chat/completions 响应 -> Gemini generateContent 响应
fn openai_to_gemini(resp: &Value, model: &str) -> Value {
    let content_text = resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("");

    let usage = resp.get("usage");
    let prompt_tokens = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let completion_tokens = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    json!({
        "candidates": [{
            "content": {
                "parts": [{"text": content_text}],
                "role": "model"
            },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": prompt_tokens,
            "candidatesTokenCount": completion_tokens,
            "totalTokenCount": prompt_tokens + completion_tokens
        },
        "modelVersion": model
    })
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

/// 从请求头获取 Cursor Welfare 配置
fn get_cursor_welfare_config(headers: &HeaderMap) -> (String, String) {
    let api_key = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim_start_matches("Bearer ").to_string())
        .unwrap_or_default();

    let cursor_welfare_port = crate::modules::subprocess::get_cursor_welfare_port();
    let base_url = if cursor_welfare_port > 0 {
        format!("http://localhost:{}", cursor_welfare_port)
    } else {
        headers
            .get("x-base-url")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http://localhost:8002")
            .to_string()
    };

    (api_key, base_url)
}
