use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ApiProtocol {
    OpenAIChat,
    CodexResponses,
    AnthropicMessages,
    Unknown,
}

pub fn detect_protocol(path: &str) -> ApiProtocol {
    if path.starts_with("/v1/chat/completions") {
        ApiProtocol::OpenAIChat
    } else if path.starts_with("/v1/responses") {
        ApiProtocol::CodexResponses
    } else if path.starts_with("/v1/messages") {
        ApiProtocol::AnthropicMessages
    } else {
        ApiProtocol::Unknown
    }
}

pub fn extract_model_from_body(body: &[u8]) -> Option<String> {
    let json: Value = serde_json::from_slice(body).ok()?;
    json.get("model")?.as_str().map(|s| s.to_string())
}

pub fn is_streaming_request(body: &[u8]) -> bool {
    if let Ok(json) = serde_json::from_slice::<Value>(body) {
        json.get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        false
    }
}

pub fn resolve_upstream_for_platform(
    platform: Option<&str>,
    config_upstreams: &Option<std::collections::HashMap<String, String>>,
    default_upstream: &str,
) -> String {
    if let (Some(p), Some(upstreams)) = (platform, config_upstreams) {
        if let Some(url) = upstreams.get(p) {
            return url.clone();
        }
    }
    default_upstream.to_string()
}

pub fn rewrite_request_for_upstream(
    path: &str,
    body: &[u8],
    upstream_base_url: &str,
) -> Result<(String, Vec<u8>), String> {
    let protocol = detect_protocol(path);
    let upstream_path = match protocol {
        ApiProtocol::OpenAIChat => "/v1/chat/completions".to_string(),
        ApiProtocol::CodexResponses => {
            if path.contains("/compact") {
                "/v1/responses/compact".to_string()
            } else {
                "/v1/responses".to_string()
            }
        }
        ApiProtocol::AnthropicMessages => "/v1/responses".to_string(),
        ApiProtocol::Unknown => path.to_string(),
    };

    let upstream_url = format!("{}{}", upstream_base_url.trim_end_matches('/'), upstream_path);

    let rewritten_body = if protocol == ApiProtocol::AnthropicMessages {
        convert_anthropic_to_codex(body)?
    } else {
        body.to_vec()
    };

    Ok((upstream_url, rewritten_body))
}

fn convert_anthropic_to_codex(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut json: Value =
        serde_json::from_slice(body).map_err(|e| format!("解析请求体失败: {}", e))?;

    if let Some(messages) = json.get("messages").cloned() {
        let input = convert_anthropic_messages_to_codex_input(&messages);
        json["input"] = input;
        json.as_object_mut().map(|o| o.remove("messages"));
    }

    if let Some(max_tokens) = json.get("max_tokens").cloned() {
        json["max_output_tokens"] = max_tokens;
        json.as_object_mut().map(|o| o.remove("max_tokens"));
    }

    serde_json::to_vec(&json).map_err(|e| format!("序列化请求体失败: {}", e))
}

fn convert_anthropic_messages_to_codex_input(messages: &Value) -> Value {
    if let Some(arr) = messages.as_array() {
        let converted: Vec<Value> = arr
            .iter()
            .map(|msg| {
                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("user");
                let content = msg
                    .get("content")
                    .cloned()
                    .unwrap_or(Value::String(String::new()));

                serde_json::json!({
                    "role": role,
                    "content": content,
                })
            })
            .collect();
        Value::Array(converted)
    } else {
        messages.clone()
    }
}

pub fn build_models_response() -> Value {
    serde_json::json!({
        "object": "list",
        "data": [
            {"id": "codex-mini-latest", "object": "model", "owned_by": "openai"},
            {"id": "o4-mini", "object": "model", "owned_by": "openai"},
            {"id": "o3", "object": "model", "owned_by": "openai"},
            {"id": "o3-pro", "object": "model", "owned_by": "openai"},
            {"id": "gpt-4.1", "object": "model", "owned_by": "openai"},
            {"id": "gpt-4.1-mini", "object": "model", "owned_by": "openai"},
            {"id": "gpt-4.1-nano", "object": "model", "owned_by": "openai"},
            {"id": "claude-sonnet-4-20250514", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-4-opus-20250514", "object": "model", "owned_by": "anthropic"},
        ]
    })
}
