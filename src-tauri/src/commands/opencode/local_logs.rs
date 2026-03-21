//! 本地日志解析和导入模块
//!
//! 支持从 Claude Code、Codex CLI、Gemini CLI 和 Opencode 的本地日志文件中解析使用统计数据

use base64::Engine as _;
use crate::modules::{cursor_account, logger, opencode_db::Database};
use crate::opencode_error::AppError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::{Emitter, State};
use tiktoken_rs::{cl100k_base, get_bpe_from_model, CoreBPE};

// ============================================================================
// 数据结构
// ============================================================================

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Claude Code 日志文件数
    pub claude_files: u32,
    /// Claude Code 日志条目数（预估）
    pub claude_entries: u32,
    /// Claude Code 日志目录
    pub claude_path: Option<String>,
    /// Codex CLI 日志文件数
    pub codex_files: u32,
    /// Codex CLI 日志条目数（预估）
    pub codex_entries: u32,
    /// Codex CLI 日志目录
    pub codex_path: Option<String>,
    /// Gemini CLI 日志文件数
    pub gemini_files: u32,
    /// Gemini CLI 日志条目数（预估）
    pub gemini_entries: u32,
    /// Gemini CLI 日志目录
    pub gemini_path: Option<String>,
    /// Opencode 日志文件数
    pub opencode_files: u32,
    /// Opencode 日志条目数（预估）
    pub opencode_entries: u32,
    /// Opencode 日志目录
    pub opencode_path: Option<String>,
    /// Cursor 数据库文件数
    pub cursor_files: u32,
    /// Cursor 会话条目数（预估）
    pub cursor_entries: u32,
    /// Cursor 数据库路径
    pub cursor_path: Option<String>,
    /// Windsurf 数据库文件数
    pub windsurf_files: u32,
    /// Windsurf 会话条目数（预估）
    pub windsurf_entries: u32,
    /// Windsurf 数据库路径
    pub windsurf_path: Option<String>,
    /// Kiro 数据库文件数
    pub kiro_files: u32,
    /// Kiro 会话条目数（预估）
    pub kiro_entries: u32,
    /// Kiro 数据库路径
    pub kiro_path: Option<String>,
    /// Antigravity 数据库文件数
    pub antigravity_files: u32,
    /// Antigravity 会话条目数（预估）
    pub antigravity_entries: u32,
    /// Antigravity 数据库路径
    pub antigravity_path: Option<String>,
    /// Warp 数据库文件数
    pub warp_files: u32,
    /// Warp 会话条目数（预估）
    pub warp_entries: u32,
    /// Warp 数据库路径
    pub warp_path: Option<String>,
    /// Augment 数据库文件数
    pub augment_files: u32,
    /// Augment 会话条目数（预估）
    pub augment_entries: u32,
    /// Augment 数据库路径
    pub augment_path: Option<String>,
    /// GitHub Copilot 文件数
    pub github_copilot_files: u32,
    pub github_copilot_entries: u32,
    pub github_copilot_path: Option<String>,
    /// CodeBuddy 文件数
    pub codebuddy_files: u32,
    pub codebuddy_entries: u32,
    pub codebuddy_path: Option<String>,
    /// CodeBuddy CN 文件数
    pub codebuddy_cn_files: u32,
    pub codebuddy_cn_entries: u32,
    pub codebuddy_cn_path: Option<String>,
    /// Qoder 文件数
    pub qoder_files: u32,
    pub qoder_entries: u32,
    pub qoder_path: Option<String>,
    /// Trae 文件数
    pub trae_files: u32,
    pub trae_entries: u32,
    pub trae_path: Option<String>,
    /// WorkBuddy 文件数
    pub workbuddy_files: u32,
    pub workbuddy_entries: u32,
    pub workbuddy_path: Option<String>,
    /// OpenClaw 文件数
    pub openclaw_files: u32,
    pub openclaw_entries: u32,
    pub openclaw_path: Option<String>,
    /// 数据库中已有的本地导入记录数
    pub existing_records: u32,
}

/// 本地日志导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalLogImportResult {
    /// 新增记录数
    pub imported: u32,
    /// 跳过的重复记录数
    pub skipped: u32,
    /// 解析失败的条目数
    pub failed: u32,
    /// 总处理条目数
    pub total: u32,
}

/// 本地日志进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalLogProgress {
    pub phase: String,
    pub source: String,
    pub current: u32,
    pub total: u32,
    pub message: String,
}

fn emit_local_log_progress(
    window: &tauri::Window,
    phase: &str,
    source: &str,
    current: u32,
    total: u32,
    message: &str,
) {
    let _ = window.emit(
        "local-log-progress",
        LocalLogProgress {
            phase: phase.to_string(),
            source: source.to_string(),
            current,
            total,
            message: message.to_string(),
        },
    );
}

/// 本地日志条目
#[derive(Debug, Clone)]
pub struct LocalLogEntry {
    /// 来源: "claude" | "codex"
    pub source: String,
    /// Unix 时间戳（秒）
    pub timestamp: i64,
    /// 模型名称
    pub model: String,
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
    /// 缓存读取 token 数
    pub cache_read_tokens: u32,
    /// 缓存创建 token 数
    pub cache_creation_tokens: u32,
    /// 成本（如果日志中有）
    pub cost_usd: Option<f64>,
    /// 会话 ID（用于去重）
    pub session_id: String,
    /// 项目名称
    pub project_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CursorUsageCsvRow {
    #[serde(rename = "Date")]
    date: String,
    #[serde(default, rename = "Kind")]
    kind: Option<String>,
    #[serde(rename = "Model")]
    model: String,
    #[serde(default, rename = "Max Mode")]
    max_mode: Option<String>,
    #[serde(default, rename = "Input (w/ Cache Write)")]
    input_with_cache_write: Option<String>,
    #[serde(default, rename = "Input (w/o Cache Write)")]
    input_without_cache_write: Option<String>,
    #[serde(default, rename = "Cache Read")]
    cache_read: Option<String>,
    #[serde(default, rename = "Output Tokens")]
    output_tokens: Option<String>,
    #[serde(default, rename = "Total Tokens")]
    total_tokens: Option<String>,
    #[serde(default, rename = "Cost")]
    cost: Option<String>,
    #[serde(default, rename = "Cost to you")]
    cost_to_you: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CursorOfficialImportBatch {
    entries: Vec<LocalLogEntry>,
    range_start: i64,
    range_end: i64,
}

/// 会话统计信息
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// 对话轮数
    pub conversation_count: u32,
    /// 工具调用统计 (工具名 -> 调用次数)
    pub tool_calls: std::collections::HashMap<String, u32>,
    /// 修改的文件数
    pub files_changed: u32,
    /// 新增行数
    pub lines_added: u32,
    /// 删除行数
    pub lines_deleted: u32,
    /// 累计响应时间（毫秒）
    pub response_time_ms: u64,
    /// 累计思考时间（毫秒）
    pub thinking_time_ms: u64,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallInfo {
    pub tool_name: String,
    pub count: u32,
}

/// Cursor 对话统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CursorConversationStats {
    /// 总对话数
    pub total_conversations: u32,
    /// 总消息数
    pub total_messages: u32,
    /// 工具调用数
    pub tool_calls: u32,
    /// 文件变更数
    pub files_changed: u32,
    /// 代码块数量（代码变更）
    pub code_blocks: u32,
    /// Diff 数量
    pub diff_count: u32,
    /// 新增行数（从代码块估算）
    pub lines_added: u32,
    /// 删除行数（从 diff 估算）
    pub lines_deleted: u32,
    /// 工具调用详情
    pub tool_call_details: HashMap<String, u32>,
    /// MCP 服务器数量
    pub mcp_count: u32,
    /// 累计响应时间（毫秒）
    pub response_time_ms: u64,
    /// 累计思考时间（毫秒）- 从输出文本估算
    pub thinking_time_ms: u64,
    /// 对话累计持续时间（毫秒）- 从 createdAt 到 lastUpdated 的差值
    pub total_duration_ms: u64,
}

// ============================================================================
// Claude Code 日志解析
// ============================================================================

/// 获取 Claude Code 日志目录
fn get_claude_log_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let claude_dir = home.join(".claude").join("projects");
    if claude_dir.exists() {
        Some(claude_dir)
    } else {
        None
    }
}

const FAST_SCAN_MAX_BYTES: u64 = 256 * 1024;
const APPROX_BYTES_PER_LINE: u64 = 400;

fn estimate_entries_from_file(path: &PathBuf, is_jsonl: bool) -> u32 {
    let Ok(meta) = fs::metadata(path) else {
        return 0;
    };
    let size = meta.len();
    if is_jsonl {
        if size <= FAST_SCAN_MAX_BYTES {
            if let Ok(content) = fs::read_to_string(path) {
                let count = content.lines().count() as u32;
                return count.max(1);
            }
        }
        let approx = (size / APPROX_BYTES_PER_LINE).max(1) as u32;
        return approx;
    }
    1
}

/// 扫描 Claude Code 日志文件
fn scan_claude_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_claude_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 遍历 projects 目录下的所有子目录
    if let Ok(entries) = fs::read_dir(&log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 查找 .jsonl 文件
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let file_path = sub_entry.path();
                        if file_path.extension().map_or(false, |ext| ext == "jsonl") {
                            entry_count += estimate_entries_from_file(&file_path, true);
                            files.push(file_path);
                        }
                    }
                }
            }
        }
    }

    (files, entry_count)
}

/// 解析 Claude Code 日志文件
fn parse_claude_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件路径提取项目名称
    let project_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    for line in content.lines() {
        if let Some(entry) = parse_claude_log_line(line, &session_id, &project_name) {
            entries.push(entry);
        }
    }

    entries
}

/// 解析 Claude Code 日志行
fn parse_claude_log_line(
    line: &str,
    session_id: &str,
    project_name: &Option<String>,
) -> Option<LocalLogEntry> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // Claude Code 日志格式：
    // - type: "assistant" 的消息包含 usage 信息
    // - message.usage 包含 token 统计
    // - costUSD 可能在顶层或 message 中
    
    let msg_type = json.get("type").and_then(|v| v.as_str())?;
    
    // 只处理 assistant 类型的消息（包含使用量）
    if msg_type != "assistant" {
        return None;
    }

    // 尝试从多个位置获取 usage
    let usage = json
        .get("message")
        .and_then(|m| m.get("usage"))
        .or_else(|| json.get("usage"))?;

    let input_tokens = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let output_tokens = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_read_tokens = usage
        .get("cache_read_input_tokens")
        .or_else(|| usage.get("cacheReadInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let cache_creation_tokens = usage
        .get("cache_creation_input_tokens")
        .or_else(|| usage.get("cacheCreationInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    // 如果没有任何 token，跳过
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    // 获取模型名称
    let model = json
        .get("message")
        .and_then(|m| m.get("model"))
        .or_else(|| json.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 获取时间戳
    let timestamp = json
        .get("timestamp")
        .and_then(|v| {
            // 可能是 ISO 格式字符串或 Unix 时间戳
            if let Some(ts) = v.as_i64() {
                // 如果是毫秒，转换为秒
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                // 尝试解析 ISO 格式
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    // 获取成本
    let cost_usd = json
        .get("costUSD")
        .or_else(|| json.get("cost_usd"))
        .and_then(|v| v.as_f64());

    // 生成唯一的条目 ID
    let entry_session_id = format!("{}-{}", session_id, timestamp);

    Some(LocalLogEntry {
        source: "claude".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd,
        session_id: entry_session_id,
        project_name: project_name.clone(),
    })
}

// ============================================================================
// Codex CLI 日志解析
// ============================================================================

/// 获取 Codex CLI 日志目录
fn get_codex_log_dir() -> Option<PathBuf> {
    // 优先使用环境变量
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        let path = PathBuf::from(codex_home);
        if path.exists() {
            return Some(path);
        }
    }

    // 默认位置
    let home = dirs::home_dir()?;
    let codex_dir = home.join(".codex");
    if codex_dir.exists() {
        Some(codex_dir)
    } else {
        None
    }
}

/// 扫描 Codex CLI 日志文件
fn scan_codex_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_codex_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 递归扫描函数
    fn scan_dir_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>, entry_count: &mut u32) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录 (sessions/YYYY/MM/DD/)
                    scan_dir_recursive(&path, files, entry_count);
                } else if path.extension().map_or(false, |ext| ext == "jsonl") {
                    // 找到 .jsonl 文件
                    *entry_count += estimate_entries_from_file(&path, true);
                    files.push(path);
                }
            }
        }
    }

    // 扫描根目录和 sessions 子目录
    scan_dir_recursive(&log_dir, &mut files, &mut entry_count);
    
    // 也扫描 sessions 目录（如果和根目录不同）
    let sessions_dir = log_dir.join("sessions");
    if sessions_dir.exists() && sessions_dir != log_dir {
        scan_dir_recursive(&sessions_dir, &mut files, &mut entry_count);
    }

    (files, entry_count)
}

/// 解析 Codex CLI 日志文件
fn parse_codex_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Codex 使用累计 token，需要追踪上一次的值来计算 delta
    let mut last_input_tokens = 0u32;
    let mut last_output_tokens = 0u32;
    let mut last_cached_tokens = 0u32;
    let mut current_model = "gpt-5".to_string();

    for line in content.lines() {
        if let Some(result) =
            parse_codex_log_line(line, &session_id, last_input_tokens, last_output_tokens, last_cached_tokens)
        {
            last_input_tokens = result.new_input;
            last_output_tokens = result.new_output;
            last_cached_tokens = result.new_cached;
            if let Some(m) = result.model {
                current_model = m;
            }

            if let Some(mut entry) = result.entry {
                if entry.model == "unknown" {
                    entry.model = current_model.clone();
                }
                entries.push(entry);
            }
        }
    }

    entries
}

/// Codex 解析结果
struct CodexParseResult {
    entry: Option<LocalLogEntry>,
    new_input: u32,
    new_output: u32,
    new_cached: u32,
    model: Option<String>,
}

/// 解析 Codex CLI 日志行
/// 返回 (条目, 累计输入, 累计输出, 累计缓存, 模型)
fn parse_codex_log_line(
    line: &str,
    session_id: &str,
    last_input: u32,
    last_output: u32,
    last_cached: u32,
) -> Option<CodexParseResult> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;

    // 处理模型上下文
    if json.get("type").and_then(|v| v.as_str()) == Some("turn_context") {
        let model = json
            .get("payload")
            .and_then(|p| p.get("model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        return Some(CodexParseResult {
            entry: None,
            new_input: last_input,
            new_output: last_output,
            new_cached: last_cached,
            model,
        });
    }

    // Codex 日志格式：
    // 1) type=event_msg, payload.type=token_count
    // 2) event_msg.payload.type=token_count
    let payload = if json.get("type").and_then(|v| v.as_str()) == Some("event_msg") {
        json.get("payload")?
    } else if let Some(event_msg) = json.get("event_msg") {
        event_msg.get("payload")?
    } else {
        return None;
    };

    let payload_type = payload.get("type").and_then(|v| v.as_str())?;
    if payload_type != "token_count" {
        return None;
    }

    let info = payload.get("info");
    let total_usage = info
        .and_then(|i| i.get("total_token_usage"))
        .or_else(|| payload.get("total_token_usage"))
        .or_else(|| payload.get("token_usage"));

    let last_usage = info.and_then(|i| i.get("last_token_usage"));

    let mut total_input = total_usage
        .and_then(|u| u.get("input_tokens"))
        .or_else(|| payload.get("input_tokens"))
        .or_else(|| payload.get("inputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let mut total_output = total_usage
        .and_then(|u| u.get("output_tokens"))
        .or_else(|| payload.get("output_tokens"))
        .or_else(|| payload.get("outputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let mut total_cached = total_usage
        .and_then(|u| u.get("cached_input_tokens"))
        .or_else(|| payload.get("cached_input_tokens"))
        .or_else(|| payload.get("cachedInputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let (input_delta, output_delta, cached_delta) = if total_input > 0 || total_output > 0 || total_cached > 0 {
        (
            total_input.saturating_sub(last_input),
            total_output.saturating_sub(last_output),
            total_cached.saturating_sub(last_cached),
        )
    } else if let Some(last) = last_usage {
        let input = last
            .get("input_tokens")
            .or_else(|| last.get("inputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = last
            .get("output_tokens")
            .or_else(|| last.get("outputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cached = last
            .get("cached_input_tokens")
            .or_else(|| last.get("cachedInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        total_input = last_input.saturating_add(input);
        total_output = last_output.saturating_add(output);
        total_cached = last_cached.saturating_add(cached);

        (input, output, cached)
    } else {
        return None;
    };

    // 如果没有变化，跳过
    if input_delta == 0 && output_delta == 0 && cached_delta == 0 {
        return None;
    }

    // 获取时间戳
    let timestamp_value = json
        .get("timestamp")
        .or_else(|| payload.get("timestamp"))
        .or_else(|| json.get("event_msg").and_then(|m| m.get("timestamp")));

    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let entry_session_id = format!("{}-{}", session_id, timestamp);

    let entry = LocalLogEntry {
        source: "codex".to_string(),
        timestamp,
        model: "unknown".to_string(), // 后续更新
        input_tokens: input_delta,
        output_tokens: output_delta,
        cache_read_tokens: cached_delta,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    };

    Some(CodexParseResult {
        entry: Some(entry),
        new_input: total_input,
        new_output: total_output,
        new_cached: total_cached,
        model: None,
    })
}

// ============================================================================
// Gemini CLI 日志解析
// ============================================================================

/// 获取 Gemini CLI 日志目录
fn get_gemini_log_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let gemini_dir = home.join(".gemini").join("tmp");
    if gemini_dir.exists() {
        Some(gemini_dir)
    } else {
        None
    }
}

/// 扫描 Gemini CLI 日志文件
fn scan_gemini_logs() -> (Vec<PathBuf>, u32) {
    let Some(log_dir) = get_gemini_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 递归扫描 tmp/<project_hash>/chats/ 目录
    fn scan_gemini_recursive(dir: &PathBuf, files: &mut Vec<PathBuf>, entry_count: &mut u32) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 递归扫描子目录
                    scan_gemini_recursive(&path, files, entry_count);
                } else {
                    // Gemini 日志可能是 .json 或 .jsonl
                    let ext = path.extension().and_then(|e| e.to_str());
                    if ext == Some("json") || ext == Some("jsonl") {
                        // 检查是否在 chats 目录下或者是 session 文件
                        let is_chat_file = path.parent()
                            .and_then(|p| p.file_name())
                            .map_or(false, |n| n == "chats");
                        let is_session_file = path.file_name()
                            .and_then(|n| n.to_str())
                            .map_or(false, |n| n.starts_with("session-"));
                        
                        if is_chat_file || is_session_file {
                            let is_jsonl = ext == Some("jsonl");
                            *entry_count += estimate_entries_from_file(&path, is_jsonl);
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    scan_gemini_recursive(&log_dir, &mut files, &mut entry_count);

    (files, entry_count)
}

/// 解析 Gemini CLI 日志文件
fn parse_gemini_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    // 从文件名提取会话 ID
    let session_id = path
        .file_stem()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let ext = path.extension().and_then(|e| e.to_str());

    if ext == Some("jsonl") {
        // JSONL 格式：每行一个 JSON 对象
        for line in content.lines() {
            if let Some(entry) = parse_gemini_log_line(line, &session_id) {
                entries.push(entry);
            }
        }
    } else {
        // JSON 格式：整个文件是一个 JSON 对象
        entries.extend(parse_gemini_json_file(&content, &session_id));
    }

    entries
}

/// 解析 Gemini CLI 日志行 (JSONL 格式)
fn parse_gemini_log_line(line: &str, session_id: &str) -> Option<LocalLogEntry> {
    let json: serde_json::Value = serde_json::from_str(line).ok()?;
    parse_gemini_usage_from_json(&json, session_id)
}

/// 解析 Gemini CLI JSON 文件
fn parse_gemini_json_file(content: &str, session_id: &str) -> Vec<LocalLogEntry> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(content) else {
        return vec![];
    };

    parse_gemini_entries_from_value(&json, session_id)
}

/// 从 Gemini JSON 中提取使用量（支持 messages 数组）
fn parse_gemini_entries_from_value(json: &serde_json::Value, session_id: &str) -> Vec<LocalLogEntry> {
    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        let base_session_id = json
            .get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or(session_id);
        return messages
            .iter()
            .filter_map(|m| parse_gemini_message(m, base_session_id))
            .collect();
    }

    if let Some(array) = json.as_array() {
        return array
            .iter()
            .filter_map(|m| parse_gemini_message(m, session_id))
            .collect();
    }

    if let Some(entry) = parse_gemini_usage_from_json(json, session_id) {
        return vec![entry];
    }

    vec![]
}

/// 解析 Gemini messages 中的单条消息
fn parse_gemini_message(message: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    let tokens = message.get("tokens")?;

    let input_tokens = tokens
        .get("input")
        .or_else(|| tokens.get("prompt"))
        .or_else(|| tokens.get("prompt_tokens"))
        .or_else(|| tokens.get("inputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let output_tokens = tokens
        .get("output")
        .or_else(|| tokens.get("completion"))
        .or_else(|| tokens.get("completion_tokens"))
        .or_else(|| tokens.get("outputTokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let thoughts_tokens = tokens
        .get("thoughts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let tool_tokens = tokens
        .get("tool")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let cache_read_tokens = tokens
        .get("cached")
        .or_else(|| tokens.get("cache").and_then(|c| c.get("read")))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let cache_creation_tokens = tokens
        .get("cache")
        .and_then(|c| c.get("write"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let output_total = output_tokens
        .saturating_add(thoughts_tokens)
        .saturating_add(tool_tokens);

    if input_tokens == 0 && output_total == 0 && cache_read_tokens == 0 && cache_creation_tokens == 0 {
        return None;
    }

    let model = message
        .get("model")
        .or_else(|| message.get("modelId"))
        .or_else(|| message.get("modelID"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let timestamp_value = message.get("timestamp").or_else(|| message.get("time"));
    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let msg_id = message
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg");
    let entry_session_id = format!("{}-{}-{}", session_id, timestamp, msg_id);

    Some(LocalLogEntry {
        source: "gemini".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens: output_total,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    })
}

/// 从 Gemini JSON 中提取使用量
fn parse_gemini_usage_from_json(json: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    // Gemini CLI 日志格式：
    // - stats 对象包含 token 使用统计
    // - usageMetadata 也可能包含使用信息
    
    // 尝试从 stats 获取
    let stats = json.get("stats")
        .or_else(|| json.get("usageMetadata"));
    
    let (input_tokens, output_tokens, cached_tokens) = if let Some(stats) = stats {
        let input = stats.get("promptTokenCount")
            .or_else(|| stats.get("prompt_tokens"))
            .or_else(|| stats.get("inputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let output = stats.get("candidatesTokenCount")
            .or_else(|| stats.get("completion_tokens"))
            .or_else(|| stats.get("outputTokens"))
            .or_else(|| stats.get("responseTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        let cached = stats.get("cachedContentTokenCount")
            .or_else(|| stats.get("cached_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        (input, output, cached)
    } else {
        // 尝试从顶层获取
        let input = json.get("inputTokens")
            .or_else(|| json.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = json.get("outputTokens")
            .or_else(|| json.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cached = json.get("cachedTokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        
        (input, output, cached)
    };

    // 如果没有任何 token，跳过
    if input_tokens == 0 && output_tokens == 0 {
        return None;
    }

    // 获取模型名称
    let model = json.get("model")
        .or_else(|| json.get("modelVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-2.5-flash")
        .to_string();

    // 获取时间戳
    let timestamp = json.get("timestamp")
        .or_else(|| json.get("createTime"))
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let entry_session_id = format!("{}-{}", session_id, timestamp);

    Some(LocalLogEntry {
        source: "gemini".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens: cached_tokens,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id: entry_session_id,
        project_name: None,
    })
}

// ============================================================================
// Opencode 日志解析
// ============================================================================

/// 获取 Opencode 日志目录
fn get_opencode_log_dir() -> Option<PathBuf> {
    // Opencode 存储位置：~/.local/share/opencode/storage/
    let home = dirs::home_dir()?;
    
    #[cfg(windows)]
    let opencode_dir = home.join(".local").join("share").join("opencode").join("storage");
    #[cfg(not(windows))]
    let opencode_dir = home.join(".local").join("share").join("opencode").join("storage");
    
    if opencode_dir.exists() {
        Some(opencode_dir)
    } else {
        None
    }
}

/// 扫描 Opencode 日志文件
fn scan_opencode_logs() -> (Vec<PathBuf>, u32) {
    let Some(storage_dir) = get_opencode_log_dir() else {
        return (vec![], 0);
    };

    let mut files = Vec::new();
    let mut entry_count = 0u32;

    // 扫描 message/{sessionID}/{messageID}.json 文件
    // opencode 的消息文件名是 messageID.json，不是 msg_messageID.json
    let message_dir = storage_dir.join("message");
    if message_dir.exists() {
        if let Ok(sessions) = fs::read_dir(&message_dir) {
            for session in sessions.flatten() {
                let session_path = session.path();
                if session_path.is_dir() {
                    if let Ok(messages) = fs::read_dir(&session_path) {
                        for msg in messages.flatten() {
                            let msg_path = msg.path();
                            if msg_path.extension().and_then(|e| e.to_str()) == Some("json") {
                                // 所有 .json 文件都是消息文件
                                files.push(msg_path);
                                entry_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    (files, entry_count)
}

/// 解析 Opencode 日志文件
fn parse_opencode_log_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return entries;
    };

    // 从路径提取会话 ID
    let session_id = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // 尝试解析 Opencode 消息格式
    if let Some(entry) = parse_opencode_message(&json, &session_id) {
        entries.push(entry);
    }

    entries
}

/// 解析 Opencode 消息
fn parse_opencode_message(json: &serde_json::Value, session_id: &str) -> Option<LocalLogEntry> {
    // Opencode 消息格式：优先读取 tokens 字段
    let (input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens) = if let Some(tokens) = json.get("tokens") {
        let input = tokens
            .get("input")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let output = tokens
            .get("output")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let reasoning = tokens
            .get("reasoning")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cache_read = tokens
            .get("cache")
            .and_then(|c| c.get("read"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let cache_write = tokens
            .get("cache")
            .and_then(|c| c.get("write"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        (input, output.saturating_add(reasoning), cache_read, cache_write)
    } else if let Some(usage) = json.get("usage") {
        let input = usage
            .get("input_tokens")
            .or_else(|| usage.get("inputTokens"))
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let output = usage
            .get("output_tokens")
            .or_else(|| usage.get("outputTokens"))
            .or_else(|| usage.get("completion_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let cache_read = usage
            .get("cache_read_input_tokens")
            .or_else(|| usage.get("cacheReadInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let cache_creation = usage
            .get("cache_creation_input_tokens")
            .or_else(|| usage.get("cacheCreationInputTokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        (input, output, cache_read, cache_creation)
    } else {
        return None;
    };

    // 如果没有 token 数据，跳过
    if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 && cache_creation_tokens == 0 {
        return None;
    }

    // 获取模型
    let model = json
        .get("modelID")
        .or_else(|| json.get("modelId"))
        .or_else(|| json.get("model").and_then(|m| m.get("modelID")))
        .or_else(|| json.get("model").and_then(|m| m.get("modelId")))
        .or_else(|| json.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 获取时间戳
    let timestamp_value = json
        .get("time")
        .and_then(|t| t.get("completed").or_else(|| t.get("created")))
        .or_else(|| json.get("timestamp"))
        .or_else(|| json.get("created_at"));

    let timestamp = timestamp_value
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    // 获取成本
    let cost_usd = json
        .get("cost")
        .or_else(|| json.get("costUSD"))
        .and_then(|v| v.as_f64());

    let msg_id = json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("msg");
    let entry_session_id = format!("{}-{}-{}", session_id, timestamp, msg_id);

    Some(LocalLogEntry {
        source: "opencode".to_string(),
        timestamp,
        model,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
        cost_usd,
        session_id: entry_session_id,
        project_name: None,
    })
}

// ============================================================================
// Cursor 日志解析
// ============================================================================

/// 获取 Cursor 数据库路径列表（globalStorage + workspaceStorage）
fn get_cursor_db_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    #[cfg(windows)]
    {
        let mut base_dirs: Vec<PathBuf> = Vec::new();
        if let Ok(appdata) = std::env::var("APPDATA") {
            base_dirs.push(PathBuf::from(appdata));
        }
        if let Ok(localapp) = std::env::var("LOCALAPPDATA") {
            base_dirs.push(PathBuf::from(localapp));
        }
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join("AppData").join("Roaming"));
            base_dirs.push(home.join("AppData").join("Local"));
        }

        for base in base_dirs {
            let user_dir = base.join("Cursor").join("User");
            if !user_dir.exists() {
                continue;
            }

            // globalStorage/state.vscdb
            let global_db = user_dir.join("globalStorage").join("state.vscdb");
            if global_db.exists() {
                let key = global_db.to_string_lossy().to_string();
                if seen.insert(key) {
                    paths.push(global_db);
                }
            }

            // workspaceStorage/*/state.vscdb
            let workspace_dir = user_dir.join("workspaceStorage");
            if workspace_dir.exists() {
                if let Ok(workspaces) = fs::read_dir(&workspace_dir) {
                    for entry in workspaces.flatten() {
                        let db_path = entry.path().join("state.vscdb");
                        if db_path.exists() {
                            let key = db_path.to_string_lossy().to_string();
                            if seen.insert(key) {
                                paths.push(db_path);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let base_dirs = vec![
                home.join("Library").join("Application Support"),
                home.join(".config"),
            ];
            for base in base_dirs {
                let user_dir = base.join("Cursor").join("User");
                if !user_dir.exists() {
                    continue;
                }

                let global_db = user_dir.join("globalStorage").join("state.vscdb");
                if global_db.exists() {
                    let key = global_db.to_string_lossy().to_string();
                    if seen.insert(key) {
                        paths.push(global_db);
                    }
                }

                let workspace_dir = user_dir.join("workspaceStorage");
                if workspace_dir.exists() {
                    if let Ok(workspaces) = fs::read_dir(&workspace_dir) {
                        for entry in workspaces.flatten() {
                            let db_path = entry.path().join("state.vscdb");
                            if db_path.exists() {
                                let key = db_path.to_string_lossy().to_string();
                                if seen.insert(key) {
                                    paths.push(db_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        if let Some(home) = dirs::home_dir() {
            let base_dirs = vec![
                home.join(".config"),
                home.join(".local").join("share"),
            ];
            for base in base_dirs {
                let user_dir = base.join("Cursor").join("User");
                if !user_dir.exists() {
                    continue;
                }

                let global_db = user_dir.join("globalStorage").join("state.vscdb");
                if global_db.exists() {
                    let key = global_db.to_string_lossy().to_string();
                    if seen.insert(key) {
                        paths.push(global_db);
                    }
                }

                let workspace_dir = user_dir.join("workspaceStorage");
                if workspace_dir.exists() {
                    if let Ok(workspaces) = fs::read_dir(&workspace_dir) {
                        for entry in workspaces.flatten() {
                            let db_path = entry.path().join("state.vscdb");
                            if db_path.exists() {
                                let key = db_path.to_string_lossy().to_string();
                                if seen.insert(key) {
                                    paths.push(db_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    paths
}

/// 获取 Cursor 主数据库路径（优先 globalStorage）
fn get_cursor_db_path() -> Option<PathBuf> {
    let paths = get_cursor_db_paths();
    if let Some(global) = paths.iter().find(|p| p.to_string_lossy().contains("globalStorage")) {
        return Some(global.clone());
    }
    paths.into_iter().next()
}

/// 扫描 Cursor 数据库
fn scan_cursor_logs() -> (Vec<PathBuf>, u32) {
    let db_paths = get_cursor_db_paths();
    if db_paths.is_empty() {
        return (vec![], 0);
    }

    let mut entry_count = 0u32;
    // 如果数据库文件过多，扫描计数会很慢，优先只统计 globalStorage
    if db_paths.len() > 8 {
        if let Some(global) = db_paths
            .iter()
            .find(|p| p.to_string_lossy().contains("globalStorage"))
        {
            entry_count = count_cursor_sessions(global).unwrap_or(0);
        }
    } else {
        for path in &db_paths {
            entry_count = entry_count.saturating_add(count_cursor_sessions(path).unwrap_or(0));
        }
    }

    (db_paths, entry_count)
}

/// 统计 Cursor 数据库中的会话数量
fn count_cursor_sessions(db_path: &PathBuf) -> Option<u32> {
    use rusqlite::{Connection, OpenFlags};
    
    // 以只读模式打开 Cursor 数据库
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    
    let mut count = 0u32;

    // ItemTable: Chat + Composer + aiService
    let item_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM ItemTable WHERE key IN (
            'workbench.panel.aichat.view.aichat.chatdata',
            'composer.composerData',
            'aiService.prompts',
            'aiService.generations'
        )",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0);
    count += item_count;

    let pane_count: u32 = conn.query_row(
        "SELECT COUNT(*) FROM ItemTable WHERE key LIKE 'workbench.panel.composerChatViewPane.%'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(0);
    count += pane_count;

    // cursorDiskKV: Composer/Agent（仅 globalStorage）
    let has_cursor_kv = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='cursorDiskKV'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    if has_cursor_kv {
        let cursor_kv_count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%' OR key LIKE 'bubbleId:%'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
        count += cursor_kv_count;
    }

    Some(count)
}

/// 获取 workspaceStorage 的 workspace ID
fn cursor_workspace_id(path: &PathBuf) -> Option<String> {
    let parent = path.parent()?;
    let parent_name = parent.file_name()?.to_string_lossy().to_string();
    let grand = parent.parent()?;
    let grand_name = grand.file_name()?.to_string_lossy();
    if grand_name == "workspaceStorage" {
        Some(parent_name)
    } else {
        None
    }
}

/// 从 ItemTable 读取原始数据
fn query_itemtable_value(conn: &rusqlite::Connection, key: &str) -> Option<Vec<u8>> {
    conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        [key],
        |row| {
            let value = row
                .get::<_, Vec<u8>>(0)
                .ok()
                .or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes()));
            Ok(value)
        },
    )
    .ok()
    .flatten()
}

fn query_itemtable_entries_like(conn: &rusqlite::Connection, pattern: &str) -> Vec<(String, Vec<u8>)> {
    let mut entries = Vec::new();
    let Ok(mut stmt) = conn.prepare("SELECT key, value FROM ItemTable WHERE key LIKE ?1") else {
        return entries;
    };

    let Ok(rows) = stmt.query_map([pattern], |row| {
        let key: String = row.get(0)?;
        let value = row
            .get::<_, Vec<u8>>(1)
            .ok()
            .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
        Ok((key, value))
    }) else {
        return entries;
    };

    for row_result in rows.flatten() {
        let (key, value_opt) = row_result;
        if let Some(value_bytes) = value_opt {
            entries.push((key, value_bytes));
        }
    }

    entries
}

/// 解析 JSON 数据（兼容 BLOB/TEXT）
fn parse_json_bytes(bytes: &[u8]) -> Option<serde_json::Value> {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) {
        return Some(json);
    }
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        let trimmed = text.trim_matches('\u{0}').trim();
        if trimmed.is_empty() {
            return None;
        }
        return serde_json::from_str::<serde_json::Value>(trimmed).ok();
    }
    None
}

/// 解析 Cursor 数据库文件
fn parse_cursor_db(path: &PathBuf) -> Vec<LocalLogEntry> {
    use rusqlite::{Connection, OpenFlags};
    
    let mut entries = Vec::new();
    
    // 以只读模式打开数据库
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return entries;
    };
    
    let workspace_id = cursor_workspace_id(path);

    // 1. Chat 模式 (workbench.panel.aichat.view.aichat.chatdata)
    if let Some(value_bytes) = query_itemtable_value(&conn, "workbench.panel.aichat.view.aichat.chatdata") {
        if let Some(json) = parse_json_bytes(&value_bytes) {
            entries.extend(parse_cursor_chat_data(&json, workspace_id.as_deref()));
        }
    }

    // 2. Workspace Composer (composer.composerData)
    if let Some(value_bytes) = query_itemtable_value(&conn, "composer.composerData") {
        if let Some(json) = parse_json_bytes(&value_bytes) {
            entries.extend(parse_cursor_workspace_composer(&json, workspace_id.as_deref()));
        }
    }

    // 3. aiService 旧格式
    let prompts_json = query_itemtable_value(&conn, "aiService.prompts")
        .and_then(|b| parse_json_bytes(&b));
    let generations_json = query_itemtable_value(&conn, "aiService.generations")
        .and_then(|b| parse_json_bytes(&b));
    if prompts_json.is_some() || generations_json.is_some() {
        entries.extend(parse_cursor_aiservice(prompts_json, generations_json, workspace_id.as_deref()));
    }

    // 4. Global Composer (cursorDiskKV)
    let has_cursor_kv = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='cursorDiskKV'",
            [],
            |row| row.get::<_, String>(0),
        )
        .is_ok();

    let bubble_token_map = if has_cursor_kv {
        Some(load_cursor_bubble_token_map(&conn))
    } else {
        None
    };

    if has_cursor_kv {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'"
        ) {
            if let Ok(rows) = stmt.query_map([], |row| {
                let key: String = row.get(0)?;
                let value = row
                    .get::<_, Vec<u8>>(1)
                    .ok()
                    .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
                Ok((key, value))
            }) {
                for row_result in rows.flatten() {
                    let (key, value_opt) = row_result;
                    let Some(value_bytes) = value_opt else { continue };
                    let Some(json) = parse_json_bytes(&value_bytes) else { continue };
                    entries.extend(parse_cursor_composer_data(
                        &conn,
                        &json,
                        &key,
                        workspace_id.as_deref(),
                        bubble_token_map.as_ref(),
                    ));
                }
            }
        }
    }

    for (key, value_bytes) in query_itemtable_entries_like(&conn, "workbench.panel.composerChatViewPane.%") {
        let Some(json) = parse_json_bytes(&value_bytes) else { continue };
        entries.extend(parse_cursor_itemtable_payload(
            &conn,
            &json,
            &key,
            workspace_id.as_deref(),
            bubble_token_map.as_ref(),
        ));
    }

    for entry in entries.iter_mut() {
        // 逗号分隔的模型名只取第一个
        if let Some(first) = entry.model.split(',').next() {
            let trimmed = first.trim();
            if trimmed != entry.model {
                entry.model = trimmed.to_string();
            }
        }
        // Cursor 本地不存储缓存 token，按 input * 62.6 估算 cache_read
        if entry.cache_read_tokens == 0 && entry.input_tokens > 0 {
            entry.cache_read_tokens = (entry.input_tokens as f64 * 62.6) as u32;
        }
    }

    entries
}

fn parse_cursor_itemtable_payload(
    conn: &rusqlite::Connection,
    json: &serde_json::Value,
    key: &str,
    workspace_id: Option<&str>,
    bubble_token_map: Option<&BubbleTokenMap>,
) -> Vec<LocalLogEntry> {
    if json.get("tabs").and_then(|v| v.as_array()).is_some() {
        return parse_cursor_chat_data(json, workspace_id);
    }
    if json.get("allComposers").and_then(|v| v.as_array()).is_some() {
        return parse_cursor_workspace_composer(json, workspace_id);
    }
    if json.get("conversation").and_then(|v| v.as_array()).is_some()
        || json.get("composerId").is_some()
    {
        return parse_cursor_composer_data(conn, json, key, workspace_id, bubble_token_map);
    }
    if json.get("bubbles").and_then(|v| v.as_array()).is_some() {
        return parse_cursor_chat_pane_item(json, key, workspace_id);
    }
    Vec::new()
}

fn parse_cursor_chat_pane_item(
    json: &serde_json::Value,
    key: &str,
    workspace_id: Option<&str>,
) -> Vec<LocalLogEntry> {
    let Some(bubbles) = json.get("bubbles").and_then(|b| b.as_array()) else {
        return Vec::new();
    };
    if bubbles.is_empty() {
        return Vec::new();
    }

    let model_hint = json.get("modelId")
        .or_else(|| json.get("model"))
        .or_else(|| json.get("modelName"))
        .and_then(|v| v.as_str());
    let (input_tokens, output_tokens) = estimate_tokens_from_messages(bubbles, model_hint);
    if input_tokens == 0 && output_tokens == 0 {
        return Vec::new();
    }

    let fallback_tab_id = key.rsplit('.').next().unwrap_or("unknown");
    let tab_id = json.get("tabId")
        .or_else(|| json.get("composerId"))
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_tab_id);
    let title = json.get("chatTitle")
        .or_else(|| json.get("name"))
        .or_else(|| json.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    let timestamp = extract_cursor_timestamp(json);
    let session_id = if let Some(id) = workspace_id {
        format!("cursor-chat-{}-{}", id, tab_id)
    } else {
        format!("cursor-chat-{}", tab_id)
    };

    vec![LocalLogEntry {
        source: "cursor".to_string(),
        timestamp,
        model: model_hint.unwrap_or("cursor-chat").to_string(),
        input_tokens,
        output_tokens,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id,
        project_name: Some(title.to_string()),
    }]
}

/// 解析 Cursor Chat 模式数据 (workbench.panel.aichat.view.aichat.chatdata)
fn parse_cursor_chat_data(json: &serde_json::Value, workspace_id: Option<&str>) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();
    
    // chatdata 是一个包含 tabs 的对象
    if let Some(tabs) = json.get("tabs").and_then(|t| t.as_array()) {
        for tab in tabs {
            // 检查是否有 bubbles
            if let Some(bubbles) = tab.get("bubbles").and_then(|b| b.as_array()) {
                if bubbles.is_empty() {
                    continue;
                }
                
                let model_hint = tab.get("modelId")
                    .or_else(|| tab.get("model"))
                    .or_else(|| tab.get("modelName"))
                    .and_then(|v| v.as_str());

                // 估算 token
                let (input_tokens, output_tokens) = estimate_tokens_from_messages(bubbles, model_hint);
                
                if input_tokens == 0 && output_tokens == 0 {
                    continue;
                }
                
                // 获取 tab ID
                let tab_id = tab.get("tabId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // 获取标题
                let title = tab.get("chatTitle")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");

                let timestamp = extract_cursor_timestamp(tab);
                let session_id = if let Some(id) = workspace_id {
                    format!("cursor-chat-{}-{}", id, tab_id)
                } else {
                    format!("cursor-chat-{}", tab_id)
                };
                
                let model_name = model_hint.unwrap_or("cursor-chat");
                entries.push(LocalLogEntry {
                    source: "cursor".to_string(),
                    timestamp,
                    model: model_name.to_string(),
                    input_tokens,
                    output_tokens,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: None,
                    session_id,
                    project_name: Some(title.to_string()),
                });
            }
        }
    }
    
    entries
}

/// 解析 Cursor Workspace Composer 数据 (composer.composerData)
fn parse_cursor_workspace_composer(json: &serde_json::Value, workspace_id: Option<&str>) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();
    
    // 检查 allComposers 数组
    if let Some(all_composers) = json.get("allComposers").and_then(|a| a.as_array()) {
        for composer in all_composers {
            // 检查 conversation 数组
            if let Some(conversation) = composer.get("conversation").and_then(|c| c.as_array()) {
                if conversation.is_empty() {
                    continue;
                }
                
                // 获取模型
                let model = composer.get("modelConfig")
                    .and_then(|m| m.get("modelName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("cursor-composer");

                // 估算 token
                let (input_tokens, output_tokens) = estimate_tokens_from_cursor_conversation(conversation, Some(model));
                
                if input_tokens == 0 && output_tokens == 0 {
                    continue;
                }
                
                // 获取 composer ID
                let composer_id = composer.get("composerId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // 获取名称
                let name = composer.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Untitled");

                let timestamp = extract_cursor_timestamp(composer);
                let session_id = if let Some(id) = workspace_id {
                    format!("cursor-workspace-{}-{}", id, composer_id)
                } else {
                    format!("cursor-workspace-{}", composer_id)
                };
                
                entries.push(LocalLogEntry {
                    source: "cursor".to_string(),
                    timestamp,
                    model: model.to_string(),
                    input_tokens,
                    output_tokens,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: None,
                    session_id,
                    project_name: Some(name.to_string()),
                });
            }
        }
    }
    
    entries
}

/// 解析 Cursor Global Composer 数据 (composerData:{uuid})
type BubbleTokenMap = HashMap<String, (u32, u32)>;

static TOKENIZER_CACHE: OnceLock<Mutex<HashMap<String, Arc<CoreBPE>>>> = OnceLock::new();
#[allow(dead_code)]
const CURSOR_USAGE_EVENTS_CSV_URL: &str =
    "https://cursor.com/api/dashboard/export-usage-events-csv?strategy=tokens";

fn tokenizer_for_model(model: &str) -> Arc<CoreBPE> {
    let cleaned = clean_model_id(model).to_lowercase();
    let cache = TOKENIZER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    if let Ok(guard) = cache.lock() {
        if let Some(tok) = guard.get(&cleaned) {
            return tok.clone();
        }
    }

    let bpe = get_bpe_from_model(&cleaned).or_else(|_| cl100k_base()).unwrap_or_else(|_| {
        // 极端情况下 fallback 到 cl100k_base
        cl100k_base().expect("cl100k_base tokenizer init failed")
    });
    let arc = Arc::new(bpe);

    if let Ok(mut guard) = cache.lock() {
        guard.insert(cleaned, arc.clone());
    }

    arc
}

fn count_tokens_for_text(text: &str, model: Option<&str>) -> usize {
    if text.is_empty() {
        return 0;
    }
    let model_name = model.unwrap_or("cl100k_base");
    let tokenizer = tokenizer_for_model(model_name);
    tokenizer.encode_with_special_tokens(text).len()
}

#[allow(dead_code)]
fn extract_workos_user_id_from_cursor_access_token(jwt: &str) -> Option<String> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        return None;
    }

    let payload_b64 = parts[1].replace('-', "+").replace('_', "/");
    let padded = match payload_b64.len() % 4 {
        2 => format!("{}==", payload_b64),
        3 => format!("{}=", payload_b64),
        _ => payload_b64,
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(padded)
        .ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let sub = value.get("sub")?.as_str()?;
    let user_id = sub.rsplit('|').next().unwrap_or(sub);
    if user_id.starts_with("user_") {
        Some(user_id.to_string())
    } else {
        None
    }
}

#[allow(dead_code)]
fn build_cursor_session_cookie_from_access_token(access_token: &str) -> Option<String> {
    let user_id = extract_workos_user_id_from_cursor_access_token(access_token)?;
    Some(format!(
        "WorkosCursorSessionToken={}%3A%3A{}",
        user_id, access_token
    ))
}

#[allow(dead_code)]
fn parse_cursor_csv_u32(value: Option<&str>) -> u32 {
    value
        .map(|raw| raw.trim())
        .filter(|raw| !raw.is_empty())
        .and_then(|raw| raw.replace(',', "").parse::<u64>().ok())
        .map(|parsed| parsed.min(u32::MAX as u64) as u32)
        .unwrap_or(0)
}

#[allow(dead_code)]
fn parse_cursor_csv_cost(value: Option<&str>) -> Option<f64> {
    let raw = value?.trim();
    if raw.is_empty() {
        return None;
    }
    if raw.eq_ignore_ascii_case("included") {
        return Some(0.0);
    }

    raw.trim_start_matches('$')
        .replace(',', "")
        .parse::<f64>()
        .ok()
}

#[allow(dead_code)]
fn parse_cursor_csv_timestamp(value: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.timestamp())
}

#[allow(dead_code)]
async fn fetch_cursor_usage_events_csv(access_token: &str) -> Result<String, String> {
    let cookie = build_cursor_session_cookie_from_access_token(access_token)
        .ok_or_else(|| "无法从 Cursor access token 解析 WorkOS 用户 ID".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 Cursor usage HTTP 客户端失败: {}", e))?;

    let response = client
        .get(CURSOR_USAGE_EVENTS_CSV_URL)
        .header("Accept", "text/csv,*/*")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
        )
        .header("Cookie", cookie)
        .send()
        .await
        .map_err(|e| format!("请求 Cursor usage CSV 失败: {}", e))?;

    let status = response.status().as_u16();
    if status == 401 || status == 403 {
        return Err("Cursor 会话已过期或未认证，请重新导入账号".to_string());
    }
    if status != 200 {
        return Err(format!("Cursor usage CSV 返回异常状态码: {}", status));
    }

    response
        .text()
        .await
        .map_err(|e| format!("读取 Cursor usage CSV 失败: {}", e))
}

#[allow(dead_code)]
fn parse_cursor_official_usage_csv(
    account_id: &str,
    account_label: Option<&str>,
    csv_text: &str,
) -> Result<Option<CursorOfficialImportBatch>, String> {
    let normalized_csv = csv_text.trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(normalized_csv.as_bytes());

    let mut entries = Vec::new();
    let mut range_start = i64::MAX;
    let mut range_end = i64::MIN;

    for row in reader.deserialize::<CursorUsageCsvRow>() {
        let row = row.map_err(|e| format!("解析 Cursor usage CSV 行失败: {}", e))?;
        let timestamp = match parse_cursor_csv_timestamp(&row.date) {
            Some(value) => value,
            None => continue,
        };

        let model = clean_model_id(&row.model);
        if model.is_empty() {
            continue;
        }

        let input_tokens = parse_cursor_csv_u32(row.input_without_cache_write.as_deref());
        let cache_creation_tokens = parse_cursor_csv_u32(row.input_with_cache_write.as_deref());
        let cache_read_tokens = parse_cursor_csv_u32(row.cache_read.as_deref());
        let output_tokens = parse_cursor_csv_u32(row.output_tokens.as_deref());

        if input_tokens == 0
            && output_tokens == 0
            && cache_read_tokens == 0
            && cache_creation_tokens == 0
        {
            continue;
        }

        let cost_usd = parse_cursor_csv_cost(row.cost_to_you.as_deref())
            .or_else(|| parse_cursor_csv_cost(row.cost.as_deref()));

        let row_signature = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            account_id,
            row.date,
            row.kind.as_deref().unwrap_or(""),
            row.model,
            row.max_mode.as_deref().unwrap_or(""),
            row.input_with_cache_write.as_deref().unwrap_or(""),
            row.input_without_cache_write.as_deref().unwrap_or(""),
            row.cache_read.as_deref().unwrap_or(""),
            row.output_tokens.as_deref().unwrap_or(""),
            row.total_tokens.as_deref().unwrap_or(""),
        );

        let project_name = account_label
            .map(|label| label.trim())
            .filter(|label| !label.is_empty())
            .map(|label| format!("Cursor 官方用量 · {}", label));

        entries.push(LocalLogEntry {
            source: "cursor".to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            cost_usd,
            session_id: format!("cursor-official-{:x}", md5::compute(row_signature.as_bytes())),
            project_name,
        });

        range_start = range_start.min(timestamp);
        range_end = range_end.max(timestamp);
    }

    if entries.is_empty() {
        Ok(None)
    } else {
        Ok(Some(CursorOfficialImportBatch {
            entries,
            range_start,
            range_end,
        }))
    }
}

#[allow(dead_code)]
async fn load_cursor_official_import_batch() -> Result<Option<CursorOfficialImportBatch>, String> {
    let accounts = cursor_account::list_accounts();
    if accounts.is_empty() {
        return Ok(None);
    }

    let mut all_entries = Vec::new();
    let mut range_start = i64::MAX;
    let mut range_end = i64::MIN;

    for account in accounts {
        if account.access_token.trim().is_empty() {
            continue;
        }

        match fetch_cursor_usage_events_csv(&account.access_token).await {
            Ok(csv_text) => match parse_cursor_official_usage_csv(
                account.id.as_str(),
                Some(account.email.as_str()),
                &csv_text,
            ) {
                Ok(Some(batch)) => {
                    range_start = range_start.min(batch.range_start);
                    range_end = range_end.max(batch.range_end);
                    all_entries.extend(batch.entries);
                }
                Ok(None) => {}
                Err(err) => logger::log_warn(&format!(
                    "[Local Logs] 解析 Cursor 官方 usage CSV 失败: account={}, error={}",
                    account.id, err
                )),
            },
            Err(err) => logger::log_warn(&format!(
                "[Local Logs] 拉取 Cursor 官方 usage CSV 失败: account={}, error={}",
                account.id, err
            )),
        }
    }

    if all_entries.is_empty() {
        Ok(None)
    } else {
        Ok(Some(CursorOfficialImportBatch {
            entries: all_entries,
            range_start,
            range_end,
        }))
    }
}

fn parse_cursor_composer_data(
    conn: &rusqlite::Connection,
    json: &serde_json::Value,
    key: &str,
    workspace_id: Option<&str>,
    bubble_token_map: Option<&BubbleTokenMap>,
) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();
    
    // 从 key 提取 composer ID
    let composer_id = json.get("composerId")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| key.strip_prefix("composerData:").unwrap_or(key));
    
    // 获取模型
    let model = json.get("modelConfig")
        .and_then(|m| m.get("modelName"))
        .and_then(|v| v.as_str())
        .or_else(|| json.get("modelId").and_then(|v| v.as_str()))
        .or_else(|| json.get("model").and_then(|v| v.as_str()))
        .unwrap_or("cursor-composer");

    // 检查 conversation 数组 (inline storage)
    let conversation = json.get("conversation").and_then(|c| c.as_array());
    
    // 估算 token（优先 inline conversation，其次 bubbleId）
    let (input_tokens, output_tokens) = if let Some(msgs) = conversation {
        if msgs.is_empty() {
            (0, 0)
        } else {
            estimate_tokens_from_cursor_conversation(msgs, Some(model))
        }
    } else {
        (0, 0)
    };

    let (input_tokens, output_tokens) = if input_tokens == 0 && output_tokens == 0 {
        if let Some(map) = bubble_token_map {
            if let Some((in_tokens, out_tokens)) = map.get(composer_id) {
                (*in_tokens, *out_tokens)
            } else {
                (0, 0)
            }
        } else {
            let bubbles = load_cursor_bubbles(conn, composer_id);
            if bubbles.is_empty() {
                (0, 0)
            } else {
                estimate_tokens_from_messages(&bubbles, Some(model))
            }
        }
    } else {
        (input_tokens, output_tokens)
    };
    
    if input_tokens == 0 && output_tokens == 0 {
        return entries;
    }
    
    // 获取名称
    let name = json.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    
    
    // 获取时间戳
    let timestamp = extract_cursor_timestamp(json);
    
    let session_id = if let Some(id) = workspace_id {
        format!("cursor-global-{}-{}", id, composer_id)
    } else {
        format!("cursor-global-{}", composer_id)
    };
    
    entries.push(LocalLogEntry {
        source: "cursor".to_string(),
        timestamp,
        model: model.to_string(),
        input_tokens,
        output_tokens,
        cache_read_tokens: 0,
        cache_creation_tokens: 0,
        cost_usd: None,
        session_id,
        project_name: Some(name.to_string()),
    });
    
    entries
}

/// 解析 Cursor aiService 旧格式（pre-v0.43）
fn parse_cursor_aiservice(
    prompts: Option<serde_json::Value>,
    generations: Option<serde_json::Value>,
    workspace_id: Option<&str>,
) -> Vec<LocalLogEntry> {
    let mut entries = Vec::new();

    let prompts_arr = prompts.and_then(|v| v.as_array().cloned()).unwrap_or_default();
    let gens_arr = generations.and_then(|v| v.as_array().cloned()).unwrap_or_default();

    let max_len = std::cmp::max(prompts_arr.len(), gens_arr.len());
    if max_len == 0 {
        return entries;
    }

    for idx in 0..max_len {
        let prompt_text = prompts_arr.get(idx)
            .and_then(|p| p.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let gen_text = gens_arr.get(idx)
            .and_then(|g| g.get("text").or_else(|| g.get("message")))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let input_tokens = (prompt_text.len() / 4).max(if !prompt_text.is_empty() { 1 } else { 0 }) as u32;
        let output_tokens = (gen_text.len() / 4).max(if !gen_text.is_empty() { 1 } else { 0 }) as u32;

        if input_tokens == 0 && output_tokens == 0 {
            continue;
        }

        let model = gens_arr.get(idx)
            .and_then(|g| g.get("model").or_else(|| g.get("modelId")).or_else(|| g.get("modelName")))
            .and_then(|v| v.as_str())
            .unwrap_or("cursor-aiservice");

        let session_id = if let Some(id) = workspace_id {
            format!("cursor-aiservice-{}-{}", id, idx)
        } else {
            format!("cursor-aiservice-{}", idx)
        };

        entries.push(LocalLogEntry {
            source: "cursor".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            input_tokens,
            output_tokens,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: None,
            session_id,
            project_name: None,
        });
    }

    entries
}

/// 加载 Cursor bubbleId 存储的消息
fn load_cursor_bubbles(conn: &rusqlite::Connection, composer_id: &str) -> Vec<serde_json::Value> {
    let mut bubbles = Vec::new();
    let key_pattern = format!("bubbleId:{}:%", composer_id);

    if let Ok(mut stmt) = conn.prepare("SELECT value FROM cursorDiskKV WHERE key LIKE ?1") {
        if let Ok(rows) = stmt.query_map([key_pattern], |row| {
            let value = row
                .get::<_, Vec<u8>>(0)
                .ok()
                .or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes()));
            Ok(value)
        }) {
            for row_result in rows.flatten() {
                let Some(value_bytes) = row_result else { continue };
                if let Some(json) = parse_json_bytes(&value_bytes) {
                    bubbles.push(json);
                }
            }
        }
    }

    bubbles
}

/// 加载 bubbleId 的 token 统计（按 composer 聚合）
/// 包含 codeBlocks, selections, diffHistories, toolResults 的重度准确统计
fn load_cursor_bubble_token_map(conn: &rusqlite::Connection) -> BubbleTokenMap {
    let mut map: BubbleTokenMap = HashMap::new();

    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = row
                .get::<_, Vec<u8>>(1)
                .ok()
                .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for row_result in rows.flatten() {
                let (key, value_opt) = row_result;
                let Some(value_bytes) = value_opt else { continue };

                let mut parts = key.split(':');
                let _prefix = parts.next();
                let composer_id = match parts.next() {
                    Some(id) => id.to_string(),
                    None => continue,
                };

                let Some(json) = parse_json_bytes(&value_bytes) else { continue };

                let role = json.get("type")
                    .or_else(|| json.get("role"))
                    .and_then(|r| {
                        if let Some(s) = r.as_str() {
                            Some(s.to_string())
                        } else if let Some(n) = r.as_i64() {
                            Some(n.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                let model = json.get("modelId")
                    .or_else(|| json.get("model"))
                    .or_else(|| json.get("modelName"))
                    .and_then(|v| v.as_str());

                let is_user = role == "user" || role == "human" || role == "1";
                let is_assistant = role == "assistant" || role == "ai" || role == "bot" || role == "2";

                if is_user {
                    // User 输入
                    let mut input_text = String::new();
                    
                    // 主内容
                    if let Some(content) = json.get("rawText")
                        .or_else(|| json.get("text"))
                        .or_else(|| json.get("content"))
                        .and_then(|c| c.as_str()) {
                        input_text.push_str(content);
                    }
                    
                    // === 重度准确：selections ===
                    if let Some(sels) = json.get("selections") {
                        let sel_text = extract_selections_text(sels);
                        if !sel_text.is_empty() {
                            if !input_text.is_empty() { input_text.push('\n'); }
                            input_text.push_str(&sel_text);
                        }
                    }
                    // context.selections
                    if let Some(ctx) = json.get("context") {
                        if let Some(sels) = ctx.get("selections") {
                            let sel_text = extract_selections_text(sels);
                            if !sel_text.is_empty() {
                                if !input_text.is_empty() { input_text.push('\n'); }
                                input_text.push_str(&sel_text);
                            }
                        }
                    }
                    
                    let tokens = count_tokens_for_text(&input_text, model) as u32;
                    if tokens > 0 {
                        let entry = map.entry(composer_id).or_insert((0, 0));
                        entry.0 = entry.0.saturating_add(tokens);
                    }
                    
                } else if is_assistant {
                    // Assistant 输出
                    let mut output_text = String::new();
                    
                    // 主内容
                    if let Some(content) = json.get("rawText")
                        .or_else(|| json.get("text"))
                        .or_else(|| json.get("content"))
                        .and_then(|c| c.as_str()) {
                        output_text.push_str(content);
                    }
                    
                    // === 重度准确：codeBlocks ===
                    if let Some(blocks) = json.get("codeBlocks") {
                        let blocks_text = extract_code_blocks_text(blocks);
                        if !blocks_text.is_empty() {
                            if !output_text.is_empty() { output_text.push('\n'); }
                            output_text.push_str(&blocks_text);
                        }
                    }
                    
                    // === 重度准确：suggestedCodeBlocks ===
                    if let Some(blocks) = json.get("suggestedCodeBlocks") {
                        let blocks_text = extract_code_blocks_text(blocks);
                        if !blocks_text.is_empty() {
                            if !output_text.is_empty() { output_text.push('\n'); }
                            output_text.push_str(&blocks_text);
                        }
                    }
                    
                    // === 重度准确：suggestedDiffs ===
                    if let Some(diffs) = json.get("suggestedDiffs") {
                        let diffs_text = extract_diff_histories_text(diffs);
                        if !diffs_text.is_empty() {
                            if !output_text.is_empty() { output_text.push('\n'); }
                            output_text.push_str(&diffs_text);
                        }
                    }
                    
                    // === 重度准确：diffHistories ===
                    if let Some(diffs) = json.get("diffHistories") {
                        let diffs_text = extract_diff_histories_text(diffs);
                        if !diffs_text.is_empty() {
                            if !output_text.is_empty() { output_text.push('\n'); }
                            output_text.push_str(&diffs_text);
                        }
                    }
                    
                    // === 重度准确：toolResults ===
                    if let Some(results) = json.get("toolResults") {
                        let results_text = extract_tool_results_text(results);
                        if !results_text.is_empty() {
                            if !output_text.is_empty() { output_text.push('\n'); }
                            output_text.push_str(&results_text);
                        }
                    }
                    
                    let tokens = count_tokens_for_text(&output_text, model) as u32;
                    if tokens > 0 {
                        let entry = map.entry(composer_id).or_insert((0, 0));
                        entry.1 = entry.1.saturating_add(tokens);
                    }
                }
            }
        }
    }

    map
}

/// 从 selections / context.selections 提取代码上下文文本 (用于输入 token)
fn extract_selections_text(selections: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = selections.as_array() {
        for sel in arr {
            // 文件路径
            if let Some(uri) = sel.get("uri").and_then(|u| u.get("fsPath")).and_then(|p| p.as_str()) {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(uri);
            }
            // 选中的代码
            let code = sel.get("text")
                .or_else(|| sel.get("rawText"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            if !code.is_empty() {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(code);
            }
        }
    }
    text
}

/// 从 codeBlocks / suggestedCodeBlocks 提取代码块文本 (用于输出 token)
fn extract_code_blocks_text(blocks: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = blocks.as_array() {
        for block in arr {
            // code 或 text 字段
            let code = block.get("code")
                .or_else(|| block.get("text"))
                .or_else(|| block.get("content"))
                .and_then(|c| c.as_str());
            if let Some(c) = code {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(c);
            }
            // 可能有 uri/filePath
            if let Some(path) = block.get("uri")
                .and_then(|u| u.get("fsPath").or_else(|| u.get("path")))
                .and_then(|p| p.as_str())
                .or_else(|| block.get("filePath").and_then(|p| p.as_str())) {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(path);
            }
        }
    }
    text
}

/// 从 diffHistories 提取 diff 文本 (用于输出 token)
fn extract_diff_histories_text(diffs: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = diffs.as_array() {
        for diff in arr {
            // 常见字段: original, modified, oldCode, newCode, diff, patch
            for key in &["original", "modified", "oldCode", "newCode", "diff", "patch", "text", "code", "content"] {
                if let Some(val) = diff.get(*key).and_then(|v| v.as_str()) {
                    if !val.is_empty() {
                        if !text.is_empty() { text.push('\n'); }
                        text.push_str(val);
                    }
                }
            }
            // 文件路径
            if let Some(path) = diff.get("uri")
                .and_then(|u| u.get("fsPath").or_else(|| u.get("path")))
                .and_then(|p| p.as_str())
                .or_else(|| diff.get("filePath").and_then(|p| p.as_str())) {
                if !text.is_empty() { text.push('\n'); }
                text.push_str(path);
            }
        }
    }
    text
}

/// 从 toolResults 提取工具结果文本 (用于输出 token)
fn extract_tool_results_text(results: &serde_json::Value) -> String {
    let mut text = String::new();
    if let Some(arr) = results.as_array() {
        for result in arr {
            // 常见字段: output, result, content, text, stdout, stderr
            for key in &["output", "result", "content", "text", "stdout", "stderr"] {
                if let Some(val) = result.get(*key).and_then(|v| v.as_str()) {
                    if !val.is_empty() {
                        if !text.is_empty() { text.push('\n'); }
                        text.push_str(val);
                    }
                }
            }
        }
    }
    text
}

/// 从 Cursor conversation 数组估算 token (type 1=user, type 2=assistant)
/// 包含 codeBlocks, selections, diffHistories, toolResults 的重度准确统计
fn estimate_tokens_from_cursor_conversation(messages: &[serde_json::Value], model_hint: Option<&str>) -> (u32, u32) {
    let mut input_text = String::new();
    let mut output_text = String::new();
    let mut input_count = 0usize;
    let mut output_count = 0usize;
    
    for msg in messages {
        // Cursor 使用 type: 1 表示用户, type: 2 表示助手
        let msg_type = msg.get("type")
            .and_then(|t| t.as_i64())
            .or_else(|| {
                msg.get("role")
                    .and_then(|r| r.as_str())
                    .and_then(|s| if s == "user" { Some(1) } else if s == "assistant" { Some(2) } else { None })
            })
            .unwrap_or(0);
        
        // 获取主内容
        let content = msg.get("text")
            .or_else(|| msg.get("rawText"))
            .or_else(|| msg.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        
        if msg_type == 1 {
            // User 输入
            if !content.is_empty() {
                if !input_text.is_empty() { input_text.push('\n'); }
                input_text.push_str(content);
            }
            input_count += 1;
            
            // === 重度准确：提取 selections (context.selections 或直接 selections) ===
            if let Some(ctx) = msg.get("context") {
                if let Some(sels) = ctx.get("selections") {
                    let sel_text = extract_selections_text(sels);
                    if !sel_text.is_empty() {
                        if !input_text.is_empty() { input_text.push('\n'); }
                        input_text.push_str(&sel_text);
                    }
                }
            }
            if let Some(sels) = msg.get("selections") {
                let sel_text = extract_selections_text(sels);
                if !sel_text.is_empty() {
                    if !input_text.is_empty() { input_text.push('\n'); }
                    input_text.push_str(&sel_text);
                }
            }
            
        } else if msg_type == 2 {
            // Assistant 输出
            if !content.is_empty() {
                if !output_text.is_empty() { output_text.push('\n'); }
                output_text.push_str(content);
            }
            output_count += 1;
            
            // === 重度准确：提取 codeBlocks ===
            if let Some(blocks) = msg.get("codeBlocks") {
                let blocks_text = extract_code_blocks_text(blocks);
                if !blocks_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&blocks_text);
                }
            }
            
            // === 重度准确：提取 suggestedCodeBlocks ===
            if let Some(blocks) = msg.get("suggestedCodeBlocks") {
                let blocks_text = extract_code_blocks_text(blocks);
                if !blocks_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&blocks_text);
                }
            }
            
            // === 重度准确：提取 diffHistories ===
            if let Some(diffs) = msg.get("diffHistories") {
                let diffs_text = extract_diff_histories_text(diffs);
                if !diffs_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&diffs_text);
                }
            }
            
            // === 重度准确：提取 toolResults ===
            if let Some(results) = msg.get("toolResults") {
                let results_text = extract_tool_results_text(results);
                if !results_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&results_text);
                }
            }
        }
    }
    
    let model = model_hint.filter(|m| !m.is_empty());
    let input_tokens = count_tokens_for_text(&input_text, model) as u32;
    let output_tokens = count_tokens_for_text(&output_text, model) as u32;
    let input_tokens = if input_tokens == 0 && input_count > 0 { 1 } else { input_tokens };
    let output_tokens = if output_tokens == 0 && output_count > 0 { 1 } else { output_tokens };
    (input_tokens, output_tokens)
}

/// 从消息数组估算 token (用于 Chat 模式的 bubbles)
/// 包含 codeBlocks, selections, suggestedDiffs, diffHistories, toolResults 的重度准确统计
fn estimate_tokens_from_messages(messages: &[serde_json::Value], model_hint: Option<&str>) -> (u32, u32) {
    let mut input_text = String::new();
    let mut output_text = String::new();
    let mut input_count = 0usize;
    let mut output_count = 0usize;
    
    for msg in messages {
        // 获取角色
        let role = msg.get("type")
            .or_else(|| msg.get("role"))
            .and_then(|r| {
                if let Some(s) = r.as_str() {
                    Some(s.to_string())
                } else if let Some(n) = r.as_i64() {
                    Some(n.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        
        // 获取主内容
        let content = msg.get("rawText")
            .or_else(|| msg.get("text"))
            .or_else(|| msg.get("content"))
            .or_else(|| msg.get("message"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        
        // 根据角色分类
        let is_user = role == "user" || role == "human" || role == "1";
        let is_assistant = role == "assistant" || role == "ai" || role == "bot" || role == "2";
        
        if is_user {
            // User 输入
            if !content.is_empty() {
                if !input_text.is_empty() { input_text.push('\n'); }
                input_text.push_str(content);
            }
            input_count += 1;
            
            // === 重度准确：提取 selections ===
            if let Some(sels) = msg.get("selections") {
                let sel_text = extract_selections_text(sels);
                if !sel_text.is_empty() {
                    if !input_text.is_empty() { input_text.push('\n'); }
                    input_text.push_str(&sel_text);
                }
            }
            // context.selections
            if let Some(ctx) = msg.get("context") {
                if let Some(sels) = ctx.get("selections") {
                    let sel_text = extract_selections_text(sels);
                    if !sel_text.is_empty() {
                        if !input_text.is_empty() { input_text.push('\n'); }
                        input_text.push_str(&sel_text);
                    }
                }
            }
            
        } else if is_assistant {
            // Assistant 输出
            if !content.is_empty() {
                if !output_text.is_empty() { output_text.push('\n'); }
                output_text.push_str(content);
            }
            output_count += 1;
            
            // === 重度准确：提取 codeBlocks ===
            if let Some(blocks) = msg.get("codeBlocks") {
                let blocks_text = extract_code_blocks_text(blocks);
                if !blocks_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&blocks_text);
                }
            }
            
            // === 重度准确：提取 suggestedCodeBlocks ===
            if let Some(blocks) = msg.get("suggestedCodeBlocks") {
                let blocks_text = extract_code_blocks_text(blocks);
                if !blocks_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&blocks_text);
                }
            }
            
            // === 重度准确：提取 suggestedDiffs (Chat 模式使用) ===
            if let Some(diffs) = msg.get("suggestedDiffs") {
                let diffs_text = extract_diff_histories_text(diffs);
                if !diffs_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&diffs_text);
                }
            }
            
            // === 重度准确：提取 diffHistories ===
            if let Some(diffs) = msg.get("diffHistories") {
                let diffs_text = extract_diff_histories_text(diffs);
                if !diffs_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&diffs_text);
                }
            }
            
            // === 重度准确：提取 toolResults ===
            if let Some(results) = msg.get("toolResults") {
                let results_text = extract_tool_results_text(results);
                if !results_text.is_empty() {
                    if !output_text.is_empty() { output_text.push('\n'); }
                    output_text.push_str(&results_text);
                }
            }
        }
    }
    
    let model = model_hint.filter(|m| !m.is_empty());
    let input_tokens = count_tokens_for_text(&input_text, model) as u32;
    let output_tokens = count_tokens_for_text(&output_text, model) as u32;

    // 若内容为空，但存在消息数量，至少返回 1 token
    let input_tokens = if input_tokens == 0 && input_count > 0 { 1 } else { input_tokens };
    let output_tokens = if output_tokens == 0 && output_count > 0 { 1 } else { output_tokens };

    (input_tokens, output_tokens)
}

/// 提取 Cursor 数据的时间戳
fn extract_cursor_timestamp(json: &serde_json::Value) -> i64 {
    json.get("timestamp")
        .or_else(|| json.get("createdAt"))
        .or_else(|| json.get("created_at"))
        .or_else(|| json.get("lastUpdated"))
        .or_else(|| json.get("lastUpdatedAt"))
        .or_else(|| json.get("updatedAt"))
        .or_else(|| json.get("lastMessageAt"))
        .or_else(|| json.get("time"))
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
        .unwrap_or_else(|| chrono::Utc::now().timestamp())
}

// ============================================================================
// VSCode 系工具通用扫描（Windsurf / Kiro / Antigravity / Augment）
// ============================================================================

/// 获取 VSCode 系 app 的 state.vscdb 路径列表
fn get_vscode_app_db_paths(app_names: &[&str]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let mut base_dirs: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") { base_dirs.push(PathBuf::from(appdata)); }
        if let Ok(localapp) = std::env::var("LOCALAPPDATA") { base_dirs.push(PathBuf::from(localapp)); }
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join("AppData").join("Roaming"));
            base_dirs.push(home.join("AppData").join("Local"));
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join("Library").join("Application Support"));
            base_dirs.push(home.join(".config"));
        }
    }
    #[cfg(all(not(windows), not(target_os = "macos")))]
    {
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join(".config"));
            base_dirs.push(home.join(".local").join("share"));
        }
    }

    for base in &base_dirs {
        for name in app_names {
            let user_dir = base.join(name).join("User");
            if !user_dir.exists() { continue; }
            // globalStorage/state.vscdb
            let global_db = user_dir.join("globalStorage").join("state.vscdb");
            if global_db.exists() {
                let key = global_db.to_string_lossy().to_string();
                if seen.insert(key) { paths.push(global_db); }
            }
            // workspaceStorage/*/state.vscdb
            let workspace_dir = user_dir.join("workspaceStorage");
            if workspace_dir.exists() {
                if let Ok(entries) = fs::read_dir(&workspace_dir) {
                    for entry in entries.flatten() {
                        let db_path = entry.path().join("state.vscdb");
                        if db_path.exists() {
                            let key = db_path.to_string_lossy().to_string();
                            if seen.insert(key) { paths.push(db_path); }
                        }
                    }
                }
            }
        }
    }
    paths
}

/// 扫描 VSCode 系 app 的数据库文件
fn scan_vscode_app_logs(app_names: &[&str]) -> (Vec<PathBuf>, u32) {
    let db_paths = get_vscode_app_db_paths(app_names);
    if db_paths.is_empty() {
        return (vec![], 0);
    }
    let mut entry_count = 0u32;
    // 只统计 globalStorage 中的会话数
    if db_paths.len() > 8 {
        if let Some(global) = db_paths.iter().find(|p| p.to_string_lossy().contains("globalStorage")) {
            entry_count = count_cursor_sessions(global).unwrap_or(0);
        }
    } else {
        for path in &db_paths {
            entry_count = entry_count.saturating_add(count_cursor_sessions(path).unwrap_or(0));
        }
    }
    (db_paths, entry_count)
}

/// 解析 VSCode 系 app 的数据库，将 source 替换为指定工具名
fn parse_vscode_app_db(path: &PathBuf, source_name: &str) -> Vec<LocalLogEntry> {
    // 复用 Cursor 的解析逻辑，然后替换 source
    let entries = parse_cursor_db(path);
    entries.into_iter().map(|mut e| {
        // 替换 source 标识
        e.source = source_name.to_string();
        // 替换 session_id 前缀避免与 Cursor 冲突
        e.session_id = e.session_id.replace("cursor-", &format!("{}-", source_name));
        e
    }).collect()
}

// ---- Windsurf ----
fn scan_windsurf_logs() -> (Vec<PathBuf>, u32) {
    scan_vscode_app_logs(&["Windsurf", "WindSurf"])
}

fn get_windsurf_log_path() -> Option<PathBuf> {
    let paths = get_vscode_app_db_paths(&["Windsurf", "WindSurf"]);
    paths.into_iter().find(|p| p.to_string_lossy().contains("globalStorage"))
}

// ---- Kiro ----
fn scan_kiro_logs() -> (Vec<PathBuf>, u32) {
    scan_vscode_app_logs(&["Kiro"])
}

fn get_kiro_log_path() -> Option<PathBuf> {
    let paths = get_vscode_app_db_paths(&["Kiro"]);
    paths.into_iter().find(|p| p.to_string_lossy().contains("globalStorage"))
}

// ---- Antigravity ----
fn scan_antigravity_logs() -> (Vec<PathBuf>, u32) {
    scan_vscode_app_logs(&["Antigravity"])
}

fn get_antigravity_log_path() -> Option<PathBuf> {
    let paths = get_vscode_app_db_paths(&["Antigravity"]);
    paths.into_iter().find(|p| p.to_string_lossy().contains("globalStorage"))
}

// ---- Augment（VS Code 扩展，复用 VS Code 数据库）----
fn scan_augment_logs() -> (Vec<PathBuf>, u32) {
    // Augment 是 VS Code 扩展，数据存储在 VS Code 的 state.vscdb 中
    // 扫描 VS Code / Cursor 的 globalStorage 以查找 Augment 相关数据
    scan_vscode_app_logs(&["Code", "Code - Insiders"])
}

fn get_augment_log_path() -> Option<PathBuf> {
    let paths = get_vscode_app_db_paths(&["Code", "Code - Insiders"]);
    paths.into_iter().find(|p| p.to_string_lossy().contains("globalStorage"))
}

// ---- Trae ----
fn scan_trae_logs() -> (Vec<PathBuf>, u32) {
    scan_vscode_app_logs(&["Trae"])
}

fn get_trae_log_path() -> Option<PathBuf> {
    let paths = get_vscode_app_db_paths(&["Trae"]);
    paths.into_iter().find(|p| p.to_string_lossy().contains("globalStorage"))
}

// ============================================================================
// OpenClaw 日志解析 (JSONL 会话文件)
// ============================================================================

/// 获取 OpenClaw 会话目录
fn get_openclaw_sessions_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // 支持环境变量覆盖
    if let Ok(state_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        let sessions_dir = PathBuf::from(state_dir).join("agents").join("main").join("sessions");
        if sessions_dir.exists() { return Some(sessions_dir); }
    }

    let sessions_dir = home.join(".openclaw").join("agents").join("main").join("sessions");
    if sessions_dir.exists() { Some(sessions_dir) } else { None }
}

/// 获取 OpenClaw 根目录（用于显示）
fn get_openclaw_base_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    if let Ok(state_dir) = std::env::var("OPENCLAW_STATE_DIR") {
        let p = PathBuf::from(state_dir);
        if p.exists() { return Some(p); }
    }
    let p = home.join(".openclaw");
    if p.exists() { Some(p) } else { None }
}

/// 扫描 OpenClaw 会话文件
fn scan_openclaw_logs() -> (Vec<PathBuf>, u32) {
    let Some(sessions_dir) = get_openclaw_sessions_dir() else {
        return (vec![], 0);
    };

    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl")
                && path.file_name().and_then(|n| n.to_str()) != Some("sessions.json")
            {
                files.push(path);
            }
        }
    }

    let mut entry_count = 0u32;
    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines() {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if json.get("type").and_then(|t| t.as_str()) == Some("message") {
                        if let Some(msg) = json.get("message") {
                            if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                                if msg.get("usage").is_some() {
                                    entry_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (files, entry_count)
}

/// 解析 OpenClaw 会话文件为 LocalLogEntry
fn parse_openclaw_session_file(path: &PathBuf) -> Vec<LocalLogEntry> {
    let mut entries: Vec<LocalLogEntry> = Vec::new();
    let Ok(content) = fs::read_to_string(path) else {
        return entries;
    };

    let mut session_id = String::new();
    let mut session_cwd = String::new();
    let mut current_provider = String::from("openclaw");
    let mut current_model = String::from("unknown");

    for line in content.lines() {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        let msg_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "session" => {
                session_id = json.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                session_cwd = json.get("cwd")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
            }
            "model_change" => {
                if let Some(provider) = json.get("provider").and_then(|v| v.as_str()) {
                    current_provider = provider.to_string();
                }
                if let Some(model_id) = json.get("modelId").and_then(|v| v.as_str()) {
                    current_model = model_id.to_string();
                }
            }
            "message" => {
                let Some(msg) = json.get("message") else { continue; };
                if msg.get("role").and_then(|r| r.as_str()) != Some("assistant") { continue; }
                let Some(usage) = msg.get("usage") else { continue; };

                let input_tokens = usage.get("input").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let output_tokens = usage.get("output").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let cache_read = usage.get("cacheRead").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let cache_write = usage.get("cacheWrite").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                if input_tokens == 0 && output_tokens == 0 { continue; }

                let model = msg.get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current_model)
                    .to_string();

                let provider = msg.get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&current_provider)
                    .to_string();

                let msg_id = json.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let timestamp = json.get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp())
                    .or_else(|| msg.get("timestamp")
                        .and_then(|v| v.as_i64())
                        .map(|ts| if ts > 1_000_000_000_000 { ts / 1000 } else { ts })
                    )
                    .unwrap_or_else(|| chrono::Utc::now().timestamp());

                let cost_usd = usage.get("cost")
                    .and_then(|c| c.get("total"))
                    .and_then(|v| v.as_f64());

                let project_name = if !session_cwd.is_empty() {
                    PathBuf::from(&session_cwd)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                };

                let entry_id = format!("openclaw-{}-{}", session_id, msg_id);

                entries.push(LocalLogEntry {
                    session_id: entry_id,
                    model,
                    input_tokens,
                    output_tokens,
                    cache_creation_tokens: cache_write,
                    cache_read_tokens: cache_read,
                    timestamp,
                    source: "openclaw".to_string(),
                    project_name,
                    cost_usd,
                });
            }
            _ => {}
        }
    }

    entries
}

/// 解析 OpenClaw 会话统计
fn parse_openclaw_session_stats(path: &PathBuf) -> SessionStats {
    let mut stats = SessionStats::default();
    let Ok(content) = fs::read_to_string(path) else {
        return stats;
    };

    for line in content.lines() {
        let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else { continue; };
        let msg_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if msg_type != "message" { continue; }

        let Some(msg) = json.get("message") else { continue; };
        let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

        match role {
            "user" => {
                if let Some(content_arr) = msg.get("content").and_then(|c| c.as_array()) {
                    let has_tool_result = content_arr.iter().any(|item| {
                        item.get("type").and_then(|t| t.as_str()) == Some("toolResult")
                    });
                    if !has_tool_result {
                        stats.conversation_count += 1;
                    }
                }
            }
            "assistant" => {
                if let Some(content_arr) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content_arr {
                        let block_type = block.get("type").and_then(|t| t.as_str());
                        match block_type {
                            Some("thinking") => {
                                if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                                    stats.thinking_time_ms += (text.len() as u64).saturating_mul(2);
                                }
                            }
                            Some("toolCall") => {
                                if let Some(tool_name) = block.get("name").and_then(|n| n.as_str()) {
                                    let normalized = normalize_tool_name(tool_name);
                                    *stats.tool_calls.entry(normalized).or_insert(0) += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    stats
}

// ============================================================================
// Warp 日志解析
// ============================================================================

/// 获取 Warp 数据库路径
fn get_warp_db_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| home.join("AppData").join("Local").to_string_lossy().to_string());
        let db_path = PathBuf::from(&local_app_data).join("Warp").join("Warp").join("data").join("warp.sqlite");
        if db_path.exists() { return Some(db_path); }
    }
    #[cfg(target_os = "macos")]
    {
        let db_path = home
            .join("Library")
            .join("Group Containers")
            .join("2BBY89MBSN.dev.warp")
            .join("Library")
            .join("Application Support")
            .join("dev.warp.Warp-Stable")
            .join("warp.sqlite");
        if db_path.exists() { return Some(db_path); }
    }
    #[cfg(target_os = "linux")]
    {
        let db_path = home.join(".local").join("share").join("warp").join("warp.sqlite");
        if db_path.exists() { return Some(db_path); }
    }
    None
}

/// 扫描 Warp 数据库
fn scan_warp_logs() -> (Vec<PathBuf>, u32) {
    let Some(db_path) = get_warp_db_path() else {
        return (vec![], 0);
    };
    // 统计对话数
    let entry_count = count_warp_conversations(&db_path).unwrap_or(0);
    (vec![db_path], entry_count)
}

/// 统计 Warp 数据库中的对话数量
fn count_warp_conversations(db_path: &PathBuf) -> Option<u32> {
    use rusqlite::{Connection, OpenFlags};
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX).ok()?;
    // 检查表是否存在
    let has_table = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='agent_conversations'")
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
        .is_ok();
    if !has_table { return Some(0); }
    conn.query_row(
        "SELECT COUNT(*) FROM agent_conversations WHERE conversation_data IS NOT NULL",
        [],
        |row| row.get::<_, i64>(0),
    ).ok().map(|c| c as u32)
}

/// 解析 Warp 数据库，提取用量记录
fn parse_warp_db(db_path: &PathBuf) -> Vec<LocalLogEntry> {
    use rusqlite::{Connection, OpenFlags};
    let mut entries = Vec::new();

    let Ok(conn) = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX) else {
        return entries;
    };

    let has_table = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='agent_conversations'")
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
        .is_ok();
    if !has_table { return entries; }

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, conversation_data FROM agent_conversations WHERE conversation_data IS NOT NULL"
    ) else { return entries; };

    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let data: String = row.get(1)?;
        Ok((id, data))
    }) {
        Ok(r) => r,
        Err(_) => return entries,
    };

    for row in rows.flatten() {
        let (conv_id, data_str) = row;
        let Ok(data) = serde_json::from_str::<serde_json::Value>(&data_str) else { continue };

        let metadata = match data.get("conversation_usage_metadata") {
            Some(m) => m,
            None => continue,
        };

        // 解析 token_usage 数组
        let token_usage = metadata.get("token_usage").and_then(|v| v.as_array());
        if let Some(usages) = token_usage {
            for usage in usages {
                let model_id = usage.get("model_id").and_then(|v| v.as_str()).unwrap_or("warp-unknown");
                let total_tokens = usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let byok_tokens = usage.get("byok_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

                let all_tokens = total_tokens.saturating_add(byok_tokens);
                if all_tokens == 0 { continue; }

                // 估算输入/输出比例 (约 60% 输入, 40% 输出)
                let input_tokens = (all_tokens * 60 / 100) as u32;
                let output_tokens = (all_tokens - input_tokens as u64) as u32;

                let session_id = format!("warp-{}-{}", conv_id, model_id);

                entries.push(LocalLogEntry {
                    source: "warp".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    model: model_id.to_string(),
                    input_tokens,
                    output_tokens,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: None,
                    session_id,
                    project_name: None,
                });
            }
        } else {
            // 无 token_usage 但有 credits_spent
            let credits = metadata.get("credits_spent").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if credits > 0.0 {
                let session_id = format!("warp-{}", conv_id);
                entries.push(LocalLogEntry {
                    source: "warp".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                    model: "warp-agent".to_string(),
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    cost_usd: Some(credits),
                    session_id,
                    project_name: None,
                });
            }
        }
    }

    entries
}

// ============================================================================
// 数据库操作
// ============================================================================

/// 获取服务商特定的模型定价
fn get_provider_model_pricing(conn: &rusqlite::Connection, provider_id: &str, model_id: &str) -> Option<(Decimal, Decimal, Decimal, Decimal)> {
    for candidate in candidate_model_ids_for_pricing(model_id) {
        let result = conn.query_row(
            "SELECT input_cost_per_million, output_cost_per_million,
                    cache_read_cost_per_million, cache_creation_cost_per_million
             FROM provider_model_pricing WHERE provider_id = ?1 AND model_id = ?2",
            [provider_id, &candidate],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        );

        match result {
            Ok((input, output, cache_read, cache_creation)) => {
                return Some((
                    Decimal::from_str(&input).unwrap_or(Decimal::ZERO),
                    Decimal::from_str(&output).unwrap_or(Decimal::ZERO),
                    Decimal::from_str(&cache_read).unwrap_or(Decimal::ZERO),
                    Decimal::from_str(&cache_creation).unwrap_or(Decimal::ZERO),
                ));
            }
            Err(_) => {
                if let Some(default_pricing) = get_model_pricing_default(conn, &candidate) {
                    return Some(default_pricing);
                }
            }
        }
    }

    None
}

/// 获取默认模型定价
fn get_model_pricing_default(conn: &rusqlite::Connection, cleaned_model_id: &str) -> Option<(Decimal, Decimal, Decimal, Decimal)> {
    let result = conn.query_row(
        "SELECT input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM model_pricing WHERE model_id = ?1",
        [cleaned_model_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    match result {
        Ok((input, output, cache_read, cache_creation)) => Some((
            Decimal::from_str(&input).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&output).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_read).unwrap_or(Decimal::ZERO),
            Decimal::from_str(&cache_creation).unwrap_or(Decimal::ZERO),
        )),
        Err(_) => None,
    }
}

/// 清洗模型 ID
fn clean_model_id(model_id: &str) -> String {
    let without_prefix = model_id.rsplit_once('/').map_or(model_id, |(_, r)| r);
    let without_suffix = without_prefix.split(':').next().unwrap_or(without_prefix);
    without_suffix.trim().replace('@', "-")
}

fn push_pricing_candidate(candidates: &mut Vec<String>, candidate: impl Into<String>) {
    let candidate = candidate.into();
    if candidate.is_empty() || candidates.iter().any(|existing| existing == &candidate) {
        return;
    }
    candidates.push(candidate);
}

fn strip_model_variant_suffix(model_id: &str) -> String {
    model_id
        .trim_end_matches("-fast")
        .trim_end_matches("-thinking")
        .trim_end_matches("-xhigh")
        .trim_end_matches("-high")
        .trim_end_matches("-medium")
        .trim_end_matches("-max")
        .to_string()
}

fn candidate_model_ids_for_pricing(model_id: &str) -> Vec<String> {
    // 逗号分隔的复合模型名 -> 取第一个
    let primary = model_id.split(',').next().unwrap_or(model_id).trim();
    let cleaned = clean_model_id(primary);
    let stripped = strip_model_variant_suffix(&cleaned);
    let mut candidates = Vec::new();

    push_pricing_candidate(&mut candidates, cleaned.clone());
    push_pricing_candidate(&mut candidates, stripped.clone());

    let lower = stripped.to_lowercase();

    // Cursor 专属模型映射
    if lower == "cursor-composer" || lower == "cursor composer" {
        push_pricing_candidate(&mut candidates, "composer-2");
    }
    if lower.starts_with("composer-") {
        push_pricing_candidate(&mut candidates, &lower);
    }
    if lower == "cursor-aiservice" || lower == "default" {
        push_pricing_candidate(&mut candidates, "cursor-auto");
    }

    // GPT 系列
    if lower.starts_with("gpt-5.4-codex") {
        push_pricing_candidate(&mut candidates, "gpt-5.4-codex");
    }
    if lower.starts_with("gpt-5.4") {
        push_pricing_candidate(&mut candidates, "gpt-5.4");
    }
    if lower.starts_with("gpt-5.2-codex") {
        push_pricing_candidate(&mut candidates, "gpt-5.2-codex");
    }
    if lower.starts_with("gpt-5.2") {
        push_pricing_candidate(&mut candidates, "gpt-5.2");
    }
    if lower.starts_with("gpt-5.1-codex") {
        push_pricing_candidate(&mut candidates, "gpt-5.1-codex");
    }
    if lower.starts_with("gpt-5.1") {
        push_pricing_candidate(&mut candidates, "gpt-5.1");
    }
    if lower.starts_with("gpt-5-codex") {
        push_pricing_candidate(&mut candidates, "gpt-5-codex");
    }
    if lower.starts_with("gpt-5") {
        push_pricing_candidate(&mut candidates, "gpt-5");
    }

    // Claude 系列
    if lower.starts_with("claude-4.6-opus") || lower.starts_with("claude-4.5-opus") {
        push_pricing_candidate(&mut candidates, "claude-opus-4-5-20251101");
        push_pricing_candidate(&mut candidates, "claude-opus-4-20250514");
    }
    if lower.starts_with("claude-4.6-sonnet") || lower.starts_with("claude-4.5-sonnet") {
        push_pricing_candidate(&mut candidates, "claude-sonnet-4-5-20250929");
        push_pricing_candidate(&mut candidates, "claude-sonnet-4-20250514");
    }
    if lower.starts_with("claude-4-sonnet") {
        push_pricing_candidate(&mut candidates, "claude-sonnet-4-20250514");
    }
    if lower.starts_with("claude-4-opus") {
        push_pricing_candidate(&mut candidates, "claude-opus-4-20250514");
    }
    if lower.starts_with("claude-3-5-sonnet") || lower.starts_with("claude-3.5-sonnet") {
        push_pricing_candidate(&mut candidates, "claude-3-5-sonnet-20241022");
    }
    if lower.starts_with("claude-3-5-haiku") || lower.starts_with("claude-3.5-haiku") {
        push_pricing_candidate(&mut candidates, "claude-3-5-haiku-20241022");
    }

    // Gemini 系列
    if lower.starts_with("gemini-3-pro") {
        push_pricing_candidate(&mut candidates, "gemini-3-pro-preview");
    }
    if lower.starts_with("gemini-3-flash") {
        push_pricing_candidate(&mut candidates, "gemini-3-flash-preview");
    }
    if lower.starts_with("gemini-2.5-pro") {
        push_pricing_candidate(&mut candidates, "gemini-2.5-pro");
    }
    if lower.starts_with("gemini-2.5-flash") {
        push_pricing_candidate(&mut candidates, "gemini-2.5-flash");
    }

    candidates
}

/// 计算成本
fn calculate_cost(entry: &LocalLogEntry, pricing: Option<(Decimal, Decimal, Decimal, Decimal)>) -> Decimal {
    let Some((input_price, output_price, cache_read_price, cache_creation_price)) = pricing else {
        return Decimal::ZERO;
    };

    let million = Decimal::from(1_000_000u64);
    
    let billable_input = (entry.input_tokens as u64).saturating_sub(entry.cache_read_tokens as u64);
    
    let input_cost = Decimal::from(billable_input) * input_price / million;
    let output_cost = Decimal::from(entry.output_tokens as u64) * output_price / million;
    let cache_read_cost = Decimal::from(entry.cache_read_tokens as u64) * cache_read_price / million;
    let cache_creation_cost = Decimal::from(entry.cache_creation_tokens as u64) * cache_creation_price / million;

    input_cost + output_cost + cache_read_cost + cache_creation_cost
}

/// 补齐缓存 token 后重算 Cursor 条目费用
fn patch_cursor_costs_after_cache_fix(conn: &rusqlite::Connection) {
    let Ok(mut stmt) = conn.prepare(
        "SELECT request_id, model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens
         FROM proxy_request_logs
         WHERE app_type = 'cursor_local' AND cache_read_tokens > 0 AND total_cost_usd = '0'"
    ) else { return };

    let Ok(rows) = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, u32>(3)?,
            row.get::<_, u32>(4)?,
            row.get::<_, u32>(5)?,
        ))
    }) else { return };

    let entries: Vec<_> = rows.flatten().collect();
    for (request_id, model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens) in entries {
        let entry = LocalLogEntry {
            session_id: request_id.clone(),
            source: "cursor".to_string(),
            model: model.clone(),
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            timestamp: 0,
            project_name: None,
            cost_usd: None,
        };
        let cost = resolve_entry_cost(conn, &entry);
        if cost > Decimal::ZERO {
            let _ = conn.execute(
                "UPDATE proxy_request_logs SET total_cost_usd = ?1 WHERE request_id = ?2",
                rusqlite::params![cost.to_string(), request_id],
            );
        }
    }
}

/// 加载已存在的 request_id（按前缀过滤）
fn load_existing_request_ids_by_prefix(conn: &rusqlite::Connection, prefix: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let like_pattern = format!("{prefix}%");
    if let Ok(mut stmt) = conn.prepare("SELECT request_id FROM proxy_request_logs WHERE request_id LIKE ?1") {
        if let Ok(rows) = stmt.query_map([like_pattern], |row| row.get::<_, String>(0)) {
            for row_result in rows.flatten() {
                set.insert(row_result);
            }
        }
    }
    set
}

fn load_existing_request_ids_by_app_type(conn: &rusqlite::Connection, app_type: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT request_id FROM proxy_request_logs WHERE app_type = ?1") {
        if let Ok(rows) = stmt.query_map([app_type], |row| row.get::<_, String>(0)) {
            for row_result in rows.flatten() {
                set.insert(row_result);
            }
        }
    }
    set
}

fn load_existing_request_ids_for_source(
    conn: &rusqlite::Connection,
    source: &str,
) -> HashSet<String> {
    match source {
        "cursor" => load_existing_request_ids_by_prefix(conn, "cursor-"),
        _ => load_existing_request_ids_by_app_type(conn, &format!("{source}_local")),
    }
}

fn build_local_stats_session_id(source: &str, file: &PathBuf) -> String {
    match source {
        "cursor" | "windsurf" | "kiro" | "antigravity" | "augment" | "trae" => {
            format!("{}:{}", source, file.to_string_lossy().replace(['\\', '/', ':'], "_"))
        }
        _ => file
            .file_stem()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    }
}

fn delete_log_entries_by_request_id_prefix(
    conn: &rusqlite::Connection,
    prefix: &str,
) -> Result<usize, AppError> {
    let like_pattern = format!("{prefix}%");
    conn.execute(
        "DELETE FROM proxy_request_logs WHERE request_id LIKE ?1",
        rusqlite::params![like_pattern],
    )
    .map_err(|e| AppError::Database(format!("按前缀删除日志条目失败: {e}")))
}

/// 插入日志条目到数据库
fn insert_log_entry(conn: &rusqlite::Connection, entry: &LocalLogEntry, cost: Decimal) -> Result<(), AppError> {
    let app_type = format!("{}_local", entry.source);
    let provider_id = format!("{}_local", entry.source);
    let provider_name = match entry.source.as_str() {
        "claude" => "Claude Code (Local)",
        "codex" => "Codex CLI (Local)",
        "gemini" => "Gemini CLI (Local)",
        "opencode" => "Opencode (Local)",
        "cursor" => "Cursor (Local)",
        "windsurf" => "Windsurf (Local)",
        "kiro" => "Kiro (Local)",
        "antigravity" => "Antigravity (Local)",
        "warp" => "Warp (Local)",
        "augment" => "Augment (Local)",
        _ => "Local Import",
    };

    let zero = Decimal::ZERO;
    let project_name = entry.project_name.as_ref().map(|name| name.trim()).filter(|name| !name.is_empty());
    
    conn.execute(
        "INSERT INTO proxy_request_logs (
            request_id, provider_id, provider_name, app_type, model, project_name,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
            input_cost_usd, output_cost_usd, cache_read_cost_usd, cache_creation_cost_usd, total_cost_usd,
            latency_ms, status_code, is_streaming, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
        rusqlite::params![
            entry.session_id,
            provider_id,
            provider_name,
            app_type,
            entry.model,
            project_name,
            entry.input_tokens,
            entry.output_tokens,
            entry.cache_read_tokens,
            entry.cache_creation_tokens,
            zero.to_string(),
            zero.to_string(),
            zero.to_string(),
            zero.to_string(),
            cost.to_string(),
            0i64, // latency_ms
            200i64, // status_code
            0, // is_streaming
            entry.timestamp,
        ],
    )
    .map_err(|e| AppError::Database(format!("插入日志条目失败: {e}")))?;

    Ok(())
}

/// 更新已存在的日志条目（用于重新导入 Cursor）
fn update_log_entry(conn: &rusqlite::Connection, entry: &LocalLogEntry, cost: Decimal) -> Result<(), AppError> {
    let app_type = format!("{}_local", entry.source);
    let provider_id = format!("{}_local", entry.source);
    let provider_name = match entry.source.as_str() {
        "claude" => "Claude Code (Local)",
        "codex" => "Codex CLI (Local)",
        "gemini" => "Gemini CLI (Local)",
        "opencode" => "Opencode (Local)",
        "cursor" => "Cursor (Local)",
        "windsurf" => "Windsurf (Local)",
        "kiro" => "Kiro (Local)",
        "antigravity" => "Antigravity (Local)",
        "warp" => "Warp (Local)",
        "augment" => "Augment (Local)",
        _ => "Local Import",
    };
    let project_name = entry.project_name.as_ref().map(|name| name.trim()).filter(|name| !name.is_empty());

    conn.execute(
        "UPDATE proxy_request_logs SET
            provider_id = ?1,
            provider_name = ?2,
            app_type = ?3,
            model = ?4,
            project_name = ?5,
            input_tokens = ?6,
            output_tokens = ?7,
            cache_read_tokens = ?8,
            cache_creation_tokens = ?9,
            total_cost_usd = ?10,
            created_at = ?11
         WHERE request_id = ?12",
        rusqlite::params![
            provider_id,
            provider_name,
            app_type,
            entry.model,
            project_name,
            entry.input_tokens,
            entry.output_tokens,
            entry.cache_read_tokens,
            entry.cache_creation_tokens,
            cost.to_string(),
            entry.timestamp,
            entry.session_id,
        ],
    )
    .map_err(|e| AppError::Database(format!("更新日志条目失败: {e}")))?;

    Ok(())
}

fn resolve_entry_cost(conn: &rusqlite::Connection, entry: &LocalLogEntry) -> Decimal {
    let provider_id = format!("{}_local", entry.source);
    let pricing = get_provider_model_pricing(conn, &provider_id, &entry.model);
    entry
        .cost_usd
        .map(|c| Decimal::from_str(&c.to_string()).unwrap_or(Decimal::ZERO))
        .unwrap_or_else(|| calculate_cost(entry, pricing))
}

/// 获取已导入的本地记录数
fn get_existing_local_records(conn: &rusqlite::Connection) -> u32 {
    conn.query_row(
        "SELECT COUNT(*) FROM proxy_request_logs WHERE app_type LIKE '%_local'",
        [],
        |row| row.get::<_, i64>(0),
    )
    .map(|c| c as u32)
    .unwrap_or(0)
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// 调试：获取 Cursor 数据库中的 key 列表
#[tauri::command]
pub async fn debug_cursor_db_keys() -> Result<Vec<String>, String> {
    use rusqlite::{Connection, OpenFlags};
    
    let Some(db_path) = get_cursor_db_path() else {
        return Err("Cursor 数据库不存在".to_string());
    };
    
    let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("打开数据库失败: {e}"))?;
    
    let mut stmt = conn.prepare("SELECT key FROM ItemTable ORDER BY key LIMIT 500")
        .map_err(|e| format!("准备查询失败: {e}"))?;
    
    let keys: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| format!("查询失败: {e}"))?
        .filter_map(|r| r.ok())
        .collect();
    
    Ok(keys)
}

/// 调试：检查 Cursor 消息中的字段结构
#[tauri::command]
pub async fn debug_cursor_message_fields() -> Result<serde_json::Value, String> {
    use std::collections::HashSet;
    
    let db_paths = get_cursor_db_paths();
    if db_paths.is_empty() {
        return Err("Cursor 数据库不存在".to_string());
    }
    
    let mut all_fields: HashSet<String> = HashSet::new();
    let mut sample_messages: Vec<serde_json::Value> = Vec::new();
    let mut message_count = 0u32;
    
    for db_path in db_paths {
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            // 检查 Global Composer 数据
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'bubbleId:%' LIMIT 10") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    let value = row.get::<_, Vec<u8>>(1)
                        .ok()
                        .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
                    Ok(value)
                }) {
                    for row_result in rows.flatten() {
                        if let Some(value_bytes) = row_result {
                            if let Some(json) = parse_json_bytes(&value_bytes) {
                                message_count += 1;
                                // 收集所有字段名
                                if let Some(obj) = json.as_object() {
                                    for key in obj.keys() {
                                        all_fields.insert(key.clone());
                                    }
                                }
                                // 保存样本消息（最多 3 条）
                                if sample_messages.len() < 3 {
                                    sample_messages.push(json);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut fields_list: Vec<String> = all_fields.into_iter().collect();
    fields_list.sort();
    
    Ok(serde_json::json!({
        "total_messages_checked": message_count,
        "all_fields": fields_list,
        "sample_messages": sample_messages,
    }))
}

/// 获取 Cursor 对话统计
#[tauri::command]
pub async fn get_cursor_conversation_stats() -> Result<CursorConversationStats, String> {
    let db_paths = get_cursor_db_paths();
    if db_paths.is_empty() {
        return Ok(CursorConversationStats::default());
    }

    let mut stats = CursorConversationStats::default();

    for db_path in db_paths {
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            // 统计 Chat 模式
            if let Ok(chat_stats) = count_cursor_chat_stats(&conn) {
                stats.total_conversations += chat_stats.0;
                stats.total_messages += chat_stats.1;
                stats.tool_calls += chat_stats.2;
                stats.files_changed += chat_stats.3;
                stats.code_blocks += chat_stats.4;
                stats.diff_count += chat_stats.5;
                stats.lines_added += chat_stats.6;
                stats.lines_deleted += chat_stats.7;
                for (tool, count) in chat_stats.8 {
                    *stats.tool_call_details.entry(tool).or_insert(0) += count;
                }
            }

            // 统计 Workspace Composer
            if let Ok(composer_stats) = count_cursor_workspace_composer_stats(&conn) {
                stats.total_conversations += composer_stats.0;
                stats.total_messages += composer_stats.1;
                stats.tool_calls += composer_stats.2;
                stats.files_changed += composer_stats.3;
                stats.code_blocks += composer_stats.4;
                stats.diff_count += composer_stats.5;
                stats.lines_added += composer_stats.6;
                stats.lines_deleted += composer_stats.7;
                for (tool, count) in composer_stats.8 {
                    *stats.tool_call_details.entry(tool).or_insert(0) += count;
                }
            }

            // 统计 Global Composer (cursorDiskKV)
            if let Ok(global_stats) = count_cursor_global_composer_stats(&conn) {
                stats.total_conversations += global_stats.0;
                stats.total_messages += global_stats.1;
                stats.tool_calls += global_stats.2;
                stats.files_changed += global_stats.3;
                stats.code_blocks += global_stats.4;
                stats.diff_count += global_stats.5;
                stats.lines_added += global_stats.6;
                stats.lines_deleted += global_stats.7;
                for (tool, count) in global_stats.8 {
                    *stats.tool_call_details.entry(tool).or_insert(0) += count;
                }
            }
        }
    }

    // 获取 Cursor MCP 数量
    stats.mcp_count = get_cursor_mcp_count();

    // 计算对话累计持续时间
    for db_path in get_cursor_db_paths() {
        if let Ok(conn) = rusqlite::Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            stats.total_duration_ms += calculate_cursor_duration(&conn);
        }
    }

    Ok(stats)
}

/// 计算 Cursor 实际使用时间（毫秒）
/// 收集所有对话的时间段，取并集后计算总时长
fn calculate_cursor_duration(conn: &rusqlite::Connection) -> u64 {
    let mut time_ranges: Vec<(i64, i64)> = Vec::new();

    // 1. 从 Chat 模式获取时间段
    if let Some(value) = query_itemtable_value(conn, "workbench.panel.aichat.view.aichat.chatdata") {
        if let Some(json) = parse_json_bytes(&value) {
            if let Some(tabs) = json.get("tabs").and_then(|t| t.as_array()) {
                for tab in tabs {
                    if let Some((start, end)) = get_conversation_time_range(tab) {
                        time_ranges.push((start, end));
                    }
                }
            }
        }
    }

    // 2. 从 Workspace Composer 获取时间段
    if let Some(value) = query_itemtable_value(conn, "composer.composerData") {
        if let Some(json) = parse_json_bytes(&value) {
            if let Some(all_composers) = json.get("allComposers").and_then(|a| a.as_array()) {
                for composer in all_composers {
                    if let Some((start, end)) = get_conversation_time_range(composer) {
                        time_ranges.push((start, end));
                    }
                }
            }
        }
    }

    // 3. 从 Global Composer (cursorDiskKV) 获取时间段
    if let Ok(mut stmt) = conn.prepare("SELECT value FROM cursorDiskKV WHERE key LIKE 'composerData:%'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let value = row
                .get::<_, Vec<u8>>(0)
                .ok()
                .or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes()));
            Ok(value)
        }) {
            for row_result in rows.flatten() {
                let Some(value_bytes) = row_result else { continue };
                if let Some(json) = parse_json_bytes(&value_bytes) {
                    if let Some((start, end)) = get_conversation_time_range(&json) {
                        time_ranges.push((start, end));
                    }
                }
            }
        }
    }

    // 合并重叠的时间段并计算总时长
    merge_time_ranges_and_sum(&mut time_ranges)
}

/// 获取对话的时间范围 (createdAt, lastUpdated)，单位毫秒
fn get_conversation_time_range(json: &serde_json::Value) -> Option<(i64, i64)> {
    // 获取创建时间
    let created_at = json.get("createdAt")
        .or_else(|| json.get("created_at"))
        .or_else(|| json.get("timestamp"))
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                Some(ts)
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp_millis())
            } else {
                None
            }
        })?;

    // 获取最后更新时间
    let last_updated = json.get("lastUpdated")
        .or_else(|| json.get("lastUpdatedAt"))
        .or_else(|| json.get("updatedAt"))
        .or_else(|| json.get("lastMessageAt"))
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                Some(ts)
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp_millis())
            } else {
                None
            }
        })?;

    // 转换为毫秒（如果是秒级时间戳）
    let start_ms = if created_at < 1_000_000_000_000 {
        created_at * 1000
    } else {
        created_at
    };
    
    let end_ms = if last_updated < 1_000_000_000_000 {
        last_updated * 1000
    } else {
        last_updated
    };

    // 只有有效的时间段才返回（开始小于结束，且时间跨度合理，不超过24小时）
    if end_ms > start_ms {
        let duration = end_ms - start_ms;
        // 单次对话时间超过24小时的认为是异常数据，跳过
        if duration < 24 * 60 * 60 * 1000 {
            return Some((start_ms, end_ms));
        }
    }

    None
}

/// 合并重叠的时间段并计算总时长
fn merge_time_ranges_and_sum(ranges: &mut Vec<(i64, i64)>) -> u64 {
    if ranges.is_empty() {
        return 0;
    }

    // 按开始时间排序
    ranges.sort_by_key(|r| r.0);

    let mut merged: Vec<(i64, i64)> = Vec::new();
    let mut current = ranges[0];

    for &(start, end) in ranges.iter().skip(1) {
        if start <= current.1 {
            // 有重叠，合并
            current.1 = current.1.max(end);
        } else {
            // 无重叠，保存当前段，开始新段
            merged.push(current);
            current = (start, end);
        }
    }
    merged.push(current);

    // 计算总时长
    merged.iter().map(|(s, e)| (e - s) as u64).sum()
}


/// 获取 Cursor MCP 服务器数量
fn get_cursor_mcp_count() -> u32 {
    // Cursor MCP 配置文件路径
    // Windows: %USERPROFILE%\.cursor\mcp.json
    // macOS/Linux: ~/.cursor/mcp.json
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return 0,
    };
    
    let mcp_path = home.join(".cursor").join("mcp.json");
    
    if !mcp_path.exists() {
        return 0;
    }
    
    // 读取并解析 MCP 配置文件
    let content = match std::fs::read_to_string(&mcp_path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(_) => return 0,
    };
    
    // 统计 mcpServers 中的服务器数量
    if let Some(servers) = json.get("mcpServers").and_then(|s| s.as_object()) {
        servers.len() as u32
    } else {
        0
    }
}

/// 统计 Chat 模式数据
fn count_cursor_chat_stats(conn: &rusqlite::Connection) -> Result<CursorStatsResult, String> {
    let mut conversations = 0u32;
    let mut messages = 0u32;
    let mut tool_calls = 0u32;
    let mut files_changed = 0u32;
    let mut code_blocks = 0u32;
    let mut diff_count = 0u32;
    let mut lines_added = 0u32;
    let mut lines_deleted = 0u32;
    let mut tool_details: HashMap<String, u32> = HashMap::new();

    if let Some(value) = query_itemtable_value(conn, "workbench.panel.aichat.view.aichat.chatdata") {
        if let Some(json) = parse_json_bytes(&value) {
            if let Some(tabs) = json.get("tabs").and_then(|t| t.as_array()) {
                for tab in tabs {
                    if let Some(bubbles) = tab.get("bubbles").and_then(|b| b.as_array()) {
                        if bubbles.is_empty() {
                            continue;
                        }
                        conversations += 1;
                        messages += bubbles.len() as u32;

                        for bubble in bubbles {
                            count_message_stats(bubble, &mut tool_calls, &mut files_changed, &mut code_blocks, &mut diff_count, &mut lines_added, &mut lines_deleted, &mut tool_details);
                        }
                    }
                }
            }
        }
    }

    Ok((conversations, messages, tool_calls, files_changed, code_blocks, diff_count, lines_added, lines_deleted, tool_details))
}

/// 统计 Workspace Composer 数据
fn count_cursor_workspace_composer_stats(conn: &rusqlite::Connection) -> Result<CursorStatsResult, String> {
    let mut conversations = 0u32;
    let mut messages = 0u32;
    let mut tool_calls = 0u32;
    let mut files_changed = 0u32;
    let mut code_blocks = 0u32;
    let mut diff_count = 0u32;
    let mut lines_added = 0u32;
    let mut lines_deleted = 0u32;
    let mut tool_details: HashMap<String, u32> = HashMap::new();

    if let Some(value) = query_itemtable_value(conn, "composer.composerData") {
        if let Some(json) = parse_json_bytes(&value) {
            if let Some(all_composers) = json.get("allComposers").and_then(|a| a.as_array()) {
                for composer in all_composers {
                    if let Some(conversation) = composer.get("conversation").and_then(|c| c.as_array()) {
                        if conversation.is_empty() {
                            continue;
                        }
                        conversations += 1;
                        messages += conversation.len() as u32;

                        for msg in conversation {
                            count_message_stats(msg, &mut tool_calls, &mut files_changed, &mut code_blocks, &mut diff_count, &mut lines_added, &mut lines_deleted, &mut tool_details);
                        }
                    }
                }
            }
        }
    }

    Ok((conversations, messages, tool_calls, files_changed, code_blocks, diff_count, lines_added, lines_deleted, tool_details))
}

/// Cursor 统计结果类型
type CursorStatsResult = (u32, u32, u32, u32, u32, u32, u32, u32, HashMap<String, u32>);
// (conversations, messages, tool_calls, files_changed, code_blocks, diff_count, lines_added, lines_deleted, tool_details)

/// 统计 Global Composer 数据 (cursorDiskKV)
fn count_cursor_global_composer_stats(conn: &rusqlite::Connection) -> Result<CursorStatsResult, String> {
    let mut messages = 0u32;
    let mut tool_calls = 0u32;
    let mut files_changed = 0u32;
    let mut code_blocks = 0u32;
    let mut diff_count = 0u32;
    let mut lines_added = 0u32;
    let mut lines_deleted = 0u32;
    let mut tool_details: HashMap<String, u32> = HashMap::new();

    // 收集所有 composer ID（用于统计对话数）
    let mut composer_ids: HashSet<String> = HashSet::new();
    let mut inline_composer_ids: HashSet<String> = HashSet::new();

    // 统计 composerData:{uuid}
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = row
                .get::<_, Vec<u8>>(1)
                .ok()
                .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for row_result in rows.flatten() {
                let (key, value_opt) = row_result;
                let Some(value_bytes) = value_opt else { continue };
                let Some(json) = parse_json_bytes(&value_bytes) else { continue };

                // 提取 composer ID
                let composer_id = json.get("composerId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| key.strip_prefix("composerData:").unwrap_or(&key).to_string());

                composer_ids.insert(composer_id.clone());

                // 检查 inline conversation
                if let Some(conversation) = json.get("conversation").and_then(|c| c.as_array()) {
                    if !conversation.is_empty() {
                        inline_composer_ids.insert(composer_id);
                        messages += conversation.len() as u32;

                        for msg in conversation {
                            count_message_stats(msg, &mut tool_calls, &mut files_changed, &mut code_blocks, &mut diff_count, &mut lines_added, &mut lines_deleted, &mut tool_details);
                        }
                    }
                }
            }
        }
    }

    // 统计 bubbleId:{composer}:{bubble} (separate storage)
    // 同时收集 bubble 中的 composer ID
    let mut bubble_composer_ids: HashSet<String> = HashSet::new();
    
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = row
                .get::<_, Vec<u8>>(1)
                .ok()
                .or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for row_result in rows.flatten() {
                let (key, value_opt) = row_result;
                let Some(value_bytes) = value_opt else { continue };
                let Some(json) = parse_json_bytes(&value_bytes) else { continue };

                // 提取 composer ID (bubbleId:{composerId}:{bubbleId})
                let parts: Vec<&str> = key.split(':').collect();
                if parts.len() >= 2 {
                    let composer_id = parts[1].to_string();
                    // 只有不在 inline_composer_ids 中的才统计（避免重复）
                    if !inline_composer_ids.contains(&composer_id) {
                        bubble_composer_ids.insert(composer_id);
                    }
                }

                messages += 1;
                count_message_stats(&json, &mut tool_calls, &mut files_changed, &mut code_blocks, &mut diff_count, &mut lines_added, &mut lines_deleted, &mut tool_details);
            }
        }
    }

    // 对话数 = inline 对话数 + bubble-only 对话数
    let conversations = (inline_composer_ids.len() + bubble_composer_ids.len()) as u32;

    Ok((conversations, messages, tool_calls, files_changed, code_blocks, diff_count, lines_added, lines_deleted, tool_details))
}

/// 统计单条消息的各项指标（包含行数统计）
fn count_message_stats(
    msg: &serde_json::Value,
    tool_calls: &mut u32,
    files_changed: &mut u32,
    code_blocks: &mut u32,
    diff_count: &mut u32,
    lines_added: &mut u32,
    lines_deleted: &mut u32,
    tool_details: &mut HashMap<String, u32>,
) {
    // 统计代码块 (多种可能的字段名)
    for key in &["codeBlocks", "suggestedCodeBlocks", "code_blocks", "suggested_code_blocks"] {
        if let Some(blocks) = msg.get(*key).and_then(|b| b.as_array()) {
            *code_blocks += blocks.len() as u32;
            // 从代码块内容估算新增行数
            for block in blocks {
                if let Some(code) = block.get("code")
                    .or_else(|| block.get("text"))
                    .or_else(|| block.get("content"))
                    .and_then(|c| c.as_str()) {
                    *lines_added += code.lines().count() as u32;
                }
            }
        }
    }

    // 统计 diff (多种可能的字段名)
    for key in &["suggestedDiffs", "diffHistories", "diffs", "suggested_diffs", "diff_histories", "edits", "changes"] {
        if let Some(diffs) = msg.get(*key).and_then(|d| d.as_array()) {
            *diff_count += diffs.len() as u32;
            // suggestedDiffs 和 diffs 通常表示文件变更
            if *key == "suggestedDiffs" || *key == "diffs" || *key == "edits" || *key == "changes" {
                *files_changed += diffs.len() as u32;
            }
            // 从 diff 内容估算行数变更
            for diff in diffs {
                // 尝试读取 additions/deletions 字段
                if let Some(adds) = diff.get("additions").or_else(|| diff.get("linesAdded")).and_then(|v| v.as_u64()) {
                    *lines_added += adds as u32;
                }
                if let Some(dels) = diff.get("deletions").or_else(|| diff.get("linesDeleted")).and_then(|v| v.as_u64()) {
                    *lines_deleted += dels as u32;
                }
                // 如果没有明确字段，尝试从 diff 文本估算
                if let Some(diff_text) = diff.get("diff").or_else(|| diff.get("patch")).and_then(|d| d.as_str()) {
                    for line in diff_text.lines() {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            *lines_added += 1;
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            *lines_deleted += 1;
                        }
                    }
                }
                // 从 newCode/oldCode 估算
                if let Some(new_code) = diff.get("newCode").or_else(|| diff.get("modified")).and_then(|c| c.as_str()) {
                    *lines_added += new_code.lines().count() as u32;
                }
                if let Some(old_code) = diff.get("oldCode").or_else(|| diff.get("original")).and_then(|c| c.as_str()) {
                    *lines_deleted += old_code.lines().count() as u32;
                }
            }
        }
    }

    // 检查 context 中的文件信息
    if let Some(ctx) = msg.get("context") {
        // context.selections 中的文件
        if let Some(sels) = ctx.get("selections").and_then(|s| s.as_array()) {
            // 统计不同的文件
            let file_set: HashSet<String> = sels.iter()
                .filter_map(|s| s.get("uri").and_then(|u| u.get("fsPath")).and_then(|p| p.as_str()))
                .map(|s| s.to_string())
                .collect();
            *files_changed += file_set.len() as u32;
        }
    }

    // 统计工具调用 (多种可能的字段名)
    for key in &["toolResults", "toolCalls", "tool_results", "tool_calls", "tools", "actions"] {
        if let Some(tools) = msg.get(*key).and_then(|t| t.as_array()) {
            *tool_calls += tools.len() as u32;
            for tool in tools {
                // 尝试多种可能的工具名字段
                let name = tool.get("name")
                    .or_else(|| tool.get("toolName"))
                    .or_else(|| tool.get("tool_name"))
                    .or_else(|| tool.get("type"))
                    .or_else(|| tool.get("action"))
                    .and_then(|n| n.as_str());
                if let Some(n) = name {
                    *tool_details.entry(n.to_string()).or_insert(0) += 1;
                }
            }
        }
    }

    // 检查 Agent 模式的特殊字段
    // Cursor Agent 可能使用 capabilityResults, terminalCommands 等
    for key in &["capabilityResults", "terminalCommands", "fileOperations", "codeActions"] {
        if let Some(items) = msg.get(*key).and_then(|t| t.as_array()) {
            *tool_calls += items.len() as u32;
            for item in items {
                let name = item.get("type")
                    .or_else(|| item.get("name"))
                    .or_else(|| item.get("capability"))
                    .and_then(|n| n.as_str())
                    .unwrap_or(*key);
                *tool_details.entry(name.to_string()).or_insert(0) += 1;
            }
        }
    }
}

/// 扫描本地日志文件
#[tauri::command]
pub async fn scan_local_logs(
    window: tauri::Window,
    db: State<'_, Arc<Database>>,
) -> Result<ScanResult, String> {
    let total_steps = 12;
    let mut step = 0u32;

    step += 1;
    emit_local_log_progress(&window, "scan", "claude", step, total_steps, "扫描 Claude Code");
    let (claude_files, claude_entries) = scan_claude_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "codex", step, total_steps, "扫描 Codex CLI");
    let (codex_files, codex_entries) = scan_codex_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "gemini", step, total_steps, "扫描 Gemini CLI");
    let (gemini_files, gemini_entries) = scan_gemini_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "opencode", step, total_steps, "扫描 Opencode");
    let (opencode_files, opencode_entries) = scan_opencode_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "cursor", step, total_steps, "扫描 Cursor");
    let (cursor_files, cursor_entries) = scan_cursor_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "windsurf", step, total_steps, "扫描 Windsurf");
    let (windsurf_files, windsurf_entries) = scan_windsurf_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "kiro", step, total_steps, "扫描 Kiro");
    let (kiro_files, kiro_entries) = scan_kiro_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "antigravity", step, total_steps, "扫描 Antigravity");
    let (antigravity_files, antigravity_entries) = scan_antigravity_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "warp", step, total_steps, "扫描 Warp");
    let (warp_files, warp_entries) = scan_warp_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "augment", step, total_steps, "扫描 Augment");
    let (augment_files, augment_entries) = scan_augment_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "trae", step, total_steps, "扫描 Trae");
    let (trae_files, trae_entries) = scan_trae_logs();

    step += 1;
    emit_local_log_progress(&window, "scan", "openclaw", step, total_steps, "扫描 OpenClaw");
    let (openclaw_files, openclaw_entries) = scan_openclaw_logs();

    let existing_records = {
        let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
        get_existing_local_records(&conn)
    };

    emit_local_log_progress(&window, "scan", "done", total_steps, total_steps, "扫描完成");

    Ok(ScanResult {
        claude_files: claude_files.len() as u32,
        claude_entries,
        claude_path: get_claude_log_dir().map(|p| p.to_string_lossy().to_string()),
        codex_files: codex_files.len() as u32,
        codex_entries,
        codex_path: get_codex_log_dir().map(|p| p.to_string_lossy().to_string()),
        gemini_files: gemini_files.len() as u32,
        gemini_entries,
        gemini_path: get_gemini_log_dir().map(|p| p.to_string_lossy().to_string()),
        opencode_files: opencode_files.len() as u32,
        opencode_entries,
        opencode_path: get_opencode_log_dir().map(|p| p.to_string_lossy().to_string()),
        cursor_files: cursor_files.len() as u32,
        cursor_entries,
        cursor_path: get_cursor_db_path().map(|p| p.to_string_lossy().to_string()),
        windsurf_files: windsurf_files.len() as u32,
        windsurf_entries,
        windsurf_path: get_windsurf_log_path().map(|p| p.to_string_lossy().to_string()),
        kiro_files: kiro_files.len() as u32,
        kiro_entries,
        kiro_path: get_kiro_log_path().map(|p| p.to_string_lossy().to_string()),
        antigravity_files: antigravity_files.len() as u32,
        antigravity_entries,
        antigravity_path: get_antigravity_log_path().map(|p| p.to_string_lossy().to_string()),
        warp_files: warp_files.len() as u32,
        warp_entries,
        warp_path: get_warp_db_path().map(|p| p.to_string_lossy().to_string()),
        augment_files: augment_files.len() as u32,
        augment_entries,
        augment_path: get_augment_log_path().map(|p| p.to_string_lossy().to_string()),
        github_copilot_files: 0,
        github_copilot_entries: 0,
        github_copilot_path: None,
        codebuddy_files: 0,
        codebuddy_entries: 0,
        codebuddy_path: None,
        codebuddy_cn_files: 0,
        codebuddy_cn_entries: 0,
        codebuddy_cn_path: None,
        qoder_files: 0,
        qoder_entries: 0,
        qoder_path: None,
        trae_files: trae_files.len() as u32,
        trae_entries,
        trae_path: get_trae_log_path().map(|p| p.to_string_lossy().to_string()),
        workbuddy_files: 0,
        workbuddy_entries: 0,
        workbuddy_path: None,
        openclaw_files: openclaw_files.len() as u32,
        openclaw_entries,
        openclaw_path: get_openclaw_base_path().map(|p| p.to_string_lossy().to_string()),
        existing_records,
    })
}

/// 导入本地日志
#[tauri::command]
pub async fn import_local_logs(
    window: tauri::Window,
    sources: Vec<String>,
    db: State<'_, Arc<Database>>,
) -> Result<LocalLogImportResult, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;
    let mut total = 0u32;
    
    // 用于去重的集合
    let mut seen_ids: HashSet<String> = HashSet::new();
    let total_sources = sources.len() as u32;
    let mut source_index = 0u32;
    
    // 导入 Claude Code 日志
    if sources.contains(&"claude".to_string()) {
        source_index += 1;
        let (files, _) = scan_claude_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(
            &window,
            "import",
            "claude",
            0,
            total_files,
            &format!("导入 Claude Code ({}/{})", source_index, total_sources),
        );
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "claude_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_claude_log_file(file);
            let file_index = idx as u32 + 1;
            for entry in entries {
                total += 1;
                
                // 检查是否已处理过
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                // 检查数据库中是否已存在
                if existing_ids.contains(&entry.session_id) {
                    let cost = resolve_entry_cost(&conn, &entry);
                    match update_log_entry(&conn, &entry, cost) {
                        Ok(_) => imported += 1,
                        Err(_) => failed += 1,
                    }
                    continue;
                }
                existing_ids.insert(entry.session_id.clone());
                
                // 计算成本（优先使用服务商特定定价）
                let cost = resolve_entry_cost(&conn, &entry);
                
                // 插入数据库
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }

            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(
                    &window,
                    "import",
                    "claude",
                    file_index,
                    total_files,
                    &format!("导入 Claude Code ({}/{})", source_index, total_sources),
                );
            }
            
            // 解析并保存会话统计信息
            let stats = parse_claude_session_stats(file);
            let session_id = file
                .file_stem()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let provider_id = "claude_local".to_string();
            let _ = save_session_stats(&conn, &session_id, "claude", Some(&provider_id), &stats);
        }
        let _ = conn.execute_batch("COMMIT");
    }
    
    // 导入 Codex CLI 日志
    if sources.contains(&"codex".to_string()) {
        source_index += 1;
        let (files, _) = scan_codex_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(
            &window,
            "import",
            "codex",
            0,
            total_files,
            &format!("导入 Codex CLI ({}/{})", source_index, total_sources),
        );
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "codex_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_codex_log_file(file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if existing_ids.contains(&entry.session_id) {
                    let cost = resolve_entry_cost(&conn, &entry);
                    match update_log_entry(&conn, &entry, cost) {
                        Ok(_) => imported += 1,
                        Err(_) => failed += 1,
                    }
                    continue;
                }
                existing_ids.insert(entry.session_id.clone());
                
                // 计算成本（优先使用服务商特定定价）
                let cost = resolve_entry_cost(&conn, &entry);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }

            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(
                    &window,
                    "import",
                    "codex",
                    file_index,
                    total_files,
                    &format!("导入 Codex CLI ({}/{})", source_index, total_sources),
                );
            }
            
            // 解析并保存会话统计信息
            let stats = parse_codex_session_stats(file);
            if stats.conversation_count > 0 || stats.tool_calls.values().sum::<u32>() > 0 {
                let session_id = file
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let provider_id = "codex_local".to_string();
                let _ = save_session_stats(&conn, &session_id, "codex", Some(&provider_id), &stats);
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }
    
    // 导入 Gemini CLI 日志
    if sources.contains(&"gemini".to_string()) {
        source_index += 1;
        let (files, _) = scan_gemini_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(
            &window,
            "import",
            "gemini",
            0,
            total_files,
            &format!("导入 Gemini CLI ({}/{})", source_index, total_sources),
        );
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "gemini_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_gemini_log_file(file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if existing_ids.contains(&entry.session_id) {
                    let cost = resolve_entry_cost(&conn, &entry);
                    match update_log_entry(&conn, &entry, cost) {
                        Ok(_) => imported += 1,
                        Err(_) => failed += 1,
                    }
                    continue;
                }
                existing_ids.insert(entry.session_id.clone());
                
                // 计算成本（优先使用服务商特定定价）
                let cost = resolve_entry_cost(&conn, &entry);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }

            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(
                    &window,
                    "import",
                    "gemini",
                    file_index,
                    total_files,
                    &format!("导入 Gemini CLI ({}/{})", source_index, total_sources),
                );
            }
            
            // 解析并保存会话统计信息
            let stats = parse_gemini_session_stats(file);
            if stats.conversation_count > 0 || stats.tool_calls.values().sum::<u32>() > 0 {
                let session_id = file
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let provider_id = "gemini_local".to_string();
                let _ = save_session_stats(&conn, &session_id, "gemini", Some(&provider_id), &stats);
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }
    
    // 导入 Opencode 日志
    if sources.contains(&"opencode".to_string()) {
        source_index += 1;
        let (files, _) = scan_opencode_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(
            &window,
            "import",
            "opencode",
            0,
            total_files,
            &format!("导入 Opencode ({}/{})", source_index, total_sources),
        );
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "opencode_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_opencode_log_file(file);
            for entry in entries {
                total += 1;
                
                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                if existing_ids.contains(&entry.session_id) {
                    let cost = resolve_entry_cost(&conn, &entry);
                    match update_log_entry(&conn, &entry, cost) {
                        Ok(_) => imported += 1,
                        Err(_) => failed += 1,
                    }
                    continue;
                }
                existing_ids.insert(entry.session_id.clone());
                
                // 计算成本（优先使用服务商特定定价）
                let cost = resolve_entry_cost(&conn, &entry);
                
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }

            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(
                    &window,
                    "import",
                    "opencode",
                    file_index,
                    total_files,
                    &format!("导入 Opencode ({}/{})", source_index, total_sources),
                );
            }
            
            // 解析并保存会话统计信息
            let stats = parse_opencode_session_stats(file);
            if stats.conversation_count > 0 || stats.tool_calls.values().sum::<u32>() > 0 {
                let session_id = file
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let provider_id = "opencode_local".to_string();
                let _ = save_session_stats(&conn, &session_id, "opencode", Some(&provider_id), &stats);
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }
    
    // 导入 Cursor 日志
    if sources.contains(&"cursor".to_string()) {
        source_index += 1;
        let (files, _) = scan_cursor_logs();
        let total_units = files.len() as u32;
        emit_local_log_progress(
            &window,
            "import",
            "cursor",
            0,
            total_units,
            &format!("导入 Cursor ({}/{})", source_index, total_sources),
        );
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        if let Err(err) = delete_log_entries_by_request_id_prefix(&conn, "cursor-official-") {
            logger::log_warn(&format!(
                "[Local Logs] 清理历史 Cursor 官方导入记录失败: {}",
                err
            ));
        }

        let mut existing_cursor_ids = load_existing_request_ids_by_prefix(&conn, "cursor-");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_cursor_db(file);
            for entry in entries {
                total += 1;

                if seen_ids.contains(&entry.session_id) {
                    skipped += 1;
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());

                if existing_cursor_ids.contains(&entry.session_id) {
                    // 已存在则更新（用于重新导入刷新统计）
                    let cost = resolve_entry_cost(&conn, &entry);
                    match update_log_entry(&conn, &entry, cost) {
                        Ok(_) => imported += 1,
                        Err(_) => failed += 1,
                    }
                    continue;
                }
                existing_cursor_ids.insert(entry.session_id.clone());

                let cost = resolve_entry_cost(&conn, &entry);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }

            let file_index = idx as u32 + 1;
            if total_units > 0 && (file_index == total_units || file_index % 5 == 0) {
                emit_local_log_progress(
                    &window,
                    "import",
                    "cursor",
                    file_index,
                    total_units,
                    &format!("导入 Cursor ({}/{})", source_index, total_sources),
                );
            }
        }

        for file in &files {
            let stats = parse_cursor_session_stats(file);
            if stats.conversation_count > 0 || stats.tool_calls.values().sum::<u32>() > 0 {
                let session_id = file.to_string_lossy().replace(['\\', '/', ':'], "_");
                let provider_id = "cursor_local".to_string();
                let _ = save_session_stats(&conn, &session_id, "cursor", Some(&provider_id), &stats);
            }
        }

        // 补齐旧记录中缓存 token 为 0 的条目（Cursor 本地不存储缓存 token）
        let patched = conn.execute(
            "UPDATE proxy_request_logs
             SET cache_read_tokens = CAST(input_tokens * 62.6 AS INTEGER)
             WHERE app_type = 'cursor_local'
               AND cache_read_tokens = 0
               AND input_tokens > 0",
            [],
        ).unwrap_or(0);
        if patched > 0 {
            logger::log_info(&format!(
                "[Local Logs] 已为 {} 条 Cursor 旧记录补齐缓存 Token 估算",
                patched
            ));
        }

        // 逗号分隔的模型名只取第一个
        let _ = conn.execute(
            "UPDATE proxy_request_logs
             SET model = SUBSTR(model, 1, INSTR(model, ',') - 1)
             WHERE app_type = 'cursor_local'
               AND model LIKE '%,%'",
            [],
        );

        // 重算费用：补齐缓存 token 后需要更新 total_cost_usd
        if patched > 0 {
            patch_cursor_costs_after_cache_fix(&conn);
        }

        let _ = conn.execute_batch("COMMIT");
    }
    
    // 导入 Windsurf 日志
    if sources.contains(&"windsurf".to_string()) {
        source_index += 1;
        let (files, _) = scan_windsurf_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "windsurf", 0, total_files, &format!("导入 Windsurf ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "windsurf_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_vscode_app_db(file, "windsurf");
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "windsurf", file_index, total_files, &format!("导入 Windsurf ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    // 导入 Kiro 日志
    if sources.contains(&"kiro".to_string()) {
        source_index += 1;
        let (files, _) = scan_kiro_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "kiro", 0, total_files, &format!("导入 Kiro ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "kiro_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_vscode_app_db(file, "kiro");
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "kiro", file_index, total_files, &format!("导入 Kiro ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    // 导入 Antigravity 日志
    if sources.contains(&"antigravity".to_string()) {
        source_index += 1;
        let (files, _) = scan_antigravity_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "antigravity", 0, total_files, &format!("导入 Antigravity ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "antigravity_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_vscode_app_db(file, "antigravity");
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "antigravity", file_index, total_files, &format!("导入 Antigravity ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    // 导入 Warp 日志
    if sources.contains(&"warp".to_string()) {
        source_index += 1;
        let (files, _) = scan_warp_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "warp", 0, total_files, &format!("导入 Warp ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "warp_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for file in &files {
            let entries = parse_warp_db(file);
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = entry.cost_usd
                    .map(|c| Decimal::from_str(&c.to_string()).unwrap_or(Decimal::ZERO))
                    .unwrap_or_else(|| calculate_cost(&entry, pricing));
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
        }
        let _ = conn.execute_batch("COMMIT");
        emit_local_log_progress(&window, "import", "warp", total_files, total_files, &format!("导入 Warp ({}/{})", source_index, total_sources));
    }

    // 导入 Augment 日志
    if sources.contains(&"augment".to_string()) {
        source_index += 1;
        let (files, _) = scan_augment_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "augment", 0, total_files, &format!("导入 Augment ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "augment_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_vscode_app_db(file, "augment");
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "augment", file_index, total_files, &format!("导入 Augment ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    // 导入 Trae 日志
    if sources.contains(&"trae".to_string()) {
        source_index += 1;
        let (files, _) = scan_trae_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "trae", 0, total_files, &format!("导入 Trae ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "trae_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_vscode_app_db(file, "trae");
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let provider_id = format!("{}_local", entry.source);
                let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                let cost = calculate_cost(&entry, pricing);
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "trae", file_index, total_files, &format!("导入 Trae ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    // 导入 OpenClaw 日志
    if sources.contains(&"openclaw".to_string()) {
        source_index += 1;
        let (files, _) = scan_openclaw_logs();
        let total_files = files.len() as u32;
        emit_local_log_progress(&window, "import", "openclaw", 0, total_files, &format!("导入 OpenClaw ({}/{})", source_index, total_sources));
        let mut existing_ids = load_existing_request_ids_by_app_type(&conn, "openclaw_local");
        let _ = conn.execute_batch("BEGIN IMMEDIATE");
        for (idx, file) in files.iter().enumerate() {
            let entries = parse_openclaw_session_file(file);
            for entry in entries {
                total += 1;
                if seen_ids.contains(&entry.session_id) { skipped += 1; continue; }
                seen_ids.insert(entry.session_id.clone());
                if existing_ids.contains(&entry.session_id) { skipped += 1; continue; }
                existing_ids.insert(entry.session_id.clone());
                let cost = entry.cost_usd
                    .map(|c| Decimal::from_str(&c.to_string()).unwrap_or(Decimal::ZERO))
                    .unwrap_or_else(|| {
                        let provider_id = format!("{}_local", entry.source);
                        let pricing = get_provider_model_pricing(&conn, &provider_id, &entry.model);
                        calculate_cost(&entry, pricing)
                    });
                match insert_log_entry(&conn, &entry, cost) {
                    Ok(_) => imported += 1,
                    Err(_) => failed += 1,
                }
            }
            let file_index = idx as u32 + 1;
            if total_files > 0 && (file_index == total_files || file_index % 5 == 0) {
                emit_local_log_progress(&window, "import", "openclaw", file_index, total_files, &format!("导入 OpenClaw ({}/{})", source_index, total_sources));
            }
        }
        let _ = conn.execute_batch("COMMIT");
    }

    emit_local_log_progress(&window, "import", "done", total_sources, total_sources, "导入完成");

    Ok(LocalLogImportResult {
        imported,
        skipped,
        failed,
        total,
    })
}

/// 清除本地导入的日志
#[tauri::command]
pub async fn clear_local_logs(db: State<'_, Arc<Database>>) -> Result<u32, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let deleted = conn
        .execute("DELETE FROM proxy_request_logs WHERE app_type LIKE '%_local'", [])
        .map_err(|e| format!("清除本地日志失败: {e}"))?;
    
    Ok(deleted as u32)
}

/// 自动导入本地日志（静默模式，用于后台自动导入）
/// 返回新导入的记录数
#[tauri::command]
pub async fn auto_import_local_logs(db: State<'_, Arc<Database>>) -> Result<u32, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut imported = 0u32;
    
    // 用于去重的集合
    let mut seen_ids: HashSet<String> = HashSet::new();
    
    // 自动导入所有来源的日志
    let sources = vec!["claude", "codex", "gemini", "opencode", "cursor", "windsurf", "kiro", "antigravity", "warp", "augment", "trae", "openclaw"];
    
    for source in sources {
        let (files, _) = match source {
            "claude" => scan_claude_logs(),
            "codex" => scan_codex_logs(),
            "gemini" => scan_gemini_logs(),
            "opencode" => scan_opencode_logs(),
            "cursor" => scan_cursor_logs(),
            "windsurf" => scan_windsurf_logs(),
            "kiro" => scan_kiro_logs(),
            "antigravity" => scan_antigravity_logs(),
            "warp" => scan_warp_logs(),
            "augment" => scan_augment_logs(),
            "trae" => scan_trae_logs(),
            "openclaw" => scan_openclaw_logs(),
            _ => continue,
        };

        if source == "cursor" {
            if let Err(err) = delete_log_entries_by_request_id_prefix(&conn, "cursor-official-") {
                logger::log_warn(&format!(
                    "[Local Logs] 清理历史 Cursor 官方导入记录失败: {}",
                    err
                ));
            }
        }

        let mut existing_ids = load_existing_request_ids_for_source(&conn, source);
        
        for file in &files {
            let entries: Vec<LocalLogEntry> = match source {
                "claude" => parse_claude_log_file(file),
                "codex" => parse_codex_log_file(file),
                "gemini" => parse_gemini_log_file(file),
                "opencode" => parse_opencode_log_file(file),
                "cursor" => parse_cursor_db(file),
                "windsurf" => parse_vscode_app_db(file, "windsurf"),
                "kiro" => parse_vscode_app_db(file, "kiro"),
                "antigravity" => parse_vscode_app_db(file, "antigravity"),
                "warp" => parse_warp_db(file),
                "augment" => parse_vscode_app_db(file, "augment"),
                "trae" => parse_vscode_app_db(file, "trae"),
                "openclaw" => parse_openclaw_session_file(file),
                _ => continue,
            };
            
            for entry in entries {
                // 检查是否已处理过
                if seen_ids.contains(&entry.session_id) {
                    continue;
                }
                seen_ids.insert(entry.session_id.clone());
                
                // 检查数据库中是否已存在
                if existing_ids.contains(&entry.session_id) {
                    let cost = resolve_entry_cost(&conn, &entry);
                    if update_log_entry(&conn, &entry, cost).is_ok() {
                        imported += 1;
                    }
                    continue;
                }
                existing_ids.insert(entry.session_id.clone());
                
                // 计算成本（优先使用服务商特定定价）
                let cost = resolve_entry_cost(&conn, &entry);
                
                // 插入数据库
                if insert_log_entry(&conn, &entry, cost).is_ok() {
                    imported += 1;
                }
            }
        }
        
        // 解析并保存会话统计信息
        for file in &files {
            let stats = match source {
                "claude" => parse_claude_session_stats(file),
                "codex" => parse_codex_session_stats(file),
                "gemini" => parse_gemini_session_stats(file),
                "opencode" => parse_opencode_session_stats(file),
                "cursor" | "windsurf" | "kiro" | "antigravity" | "augment" | "trae" => parse_cursor_session_stats(file),
                "openclaw" => parse_openclaw_session_stats(file),
                _ => continue,
            };
            
            // 只有有数据时才保存
            if stats.conversation_count > 0 || stats.tool_calls.values().sum::<u32>() > 0 {
                let session_id = build_local_stats_session_id(source, file);
                let provider_id = format!("{}_local", source);
                let _ = save_session_stats(&conn, &session_id, source, Some(&provider_id), &stats);
            }
        }
    }
    
    Ok(imported)
}

// ============================================================================
// 会话统计解析
// ============================================================================

/// 解析 Claude Code 日志文件的会话统计信息
fn parse_claude_session_stats(path: &PathBuf) -> SessionStats {
    let mut stats = SessionStats::default();
    
    let Ok(content) = fs::read_to_string(path) else {
        return stats;
    };
    
    let mut last_user_timestamp_ms: Option<i64> = None;
    let mut first_assistant_after_user = true;
    let mut files_modified: HashSet<String> = HashSet::new();
    
    for line in content.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let msg_type = json.get("type").and_then(|v| v.as_str());
            
            match msg_type {
                Some("user") => {
                    // 用户消息
                    // 检查是否是工具结果（嵌套在 user 消息中）
                    if let Some(message) = json.get("message") {
                        if let Some(content_arr) = message.get("content").and_then(|c| c.as_array()) {
                            let has_tool_result = content_arr.iter().any(|item| {
                                item.get("type").and_then(|t| t.as_str()) == Some("tool_result")
                            });
                            
                            if !has_tool_result {
                                // 普通用户消息，计算对话轮数
                                stats.conversation_count += 1;
                                last_user_timestamp_ms = extract_timestamp_ms(&json);
                                first_assistant_after_user = true;
                            }
                        } else if message.get("role").and_then(|r| r.as_str()) == Some("user") {
                            // 旧格式用户消息
                            stats.conversation_count += 1;
                            last_user_timestamp_ms = extract_timestamp_ms(&json);
                            first_assistant_after_user = true;
                        }
                    }
                }
                Some("assistant") => {
                    // 助手消息，解析 content 数组
                    // 只计算用户消息后第一个助手消息的响应时间
                    if first_assistant_after_user {
                        if let (Some(user_ts_ms), Some(assistant_ts_ms)) = 
                            (last_user_timestamp_ms, extract_timestamp_ms(&json)) 
                        {
                            let response_ms = (assistant_ts_ms - user_ts_ms).abs() as u64;
                            // 响应时间应该在合理范围内（小于5分钟）
                            if response_ms < 300000 {
                                stats.response_time_ms += response_ms;
                            }
                        }
                        first_assistant_after_user = false;
                    }
                    
                    if let Some(message) = json.get("message") {
                        if let Some(content_arr) = message.get("content").and_then(|c| c.as_array()) {
                            for block in content_arr {
                                let block_type = block.get("type").and_then(|t| t.as_str());
                                
                                match block_type {
                                    Some("thinking") => {
                                        // 思考内容
                                        if let Some(thinking_text) = block.get("thinking").and_then(|t| t.as_str()) {
                                            // 估算思考时间：每1000字符约2秒
                                            let thinking_ms = (thinking_text.len() as u64).saturating_mul(2);
                                            stats.thinking_time_ms += thinking_ms;
                                        }
                                    }
                                    Some("tool_use") => {
                                        // 工具调用（在 assistant 消息的 content 中）
                                        if let Some(tool_name) = block.get("name").and_then(|n| n.as_str()) {
                                            let normalized_name = normalize_tool_name(tool_name);
                                            *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                                            
                                            // 检查是否是文件编辑工具
                                            if is_file_edit_tool(tool_name) {
                                                if let Some(file_path) = extract_file_path_from_tool(block) {
                                                    files_modified.insert(file_path);
                                                }
                                                
                                                // 从工具参数中提取代码变更
                                                if let Some(input) = block.get("input") {
                                                    let (added, deleted) = extract_code_changes_from_input(input, tool_name);
                                                    stats.lines_added += added;
                                                    stats.lines_deleted += deleted;
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
            
            // 从工具结果中提取代码变更
            if let Some(tool_result) = json.get("toolUseResult") {
                if let Some(stdout) = tool_result.get("stdout").and_then(|s| s.as_str()) {
                    let (added, deleted) = extract_line_changes(stdout);
                    stats.lines_added += added;
                    stats.lines_deleted += deleted;
                }
            }
        }
    }
    
    stats.files_changed = files_modified.len() as u32;
    stats
}

/// 提取时间戳（毫秒）
fn extract_timestamp_ms(json: &serde_json::Value) -> Option<i64> {
    json.get("timestamp")
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                // 判断是秒还是毫秒
                if ts > 1_000_000_000_000 {
                    Some(ts) // 已经是毫秒
                } else {
                    Some(ts * 1000) // 转换为毫秒
                }
            } else if let Some(s) = v.as_str() {
                // ISO 8601 格式
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp_millis())
            } else {
                None
            }
        })
}

/// 解析 Codex CLI 日志文件的会话统计信息
fn parse_codex_session_stats(path: &PathBuf) -> SessionStats {
    let mut stats = SessionStats::default();
    
    let Ok(content) = fs::read_to_string(path) else {
        return stats;
    };
    
    let mut files_modified: HashSet<String> = HashSet::new();
    let mut _last_user_timestamp: Option<i64> = None;
    
    for line in content.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            let event_type = json.get("type").and_then(|v| v.as_str());
            
            match event_type {
                Some("event_msg") => {
                    // 用户消息事件
                    if let Some(payload) = json.get("payload") {
                        if payload.get("type").and_then(|t| t.as_str()) == Some("user_message") {
                            stats.conversation_count += 1;
                            _last_user_timestamp = extract_timestamp(&json);
                        }
                    }
                }
                Some("response_item") => {
                    // 响应项
                    if let Some(payload) = json.get("payload") {
                        let payload_type = payload.get("type").and_then(|t| t.as_str());
                        
                        match payload_type {
                            Some("message") => {
                                // 检查 role
                                let role = payload.get("role").and_then(|r| r.as_str());
                                if role == Some("user") {
                                    // 检查是否是工具结果
                                    let is_tool_result = payload.get("content")
                                        .and_then(|c| c.as_array())
                                        .map(|arr| arr.iter().any(|item| {
                                            item.get("type").and_then(|t| t.as_str()) == Some("tool_result")
                                        }))
                                        .unwrap_or(false);
                                    
                                    if !is_tool_result {
                                        stats.conversation_count += 1;
                                        _last_user_timestamp = extract_timestamp(&json);
                                    }
                                }
                            }
                            Some("function_call") => {
                                // 工具调用
                                if let Some(tool_name) = payload.get("name").and_then(|n| n.as_str()) {
                                    let normalized_name = normalize_tool_name(tool_name);
                                    *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                                    
                                    if is_file_edit_tool(tool_name) {
                                        // 尝试从 arguments 中提取文件路径
                                        if let Some(args_str) = payload.get("arguments").and_then(|a| a.as_str()) {
                                            if let Ok(args) = serde_json::from_str::<serde_json::Value>(args_str) {
                                                if let Some(file_path) = extract_file_path_from_tool(&args) {
                                                    files_modified.insert(file_path);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Some("reasoning") => {
                                // 思考内容
                                if let Some(summary) = payload.get("summary").and_then(|s| s.as_array()) {
                                    for item in summary {
                                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                            // 估算思考时间
                                            let thinking_ms = (text.len() as u64).saturating_mul(2);
                                            stats.thinking_time_ms += thinking_ms;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    stats.files_changed = files_modified.len() as u32;
    stats
}

/// 解析 Gemini CLI 日志文件的会话统计信息
fn parse_gemini_session_stats(path: &PathBuf) -> SessionStats {
    let mut stats = SessionStats::default();
    
    let Ok(content) = fs::read_to_string(path) else {
        return stats;
    };
    
    let mut files_modified: HashSet<String> = HashSet::new();
    
    // 尝试解析为 JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        // Gemini 日志可能是消息数组或单个对象
        let messages = json.get("messages")
            .and_then(|m| m.as_array())
            .cloned()
            .or_else(|| json.as_array().cloned());
        
        if let Some(msgs) = messages {
            for msg in msgs {
                let role = msg.get("role").and_then(|r| r.as_str());
                
                if role == Some("user") {
                    stats.conversation_count += 1;
                }
                
                // 检查工具调用
                if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        if let Some(function_call) = part.get("functionCall") {
                            if let Some(tool_name) = function_call.get("name").and_then(|n| n.as_str()) {
                                let normalized_name = normalize_tool_name(tool_name);
                                *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                                
                                if is_file_edit_tool(tool_name) {
                                    if let Some(args) = function_call.get("args") {
                                        if let Some(file_path) = extract_file_path_from_tool(args) {
                                            files_modified.insert(file_path);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // 检查工具使用 (tool_use)
                if let Some(tool_calls) = msg.get("toolCalls").and_then(|t| t.as_array()) {
                    for tc in tool_calls {
                        if let Some(tool_name) = tc.get("name").and_then(|n| n.as_str()) {
                            let normalized_name = normalize_tool_name(tool_name);
                            *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    } else {
        // 尝试按行解析 JSONL
        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let role = json.get("role").and_then(|r| r.as_str());
                
                if role == Some("user") {
                    stats.conversation_count += 1;
                }
                
                // 检查工具调用
                if let Some(tool_name) = json.get("toolName")
                    .or_else(|| json.get("name"))
                    .and_then(|n| n.as_str())
                {
                    let normalized_name = normalize_tool_name(tool_name);
                    *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                }
            }
        }
    }
    
    stats.files_changed = files_modified.len() as u32;
    stats
}

/// 解析 Opencode 日志文件的会话统计信息
/// Opencode 使用分离的 JSON 文件存储：message/{sessionID}/{messageID}.json 和 part/{messageID}/{partID}.json
fn parse_opencode_session_stats(path: &PathBuf) -> SessionStats {
    let mut stats = SessionStats::default();
    
    let Ok(content) = fs::read_to_string(path) else {
        return stats;
    };
    
    // opencode 消息是单个 JSON 文件，不是 JSONL
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return stats;
    };
    
    let role = json.get("role").and_then(|r| r.as_str());
    
    // 用户消息
    if role == Some("user") {
        stats.conversation_count = 1;
    }
    
    // 助手消息
    if role == Some("assistant") {
        // 提取时间信息计算响应时间
        if let Some(time) = json.get("time") {
            let created = time.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
            let completed = time.get("completed").and_then(|v| v.as_i64()).unwrap_or(0);
            
            if completed > created && created > 0 {
                let response_ms = (completed - created) as u64;
                // 响应时间应该合理（小于1小时）
                if response_ms < 3600000 {
                    stats.response_time_ms = response_ms;
                }
            }
        }
        
        // 从助手消息中读取 tokens
        if let Some(tokens) = json.get("tokens") {
            let reasoning = tokens.get("reasoning").and_then(|v| v.as_u64()).unwrap_or(0);
            // 估算思考时间：每1000 reasoning tokens 约 2 秒
            stats.thinking_time_ms = (reasoning as u64).saturating_mul(2);
        }
    }
    
    // 从 part 目录读取工具调用
    // part 文件路径: storage/part/{messageID}/{partID}.json
    let message_id = json.get("id").and_then(|v| v.as_str());
    if let Some(msg_id) = message_id {
        if let Some(storage_dir) = path.parent().and_then(|p| p.parent()) {
            let part_dir = storage_dir.join("part").join(msg_id);
            if part_dir.exists() {
                let mut files_modified: HashSet<String> = HashSet::new();
                
                if let Ok(parts) = fs::read_dir(&part_dir) {
                    for part_entry in parts.flatten() {
                        let part_path = part_entry.path();
                        if part_path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(part_content) = fs::read_to_string(&part_path) {
                                if let Ok(part_json) = serde_json::from_str::<serde_json::Value>(&part_content) {
                                    let part_type = part_json.get("type").and_then(|t| t.as_str());
                                    
                                    // 工具调用 part
                                    if part_type == Some("tool") {
                                        if let Some(tool_name) = part_json.get("tool").and_then(|t| t.as_str()) {
                                            let normalized_name = normalize_tool_name(tool_name);
                                            *stats.tool_calls.entry(normalized_name).or_insert(0) += 1;
                                            
                                            // 提取文件变更
                                            if is_file_edit_tool(tool_name) {
                                                if let Some(state) = part_json.get("state") {
                                                    if let Some(input) = state.get("input") {
                                                        if let Some(file_path) = input.get("path").and_then(|p| p.as_str()) {
                                                            files_modified.insert(file_path.to_string());
                                                        }
                                                        
                                                        // 提取代码变更
                                                        let (added, deleted) = extract_code_changes_from_input(input, tool_name);
                                                        stats.lines_added += added;
                                                        stats.lines_deleted += deleted;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // 推理 part
                                    if part_type == Some("reasoning") {
                                        if let Some(text) = part_json.get("text").and_then(|t| t.as_str()) {
                                            // 估算思考时间：每1000字符约2秒
                                            let thinking_ms = (text.len() as u64).saturating_mul(2);
                                            stats.thinking_time_ms += thinking_ms;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                stats.files_changed = files_modified.len() as u32;
            }
        }
    }
    
    stats
}

/// 解析 Cursor 数据库的会话统计信息
fn parse_cursor_session_stats(path: &PathBuf) -> SessionStats {
    use rusqlite::{Connection, OpenFlags};
    
    let mut stats = SessionStats::default();
    let mut files_modified: HashSet<String> = HashSet::new();
    
    // 以只读模式打开数据库
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return stats;
    };
    
    // 查询包含 AI 对话数据的记录
    let mut stmt = match conn.prepare(
        "SELECT key, value FROM ItemTable WHERE key LIKE '%composerData%' OR key LIKE '%aiService%' OR key LIKE '%chat%'"
    ) {
        Ok(stmt) => stmt,
        Err(_) => return stats,
    };
    
    let rows = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Vec<u8>>(1)?,
        ))
    }) {
        Ok(rows) => rows,
        Err(_) => return stats,
    };
    
    for row_result in rows {
        if let Ok((_key, value_bytes)) = row_result {
            // 尝试解析 JSON 数据
            if let Ok(value_str) = String::from_utf8(value_bytes) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&value_str) {
                    // 解析对话数据
                    parse_cursor_stats_from_json(&json, &mut stats, &mut files_modified);
                }
            }
        }
    }
    
    stats.files_changed = files_modified.len() as u32;
    stats
}

/// 从 Cursor JSON 数据中提取统计信息
fn parse_cursor_stats_from_json(
    json: &serde_json::Value,
    stats: &mut SessionStats,
    files_modified: &mut HashSet<String>,
) {
    // 检查是否有消息数组
    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str());
            
            // 用户消息计数
            if role == Some("user") {
                stats.conversation_count += 1;
            }
            
            // 检查助手消息中的工具调用
            if role == Some("assistant") {
                // 检查工具调用
                if let Some(tool_calls) = msg.get("tool_calls").and_then(|t| t.as_array()) {
                    for tc in tool_calls {
                        if let Some(function) = tc.get("function") {
                            if let Some(tool_name) = function.get("name").and_then(|n| n.as_str()) {
                                let normalized = normalize_tool_name(tool_name);
                                *stats.tool_calls.entry(normalized).or_insert(0) += 1;
                                
                                // 检查文件编辑
                                if is_file_edit_tool(tool_name) {
                                    if let Some(args) = function.get("arguments") {
                                        if let Some(args_str) = args.as_str() {
                                            if let Ok(args_json) = serde_json::from_str::<serde_json::Value>(args_str) {
                                                if let Some(file_path) = args_json.get("path").and_then(|p| p.as_str()) {
                                                    files_modified.insert(file_path.to_string());
                                                }
                                                // 提取代码变更
                                                let (added, deleted) = extract_code_changes_from_input(&args_json, tool_name);
                                                stats.lines_added += added;
                                                stats.lines_deleted += deleted;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 检查对话/会话数组
    if let Some(conversations) = json.get("conversations").and_then(|c| c.as_array()) {
        for conv in conversations {
            if let Some(msgs) = conv.get("messages").and_then(|m| m.as_array()) {
                for msg in msgs {
                    if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                        stats.conversation_count += 1;
                    }
                }
            }
        }
    }
    
    // 检查工具使用数组
    if let Some(tools) = json.get("tools").and_then(|t| t.as_array()) {
        for tool in tools {
            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                let normalized = normalize_tool_name(name);
                *stats.tool_calls.entry(normalized).or_insert(0) += 1;
            }
        }
    }
}

/// 提取时间戳
fn extract_timestamp(json: &serde_json::Value) -> Option<i64> {
    json.get("timestamp")
        .and_then(|v| {
            if let Some(ts) = v.as_i64() {
                if ts > 1_000_000_000_000 {
                    Some(ts / 1000)
                } else {
                    Some(ts)
                }
            } else if let Some(s) = v.as_str() {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|dt| dt.timestamp())
            } else {
                None
            }
        })
}

/// 标准化工具名称
fn normalize_tool_name(name: &str) -> String {
    let name_lower = name.to_lowercase();
    
    // 处理 MCP 工具名称（如 mcp__pencil_xxx -> Pencil, mcp__context7_xxx -> Context7）
    if name_lower.starts_with("mcp__") {
        // mcp__pencil_batch_get -> pencil
        let without_prefix = &name[5..]; // 去掉 "mcp__"
        if let Some(underscore_pos) = without_prefix.find('_') {
            let tool_name = &without_prefix[..underscore_pos];
            // 首字母大写
            let mut chars = tool_name.chars();
            return match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => tool_name.to_string(),
            };
        } else {
            // 没有下划线，直接使用
            let mut chars = without_prefix.chars();
            return match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => without_prefix.to_string(),
            };
        }
    }
    
    // 处理 mcp_ 前缀（单下划线）
    if name_lower.starts_with("mcp_") {
        let without_prefix = &name[4..]; // 去掉 "mcp_"
        if let Some(underscore_pos) = without_prefix.find('_') {
            let tool_name = &without_prefix[..underscore_pos];
            let mut chars = tool_name.chars();
            return match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => tool_name.to_string(),
            };
        } else {
            let mut chars = without_prefix.chars();
            return match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => without_prefix.to_string(),
            };
        }
    }
    
    match name_lower.as_str() {
        "read" | "read_file" | "readfile" => "Read".to_string(),
        "write" | "write_file" | "writefile" | "create_file" => "Write".to_string(),
        "str_replace" | "strreplace" | "replace" | "edit" | "edit_file" => "StrReplace".to_string(),
        "grep" | "search" | "find" => "Grep".to_string(),
        "glob" | "list_files" | "ls" => "Glob".to_string(),
        "shell" | "bash" | "execute" | "run" | "terminal" | "shell_command" => "Shell".to_string(),
        "task" | "agent" | "subagent" => "Task".to_string(),
        "web_search" | "websearch" | "search_web" => "WebSearch".to_string(),
        "web_fetch" | "webfetch" | "fetch" => "WebFetch".to_string(),
        _ => name.to_string(),
    }
}

/// 检查是否是文件编辑工具
fn is_file_edit_tool(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    matches!(
        name_lower.as_str(),
        "write" | "write_file" | "writefile" | "create_file" |
        "str_replace" | "strreplace" | "replace" | "edit" | "edit_file"
    )
}

/// 从工具参数中提取文件路径
fn extract_file_path_from_tool(json: &serde_json::Value) -> Option<String> {
    // 尝试从不同位置获取文件路径
    json.get("input")
        .or_else(|| json.get("parameters"))
        .or_else(|| json.get("args"))
        .and_then(|input| {
            input.get("path")
                .or_else(|| input.get("file_path"))
                .or_else(|| input.get("filePath"))
                .or_else(|| input.get("file"))
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        })
}

/// 从工具结果中提取行变更信息
fn extract_line_changes(result: &str) -> (u32, u32) {
    let mut added = 0u32;
    let mut deleted = 0u32;
    
    // 检查常见的行变更模式
    for line in result.lines() {
        let trimmed = line.trim();
        
        // 统计以 + 开头的行（新增）
        if trimmed.starts_with('+') && !trimmed.starts_with("+++") {
            added += 1;
        }
        // 统计以 - 开头的行（删除）
        else if trimmed.starts_with('-') && !trimmed.starts_with("---") {
            deleted += 1;
        }
    }
    
    (added, deleted)
}

/// 从工具输入参数中提取代码变更
fn extract_code_changes_from_input(input: &serde_json::Value, tool_name: &str) -> (u32, u32) {
    let tool_lower = tool_name.to_lowercase();
    
    // StrReplace 工具：old_string = 删除, new_string = 新增
    if tool_lower.contains("str_replace") || tool_lower.contains("strreplace") || tool_lower.contains("replace") || tool_lower.contains("edit") {
        let old_lines = input.get("old_string")
            .or_else(|| input.get("oldString"))
            .or_else(|| input.get("old_content"))
            .or_else(|| input.get("search"))
            .or_else(|| input.get("find"))
            .and_then(|s| s.as_str())
            .map(|s| s.lines().count())
            .unwrap_or(0);
        let new_lines = input.get("new_string")
            .or_else(|| input.get("newString"))
            .or_else(|| input.get("new_content"))
            .or_else(|| input.get("replace"))
            .or_else(|| input.get("replacement"))
            .and_then(|s| s.as_str())
            .map(|s| s.lines().count())
            .unwrap_or(0);
        
        // 直接返回新增和删除的行数
        return (new_lines as u32, old_lines as u32);
    }
    
    // Write 工具：统计新内容的行数
    if tool_lower.contains("write") || tool_lower.contains("create") {
        let content_lines = input.get("contents")
            .or_else(|| input.get("content"))
            .or_else(|| input.get("text"))
            .and_then(|s| s.as_str())
            .map(|s| s.lines().count())
            .unwrap_or(0);
        
        if content_lines > 0 {
            return (content_lines as u32, 0);
        }
    }
    
    (0, 0)
}

/// 保存会话统计到数据库
fn save_session_stats(
    conn: &rusqlite::Connection,
    session_id: &str,
    source: &str,
    provider_id: Option<&str>,
    stats: &SessionStats,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().timestamp();
    
    // 插入或更新会话统计
    conn.execute(
        "INSERT INTO session_stats (
            session_id, source, provider_id, conversation_count, tool_call_count,
            files_changed, lines_added, lines_deleted, response_time_ms, thinking_time_ms,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(session_id) DO UPDATE SET
            conversation_count = excluded.conversation_count,
            tool_call_count = excluded.tool_call_count,
            files_changed = excluded.files_changed,
            lines_added = excluded.lines_added,
            lines_deleted = excluded.lines_deleted,
            response_time_ms = excluded.response_time_ms,
            thinking_time_ms = excluded.thinking_time_ms,
            updated_at = excluded.updated_at",
        rusqlite::params![
            session_id,
            source,
            provider_id,
            stats.conversation_count,
            stats.tool_calls.values().sum::<u32>(),
            stats.files_changed,
            stats.lines_added,
            stats.lines_deleted,
            stats.response_time_ms,
            stats.thinking_time_ms,
            now,
            now,
        ],
    )
    .map_err(|e| AppError::Database(format!("保存会话统计失败: {e}")))?;
    
    // 插入工具调用明细
    for (tool_name, count) in &stats.tool_calls {
        conn.execute(
            "INSERT INTO tool_calls (session_id, tool_name, call_count, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT DO NOTHING",
            rusqlite::params![session_id, tool_name, count, now],
        )
        .map_err(|e| AppError::Database(format!("保存工具调用记录失败: {e}")))?;
    }
    
    Ok(())
}
