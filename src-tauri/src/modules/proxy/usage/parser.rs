//! Response Parser - 从 API 响应中提取 token 使用量
//!
//! 支持多种 API 格式：
//! - Claude API (非流式和流式)
//! - OpenAI API (非流式和流式)
//! - Codex API (非流式和流式)
//! - Gemini API (非流式和流式)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Token 使用量统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
    /// 从响应中提取的实际模型名称
    pub model: Option<String>,
}

impl TokenUsage {
    /// 从 Claude API 非流式响应解析
    pub fn from_claude_response(body: &Value) -> Option<Self> {
        let usage = body.get("usage")?;
        let model = body.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());

        Some(Self {
            input_tokens: usage.get("input_tokens")?.as_u64()? as u32,
            output_tokens: usage.get("output_tokens")?.as_u64()? as u32,
            cache_read_tokens: usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            cache_creation_tokens: usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            model,
        })
    }

    /// 从 Claude API 流式响应事件解析
    #[allow(dead_code)]
    pub fn from_claude_stream_events(events: &[Value]) -> Option<Self> {
        let mut usage = Self::default();
        let mut model: Option<String> = None;

        for event in events {
            if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                match event_type {
                    "message_start" => {
                        if model.is_none() {
                            if let Some(message) = event.get("message") {
                                if let Some(m) = message.get("model").and_then(|v| v.as_str()) {
                                    model = Some(m.to_string());
                                }
                            }
                        }
                        if let Some(msg_usage) = event.get("message").and_then(|m| m.get("usage")) {
                            if let Some(input) = msg_usage.get("input_tokens").and_then(|v| v.as_u64()) {
                                usage.input_tokens = input as u32;
                            }
                            usage.cache_read_tokens = msg_usage
                                .get("cache_read_input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            usage.cache_creation_tokens = msg_usage
                                .get("cache_creation_input_tokens")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                        }
                    }
                    "message_delta" => {
                        if let Some(delta_usage) = event.get("usage") {
                            if let Some(output) = delta_usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                usage.output_tokens = output as u32;
                            }
                            if usage.input_tokens == 0 {
                                if let Some(input) = delta_usage.get("input_tokens").and_then(|v| v.as_u64()) {
                                    usage.input_tokens = input as u32;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if usage.input_tokens > 0 || usage.output_tokens > 0 {
            usage.model = model;
            Some(usage)
        } else {
            None
        }
    }

    /// 从 OpenAI Chat Completions API 响应解析
    pub fn from_openai_response(body: &Value) -> Option<Self> {
        let usage = body.get("usage")?;
        let model = body.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());

        let prompt_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64())?;
        let completion_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64())?;

        let cached_tokens = usage
            .get("prompt_tokens_details")
            .and_then(|d| d.get("cached_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        Some(Self {
            input_tokens: prompt_tokens as u32,
            output_tokens: completion_tokens as u32,
            cache_read_tokens: cached_tokens,
            cache_creation_tokens: 0,
            model,
        })
    }

    /// 从 Codex API 响应解析 (input_tokens/output_tokens 格式)
    pub fn from_codex_response(body: &Value) -> Option<Self> {
        let usage = body.get("usage")?;
        let model = body.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());

        let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64())?;
        let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64())?;

        let cached_tokens = usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                usage
                    .get("input_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
                    .and_then(|v| v.as_u64())
            })
            .unwrap_or(0) as u32;

        Some(Self {
            input_tokens: input_tokens as u32,
            output_tokens: output_tokens as u32,
            cache_read_tokens: cached_tokens,
            cache_creation_tokens: usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            model,
        })
    }

    /// 从 OpenAI 流式响应事件解析
    #[allow(dead_code)]
    pub fn from_openai_stream_events(events: &[Value]) -> Option<Self> {
        // OpenAI 流式响应在最后一个 chunk 中包含 usage
        for event in events.iter().rev() {
            if let Some(usage) = event.get("usage") {
                if !usage.is_null() {
                    return Self::from_openai_response(event);
                }
            }
        }
        None
    }

    /// 从 Gemini API 响应解析
    pub fn from_gemini_response(body: &Value) -> Option<Self> {
        let usage = body.get("usageMetadata")?;
        let model = body.get("modelVersion").and_then(|v| v.as_str()).map(|s| s.to_string());

        let prompt_tokens = usage.get("promptTokenCount")?.as_u64()? as u32;
        let total_tokens = usage.get("totalTokenCount")?.as_u64()? as u32;

        // 输出 tokens = 总 tokens - 输入 tokens
        let output_tokens = total_tokens.saturating_sub(prompt_tokens);

        Some(Self {
            input_tokens: prompt_tokens,
            output_tokens,
            cache_read_tokens: usage
                .get("cachedContentTokenCount")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            cache_creation_tokens: 0,
            model,
        })
    }

    /// 从 Gemini 流式响应 chunks 解析
    #[allow(dead_code)]
    pub fn from_gemini_stream_chunks(chunks: &[Value]) -> Option<Self> {
        let mut total_input = 0u32;
        let mut total_tokens = 0u32;
        let mut total_cache_read = 0u32;
        let mut model: Option<String> = None;

        for chunk in chunks {
            if let Some(usage) = chunk.get("usageMetadata") {
                total_input = usage
                    .get("promptTokenCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                total_tokens = usage
                    .get("totalTokenCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                total_cache_read = usage
                    .get("cachedContentTokenCount")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
            }

            if model.is_none() {
                if let Some(model_version) = chunk.get("modelVersion").and_then(|v| v.as_str()) {
                    model = Some(model_version.to_string());
                }
            }
        }

        let total_output = total_tokens.saturating_sub(total_input);

        if total_input > 0 || total_output > 0 {
            Some(Self {
                input_tokens: total_input,
                output_tokens: total_output,
                cache_read_tokens: total_cache_read,
                cache_creation_tokens: 0,
                model,
            })
        } else {
            None
        }
    }
}
