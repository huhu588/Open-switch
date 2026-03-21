use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub platform: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub working_directory: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
    pub message_count: u32,
    pub file_path: String,
    pub resume_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionCachePayload {
    version: u32,
    generated_at_ms: i64,
    sessions: Vec<SessionInfo>,
}

pub struct SessionManager;

impl SessionManager {
    pub fn list_sessions(platform: Option<String>, force_refresh: bool) -> Vec<SessionInfo> {
        if !force_refresh {
            if let Some(cache) = read_session_cache() {
                if session_cache_is_stale(cache.generated_at_ms) {
                    let platform_for_refresh = platform.clone();
                    std::thread::spawn(move || {
                        let fresh = scan_sessions(platform_for_refresh.as_deref());
                        if let Some(platform_name) = platform_for_refresh.as_deref() {
                            merge_platform_sessions_into_cache(platform_name, &fresh);
                        } else {
                            write_session_cache(&fresh);
                        }
                    });
                }
                return filter_sessions_by_platform(cache.sessions, platform.as_deref());
            }
        }

        let fresh = scan_sessions(platform.as_deref());
        if let Some(platform_name) = platform.as_deref() {
            merge_platform_sessions_into_cache(platform_name, &fresh);
        } else {
            write_session_cache(&fresh);
        }
        fresh
    }

    pub fn load_messages(platform: &str, source_path: &str) -> Result<Vec<SessionMessage>, String> {
        // kiro-json 类型路径包含完整文件路径，需要单独处理
        if source_path.starts_with("kiro-json:") {
            let file_path = &source_path["kiro-json:".len()..];
            return load_kiro_json_messages(file_path);
        }

        let path = Path::new(source_path);
        match platform {
            "claude-code" => load_claude_messages(path),
            "codex" => load_codex_messages(path),
            "gemini" => load_gemini_messages(path),
            "opencode" => load_opencode_messages(path),
            "openclaw" => load_openclaw_messages(path),
            "cursor" | "windsurf" | "kiro" | "antigravity" | "codebuddy"
            | "codebuddy_cn" | "qoder" | "trae" | "workbuddy"
            | "github-copilot" | "augment" => load_vscode_messages(platform, source_path),
            "warp" => load_warp_messages(source_path),
            _ => Err(format!("不支持的平台: {}", platform)),
        }
    }

    pub fn search_sessions(query: &str, platform: Option<String>, force_refresh: bool) -> Vec<SessionInfo> {
        let all = Self::list_sessions(platform, force_refresh);
        if query.is_empty() {
            return all;
        }

        let q = query.to_lowercase();
        all.into_iter()
            .filter(|s| {
                s.title.as_ref().map_or(false, |t| t.to_lowercase().contains(&q))
                    || s.summary.as_ref().map_or(false, |t| t.to_lowercase().contains(&q))
                    || s.working_directory.as_ref().map_or(false, |w| w.to_lowercase().contains(&q))
                    || s.platform.to_lowercase().contains(&q)
                    || s.id.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn delete_session(platform: &str, session_id: &str, source_path: &str) -> Result<bool, String> {
        let deleted = match platform {
            "claude-code" => delete_claude_session(session_id, source_path),
            "codex" => delete_codex_session(source_path),
            "gemini" => delete_gemini_session(source_path),
            "opencode" => delete_opencode_session(session_id, source_path),
            "openclaw" => delete_openclaw_session(session_id, source_path),
            "cursor" | "windsurf" | "kiro" | "antigravity" | "codebuddy"
            | "codebuddy_cn" | "qoder" | "trae" | "workbuddy"
            | "github-copilot" | "augment" => delete_vscode_session(session_id, source_path),
            "warp" => delete_warp_session(session_id),
            _ => Err(format!("不支持的平台: {}", platform)),
        }?;

        if deleted {
            remove_session_from_cache(platform, session_id, source_path);
        }

        Ok(deleted)
    }
}

const SESSION_CACHE_VERSION: u32 = 1;
const SESSION_CACHE_REFRESH_MS: i64 = 5 * 60 * 1000;

fn current_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn session_cache_is_stale(generated_at_ms: i64) -> bool {
    current_timestamp_ms().saturating_sub(generated_at_ms) > SESSION_CACHE_REFRESH_MS
}

fn get_session_cache_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = home.join(".config").join("opencode");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("session_cache.json"))
}

fn read_session_cache() -> Option<SessionCachePayload> {
    let cache_path = get_session_cache_path()?;
    let content = fs::read_to_string(cache_path).ok()?;
    let payload: SessionCachePayload = serde_json::from_str(&content).ok()?;
    (payload.version == SESSION_CACHE_VERSION).then_some(payload)
}

fn write_session_cache(sessions: &[SessionInfo]) {
    let Some(cache_path) = get_session_cache_path() else { return };
    let payload = SessionCachePayload {
        version: SESSION_CACHE_VERSION,
        generated_at_ms: current_timestamp_ms(),
        sessions: sessions.to_vec(),
    };
    let Ok(content) = serde_json::to_string(&payload) else { return };
    let _ = fs::write(cache_path, content);
}

fn merge_platform_sessions_into_cache(platform: &str, fresh_sessions: &[SessionInfo]) {
    let Some(mut cache) = read_session_cache() else { return };
    cache.sessions.retain(|session| session.platform != platform);
    cache.sessions.extend(fresh_sessions.iter().cloned());
    cache.sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    cache.generated_at_ms = current_timestamp_ms();
    write_session_cache(&cache.sessions);
}

fn remove_session_from_cache(platform: &str, session_id: &str, source_path: &str) {
    let Some(mut cache) = read_session_cache() else { return };
    cache.sessions.retain(|session| {
        !(session.platform == platform && session.id == session_id && session.file_path == source_path)
    });
    cache.generated_at_ms = current_timestamp_ms();
    write_session_cache(&cache.sessions);
}

fn filter_sessions_by_platform(mut sessions: Vec<SessionInfo>, platform: Option<&str>) -> Vec<SessionInfo> {
    if let Some(platform_name) = platform {
        sessions.retain(|session| session.platform == platform_name);
    }
    sessions
}

fn scan_sessions(platform: Option<&str>) -> Vec<SessionInfo> {
    type Scanner = fn() -> Vec<SessionInfo>;
    let all_scanners: Vec<(&str, Scanner)> = vec![
        ("claude-code", scan_claude_sessions as Scanner),
        ("codex", scan_codex_sessions as Scanner),
        ("gemini", scan_gemini_sessions as Scanner),
        ("opencode", scan_opencode_sessions as Scanner),
        ("openclaw", scan_openclaw_sessions as Scanner),
        ("cursor", scan_cursor_sessions as Scanner),
        ("windsurf", scan_windsurf_sessions as Scanner),
        ("kiro", scan_kiro_sessions as Scanner),
        ("antigravity", scan_antigravity_sessions as Scanner),
        ("codebuddy", scan_codebuddy_sessions as Scanner),
        ("codebuddy_cn", scan_codebuddy_cn_sessions as Scanner),
        ("qoder", scan_qoder_sessions as Scanner),
        ("trae", scan_trae_sessions as Scanner),
        ("workbuddy", scan_workbuddy_sessions as Scanner),
        ("github-copilot", scan_github_copilot_sessions as Scanner),
        ("warp", scan_warp_sessions as Scanner),
        ("augment", scan_augment_sessions as Scanner),
    ];

    let scanners: Vec<(&str, Scanner)> = match platform {
        Some(platform_name) => all_scanners
            .into_iter()
            .filter(|(name, _)| *name == platform_name)
            .collect(),
        None => all_scanners,
    };

    let mut sessions: Vec<SessionInfo> = scanners
        .into_iter()
        .flat_map(|(_, scanner)| scanner())
        .collect();

    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    sessions
}

// ──────────────────────────────────────────────
//  共享工具函数
// ──────────────────────────────────────────────

fn extract_text(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(extract_text_from_item)
            .filter(|t| !t.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(map) => map
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

fn extract_text_from_item(item: &Value) -> Option<String> {
    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    if let Some(text) = item.get("input_text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    if let Some(text) = item.get("output_text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    if let Some(content) = item.get("content") {
        let text = extract_text(content);
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

fn truncate_summary(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut result: String = trimmed.chars().take(max_chars).collect();
    result.push_str("...");
    result
}

fn path_basename(value: &str) -> Option<String> {
    let normalized = value.trim().trim_end_matches(['/', '\\']);
    normalized
        .split(['/', '\\'])
        .next_back()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn parse_timestamp_ms(value: &Value) -> Option<i64> {
    if let Some(s) = value.as_str() {
        return DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.timestamp_millis());
    }
    if let Some(n) = value.as_i64() {
        if n > 1_000_000_000_000 {
            return Some(n);
        }
        return Some(n * 1000);
    }
    if let Some(n) = value.as_f64() {
        if n > 1_000_000_000_000.0 {
            return Some(n as i64);
        }
        return Some((n * 1000.0) as i64);
    }
    None
}

fn read_head_tail_lines(
    path: &Path,
    head_n: usize,
    tail_n: usize,
) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let file = File::open(path)?;
    let file_len = file.metadata()?.len();

    if file_len < 16_384 {
        let reader = BufReader::new(file);
        let all: Vec<String> = reader.lines().map_while(Result::ok).collect();
        let head = all.iter().take(head_n).cloned().collect();
        let skip = all.len().saturating_sub(tail_n);
        let tail = all.into_iter().skip(skip).collect();
        return Ok((head, tail));
    }

    let reader = BufReader::new(file);
    let head: Vec<String> = reader.lines().take(head_n).map_while(Result::ok).collect();

    let seek_pos = file_len.saturating_sub(16_384);
    let mut file2 = File::open(path)?;
    file2.seek(SeekFrom::Start(seek_pos))?;
    let tail_reader = BufReader::new(file2);
    let all_tail: Vec<String> = tail_reader.lines().map_while(Result::ok).collect();

    let skip_first = if seek_pos > 0 { 1 } else { 0 };
    let usable: Vec<String> = all_tail.into_iter().skip(skip_first).collect();
    let skip = usable.len().saturating_sub(tail_n);
    let tail = usable.into_iter().skip(skip).collect();

    Ok((head, tail))
}

fn collect_files_with_ext(root: &Path, ext: &str, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_ext(&path, ext, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            files.push(path);
        }
    }
}

// ──────────────────────────────────────────────
//  Claude Code
// ──────────────────────────────────────────────

fn get_claude_projects_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = home.join(".claude").join("projects");
    dir.exists().then_some(dir)
}

fn scan_claude_sessions() -> Vec<SessionInfo> {
    let root = match get_claude_projects_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let mut files = Vec::new();
    collect_files_with_ext(&root, "jsonl", &mut files);

    files
        .iter()
        .filter(|p| {
            !p.file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.starts_with("agent-"))
        })
        .filter_map(|path| parse_claude_session(path))
        .collect()
}

fn parse_claude_session(path: &Path) -> Option<SessionInfo> {
    let (head, tail) = read_head_tail_lines(path, 10, 30).ok()?;

    let mut session_id: Option<String> = None;
    let mut project_dir: Option<String> = None;
    let mut created_at: Option<i64> = None;

    for line in &head {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if session_id.is_none() {
            session_id = value.get("sessionId").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        if project_dir.is_none() {
            project_dir = value.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
        if created_at.is_none() {
            created_at = value.get("timestamp").and_then(parse_timestamp_ms);
        }
    }

    let mut last_active_at: Option<i64> = None;
    let mut summary: Option<String> = None;

    for line in tail.iter().rev() {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if last_active_at.is_none() {
            last_active_at = value.get("timestamp").and_then(parse_timestamp_ms);
        }
        if summary.is_none() {
            if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            if let Some(message) = value.get("message") {
                let text = message.get("content").map(extract_text).unwrap_or_default();
                if !text.trim().is_empty() {
                    summary = Some(truncate_summary(&text, 160));
                }
            }
        }
        if last_active_at.is_some() && summary.is_some() {
            break;
        }
    }

    let session_id = session_id.or_else(|| {
        path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
    })?;

    let title = project_dir.as_deref().and_then(path_basename);

    Some(SessionInfo {
        id: session_id.clone(),
        platform: "claude-code".to_string(),
        title,
        summary,
        working_directory: project_dir,
        created_at,
        updated_at: last_active_at.or(created_at),
        message_count: 0,
        file_path: path.to_string_lossy().to_string(),
        resume_command: Some(format!("claude --resume {}", session_id)),
    })
}

fn load_claude_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(v) => v,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let message = match value.get("message") {
            Some(m) => m,
            None => continue,
        };

        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let content = message.get("content").map(extract_text).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }

        let ts = value.get("timestamp").and_then(parse_timestamp_ms);

        messages.push(SessionMessage {
            role,
            content,
            timestamp: ts,
        });
    }

    Ok(messages)
}

// ──────────────────────────────────────────────
//  Codex
// ──────────────────────────────────────────────

fn get_codex_sessions_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let primary = home.join(".codex").join("sessions");
    if primary.exists() {
        return Some(primary);
    }
    let alt = home.join(".config").join("codex").join("sessions");
    alt.exists().then_some(alt)
}

fn scan_codex_sessions() -> Vec<SessionInfo> {
    let root = match get_codex_sessions_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let mut files = Vec::new();
    collect_files_with_ext(&root, "jsonl", &mut files);

    files.iter().filter_map(|path| parse_codex_session(path)).collect()
}

fn parse_codex_session(path: &Path) -> Option<SessionInfo> {
    let (head, tail) = read_head_tail_lines(path, 10, 30).ok()?;

    let mut session_id: Option<String> = None;
    let mut project_dir: Option<String> = None;
    let mut created_at: Option<i64> = None;

    for line in &head {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if created_at.is_none() {
            created_at = value.get("timestamp").and_then(parse_timestamp_ms);
        }
        if value.get("type").and_then(Value::as_str) == Some("session_meta") {
            if let Some(payload) = value.get("payload") {
                if session_id.is_none() {
                    session_id = payload.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
                if project_dir.is_none() {
                    project_dir = payload.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());
                }
            }
        }
    }

    let mut last_active_at: Option<i64> = None;
    let mut summary: Option<String> = None;

    for line in tail.iter().rev() {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if last_active_at.is_none() {
            last_active_at = value.get("timestamp").and_then(parse_timestamp_ms);
        }
        if summary.is_none() && value.get("type").and_then(Value::as_str) == Some("response_item") {
            if let Some(payload) = value.get("payload") {
                if payload.get("type").and_then(Value::as_str) == Some("message") {
                    let text = payload.get("content").map(extract_text).unwrap_or_default();
                    if !text.trim().is_empty() {
                        summary = Some(truncate_summary(&text, 160));
                    }
                }
            }
        }
        if last_active_at.is_some() && summary.is_some() {
            break;
        }
    }

    let session_id = session_id.or_else(|| {
        let file_name = path.file_name()?.to_string_lossy().to_string();
        extract_uuid(&file_name)
    })?;

    let title = project_dir.as_deref().and_then(path_basename);

    Some(SessionInfo {
        id: session_id.clone(),
        platform: "codex".to_string(),
        title,
        summary,
        working_directory: project_dir,
        created_at,
        updated_at: last_active_at.or(created_at),
        message_count: 0,
        file_path: path.to_string_lossy().to_string(),
        resume_command: Some(format!("codex resume {}", session_id)),
    })
}

fn extract_uuid(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let pattern_len = 36; // 8-4-4-4-12
    if chars.len() < pattern_len {
        return None;
    }
    for start in 0..=(chars.len() - pattern_len) {
        let candidate: String = chars[start..start + pattern_len].iter().collect();
        if is_uuid_format(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_uuid_format(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return false;
    }
    let expected_lens = [8, 4, 4, 4, 12];
    parts.iter().zip(expected_lens.iter()).all(|(part, &len)| {
        part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit())
    })
}

fn load_codex_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(v) => v,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if value.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }

        let payload = match value.get("payload") {
            Some(p) => p,
            None => continue,
        };

        if payload.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let role = payload
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let content = payload.get("content").map(extract_text).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }

        let ts = value.get("timestamp").and_then(parse_timestamp_ms);

        messages.push(SessionMessage {
            role,
            content,
            timestamp: ts,
        });
    }

    Ok(messages)
}

// ──────────────────────────────────────────────
//  Gemini CLI
// ──────────────────────────────────────────────

fn get_gemini_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = home.join(".gemini");
    dir.exists().then_some(dir)
}

fn scan_gemini_sessions() -> Vec<SessionInfo> {
    let gemini_dir = match get_gemini_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let tmp_dir = gemini_dir.join("tmp");
    if !tmp_dir.exists() {
        return Vec::new();
    }

    let mut sessions = Vec::new();
    let project_dirs = match fs::read_dir(&tmp_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    for entry in project_dirs.flatten() {
        let chats_dir = entry.path().join("chats");
        if !chats_dir.is_dir() {
            continue;
        }

        let chat_files = match fs::read_dir(&chats_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for file_entry in chat_files.flatten() {
            let path = file_entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            if let Some(meta) = parse_gemini_session(&path) {
                sessions.push(meta);
            }
        }
    }

    sessions
}

fn parse_gemini_session(path: &Path) -> Option<SessionInfo> {
    let data = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&data).ok()?;

    let session_id = value.get("sessionId").and_then(Value::as_str)?.to_string();
    let created_at = value.get("startTime").and_then(parse_timestamp_ms);
    let last_active_at = value.get("lastUpdated").and_then(parse_timestamp_ms);

    let title = value
        .get("messages")
        .and_then(Value::as_array)
        .and_then(|msgs| {
            msgs.iter()
                .find(|m| m.get("type").and_then(Value::as_str) == Some("user"))
                .and_then(|m| m.get("content").and_then(Value::as_str))
                .filter(|s| !s.trim().is_empty())
                .map(|s| truncate_summary(s, 160))
        });

    Some(SessionInfo {
        id: session_id.clone(),
        platform: "gemini".to_string(),
        title: title.clone(),
        summary: title,
        working_directory: None,
        created_at,
        updated_at: last_active_at.or(created_at),
        message_count: 0,
        file_path: path.to_string_lossy().to_string(),
        resume_command: Some(format!("gemini --resume {}", session_id)),
    })
}

fn load_gemini_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let data = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
    let value: Value = serde_json::from_str(&data).map_err(|e| format!("解析 JSON 失败: {}", e))?;

    let messages = value
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| "未找到 messages 数组".to_string())?;

    let mut result = Vec::new();
    for msg in messages {
        let content = match msg.get("content").and_then(Value::as_str) {
            Some(c) if !c.trim().is_empty() => c.to_string(),
            _ => continue,
        };

        let role = match msg.get("type").and_then(Value::as_str) {
            Some("gemini") => "assistant".to_string(),
            Some("user") => "user".to_string(),
            Some(other) => other.to_string(),
            None => continue,
        };

        let ts = msg.get("timestamp").and_then(parse_timestamp_ms);

        result.push(SessionMessage {
            role,
            content,
            timestamp: ts,
        });
    }

    Ok(result)
}

// ──────────────────────────────────────────────
//  OpenCode
// ──────────────────────────────────────────────

fn get_opencode_storage_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        if !xdg.is_empty() {
            let dir = PathBuf::from(xdg).join("opencode").join("storage");
            if dir.exists() {
                return Some(dir);
            }
        }
    }
    let home = dirs::home_dir()?;
    let dir = home.join(".local").join("share").join("opencode").join("storage");
    dir.exists().then_some(dir)
}

fn scan_opencode_sessions() -> Vec<SessionInfo> {
    let storage = match get_opencode_storage_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let session_dir = storage.join("session");
    if !session_dir.exists() {
        return Vec::new();
    }

    let mut json_files = Vec::new();
    collect_files_with_ext(&session_dir, "json", &mut json_files);

    json_files
        .iter()
        .filter_map(|path| parse_opencode_session(&storage, path))
        .collect()
}

fn parse_opencode_session(storage: &Path, path: &Path) -> Option<SessionInfo> {
    let data = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&data).ok()?;

    let session_id = value.get("id").and_then(Value::as_str)?.to_string();
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let directory = value
        .get("directory")
        .and_then(Value::as_str)
        .map(|s| s.to_string());

    let created_at = value
        .get("time")
        .and_then(|t| t.get("created"))
        .and_then(parse_timestamp_ms);
    let updated_at = value
        .get("time")
        .and_then(|t| t.get("updated"))
        .and_then(parse_timestamp_ms);

    let display_title = title.clone().or_else(|| {
        directory.as_deref().and_then(path_basename)
    });

    let msg_dir = storage.join("message").join(&session_id);
    let source_path = msg_dir.to_string_lossy().to_string();

    Some(SessionInfo {
        id: session_id.clone(),
        platform: "opencode".to_string(),
        title: display_title,
        summary: title,
        working_directory: directory,
        created_at,
        updated_at: updated_at.or(created_at),
        message_count: 0,
        file_path: source_path,
        resume_command: Some(format!("opencode session resume {}", session_id)),
    })
}

fn load_opencode_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    if !path.is_dir() {
        return Err(format!("消息目录不存在: {}", path.display()));
    }

    let storage = path
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| "无法确定 storage 根目录".to_string())?;

    let mut msg_files = Vec::new();
    collect_files_with_ext(path, "json", &mut msg_files);

    let mut entries: Vec<(i64, String, String)> = Vec::new();

    for msg_path in &msg_files {
        let data = match fs::read_to_string(msg_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_id = match value.get("id").and_then(Value::as_str) {
            Some(id) => id.to_string(),
            None => continue,
        };

        let role = value
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();

        let created_ts = value
            .get("time")
            .and_then(|t| t.get("created"))
            .and_then(parse_timestamp_ms)
            .unwrap_or(0);

        let part_dir = storage.join("part").join(&msg_id);
        let text = collect_opencode_parts_text(&part_dir);
        if text.trim().is_empty() {
            continue;
        }

        entries.push((created_ts, role, text));
    }

    entries.sort_by_key(|(ts, _, _)| *ts);

    let messages = entries
        .into_iter()
        .map(|(ts, role, content)| SessionMessage {
            role,
            content,
            timestamp: if ts > 0 { Some(ts) } else { None },
        })
        .collect();

    Ok(messages)
}

fn collect_opencode_parts_text(part_dir: &Path) -> String {
    if !part_dir.is_dir() {
        return String::new();
    }

    let mut parts = Vec::new();
    collect_files_with_ext(part_dir, "json", &mut parts);

    let mut texts = Vec::new();
    for part_path in &parts {
        let data = match fs::read_to_string(part_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if value.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }

        if let Some(text) = value.get("text").and_then(Value::as_str) {
            if !text.trim().is_empty() {
                texts.push(text.to_string());
            }
        }
    }

    texts.join("\n")
}

// ──────────────────────────────────────────────
//  OpenClaw
// ──────────────────────────────────────────────

fn get_openclaw_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let dir = home.join(".openclaw");
    dir.exists().then_some(dir)
}

fn scan_openclaw_sessions() -> Vec<SessionInfo> {
    let openclaw_dir = match get_openclaw_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let agents_dir = openclaw_dir.join("agents");
    if !agents_dir.exists() {
        return Vec::new();
    }

    let mut sessions = Vec::new();

    let agent_entries = match fs::read_dir(&agents_dir) {
        Ok(entries) => entries,
        Err(_) => return sessions,
    };

    for agent_entry in agent_entries.flatten() {
        let agent_path = agent_entry.path();
        if !agent_path.is_dir() {
            continue;
        }

        let sessions_dir = agent_path.join("sessions");
        if !sessions_dir.is_dir() {
            continue;
        }

        let session_entries = match fs::read_dir(&sessions_dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in session_entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
                continue;
            }
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == "sessions.json")
                .unwrap_or(false)
            {
                continue;
            }
            if let Some(meta) = parse_openclaw_session(&path) {
                sessions.push(meta);
            }
        }
    }

    sessions
}

fn parse_openclaw_session(path: &Path) -> Option<SessionInfo> {
    let (head, tail) = read_head_tail_lines(path, 10, 30).ok()?;

    let mut session_id: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut created_at: Option<i64> = None;
    let mut summary: Option<String> = None;

    for line in &head {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if created_at.is_none() {
            created_at = value.get("timestamp").and_then(parse_timestamp_ms);
        }

        let event_type = value.get("type").and_then(Value::as_str).unwrap_or("");

        if event_type == "session" {
            if session_id.is_none() {
                session_id = value.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
            if cwd.is_none() {
                cwd = value.get("cwd").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
            continue;
        }

        if event_type == "message" && summary.is_none() {
            if let Some(message) = value.get("message") {
                let text = message.get("content").map(extract_text).unwrap_or_default();
                if !text.trim().is_empty() {
                    summary = Some(truncate_summary(&text, 160));
                }
            }
        }
    }

    let mut last_active_at: Option<i64> = None;
    for line in tail.iter().rev() {
        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(ts) = value.get("timestamp").and_then(parse_timestamp_ms) {
            last_active_at = Some(ts);
            break;
        }
    }

    let session_id = session_id.or_else(|| {
        path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string())
    })?;

    let title = cwd.as_deref().and_then(path_basename);

    Some(SessionInfo {
        id: session_id,
        platform: "openclaw".to_string(),
        title,
        summary,
        working_directory: cwd,
        created_at,
        updated_at: last_active_at.or(created_at),
        message_count: 0,
        file_path: path.to_string_lossy().to_string(),
        resume_command: None,
    })
}

fn load_openclaw_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(v) => v,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if value.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let message = match value.get("message") {
            Some(m) => m,
            None => continue,
        };

        let raw_role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        let role = match raw_role {
            "toolResult" => "tool".to_string(),
            other => other.to_string(),
        };

        let content = message.get("content").map(extract_text).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }

        let ts = value.get("timestamp").and_then(parse_timestamp_ms);

        messages.push(SessionMessage {
            role,
            content,
            timestamp: ts,
        });
    }

    Ok(messages)
}

// ──────────────────────────────────────────────
//  删除会话
// ──────────────────────────────────────────────

fn delete_claude_session(_session_id: &str, source_path: &str) -> Result<bool, String> {
    let path = Path::new(source_path);
    if !path.exists() {
        return Err("会话文件不存在".to_string());
    }
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(true)
}

fn delete_codex_session(source_path: &str) -> Result<bool, String> {
    let path = Path::new(source_path);
    if !path.exists() {
        return Err("会话文件不存在".to_string());
    }
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(true)
}

fn delete_gemini_session(source_path: &str) -> Result<bool, String> {
    let path = Path::new(source_path);
    if !path.exists() {
        return Err("会话文件不存在".to_string());
    }
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(true)
}

fn delete_opencode_session(session_id: &str, source_path: &str) -> Result<bool, String> {
    let msg_dir = Path::new(source_path);
    if msg_dir.exists() && msg_dir.is_dir() {
        fs::remove_dir_all(msg_dir).map_err(|e| format!("删除消息目录失败: {}", e))?;
    }

    if let Some(storage) = get_opencode_storage_dir() {
        let session_file = storage.join("session").join(format!("{}.json", session_id));
        if session_file.exists() {
            fs::remove_file(&session_file)
                .map_err(|e| format!("删除会话文件失败: {}", e))?;
        }
    }

    Ok(true)
}

fn delete_openclaw_session(_session_id: &str, source_path: &str) -> Result<bool, String> {
    let path = Path::new(source_path);
    if !path.exists() {
        return Err("会话文件不存在".to_string());
    }
    fs::remove_file(path).map_err(|e| format!("删除失败: {}", e))?;
    Ok(true)
}

// ──────────────────────────────────────────────
//  VSCode 风格平台 (Cursor / Windsurf / Kiro / Antigravity / CodeBuddy 等)
// ──────────────────────────────────────────────

const VSCODE_OPEN_FLAGS: rusqlite::OpenFlags = rusqlite::OpenFlags::from_bits_truncate(
    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY.bits()
        | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX.bits(),
);

fn platform_to_app_names(platform: &str) -> Vec<&'static str> {
    match platform {
        "cursor" => vec!["Cursor"],
        "windsurf" => vec!["Windsurf", "WindSurf"],
        "kiro" => vec!["Kiro"],
        "antigravity" => vec!["Antigravity"],
        "codebuddy" => vec!["CodeBuddy"],
        "codebuddy_cn" => vec!["CodeBuddy CN", "codebuddy cn", "codebuddy-cn"],
        "qoder" => vec!["Qoder"],
        "trae" => vec!["Trae"],
        "workbuddy" => vec!["WorkBuddy"],
        "github-copilot" => vec!["Code", "Code - Insiders"],
        "augment" => vec!["Code", "Code - Insiders", "Augment"],
        _ => vec![],
    }
}

fn get_appdata_dirs() -> Vec<PathBuf> {
    let mut dirs_list: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") { dirs_list.push(PathBuf::from(appdata)); }
        if let Ok(localapp) = std::env::var("LOCALAPPDATA") { dirs_list.push(PathBuf::from(localapp)); }
        if let Some(home) = dirs::home_dir() {
            let r = home.join("AppData").join("Roaming");
            if r.exists() && !dirs_list.iter().any(|d| d == &r) { dirs_list.push(r); }
            let l = home.join("AppData").join("Local");
            if l.exists() && !dirs_list.iter().any(|d| d == &l) { dirs_list.push(l); }
        }
    }
    #[cfg(target_os = "macos")]
    { if let Some(home) = dirs::home_dir() { dirs_list.push(home.join("Library").join("Application Support")); } }
    #[cfg(target_os = "linux")]
    { if let Some(home) = dirs::home_dir() { dirs_list.push(home.join(".config")); dirs_list.push(home.join(".local").join("share")); } }
    dirs_list
}

fn get_vscode_db_paths(app_names: &[&str]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let base_dirs = get_appdata_dirs();
    for base in &base_dirs {
        for name in app_names {
            let user_dir = base.join(name).join("User");
            if !user_dir.exists() { continue; }
            let global_db = user_dir.join("globalStorage").join("state.vscdb");
            if global_db.exists() {
                let key = global_db.to_string_lossy().to_string();
                if seen.insert(key) { paths.push(global_db); }
            }
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

fn vscode_table_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    conn.prepare(&format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", name))
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(true)))
        .unwrap_or(false)
}

fn vscode_query_item_table(conn: &rusqlite::Connection, key: &str) -> Option<Vec<u8>> {
    conn.query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |row| {
        row.get::<_, Vec<u8>>(0)
            .ok()
            .or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes()))
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }).ok()
}

fn vscode_parse_json_bytes(bytes: &[u8]) -> Option<Value> {
    if let Ok(json) = serde_json::from_slice::<Value>(bytes) { return Some(json); }
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        let trimmed = text.trim_matches('\u{0}').trim();
        if !trimmed.is_empty() { return serde_json::from_str::<Value>(trimmed).ok(); }
    }
    None
}

fn vscode_stringify_tool_former_value(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return None;
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                return vscode_stringify_tool_former_value(&parsed).or_else(|| Some(trimmed.to_string()));
            }
            Some(trimmed.to_string())
        }
        Value::Object(_) | Value::Array(_) => serde_json::to_string(value).ok().filter(|s| !s.is_empty()),
        _ => Some(value.to_string()),
    }
}

fn vscode_extract_tool_former_content(msg: &Value) -> Option<String> {
    let tool = msg.get("toolFormerData")?;
    let name = tool.get("name")
        .or_else(|| tool.get("toolName"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown_tool");
    let status = tool.get("status")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());
    let params = tool.get("params").and_then(vscode_stringify_tool_former_value);
    let result = tool.get("result")
        .or_else(|| tool.get("output"))
        .or_else(|| tool.get("response"))
        .and_then(vscode_stringify_tool_former_value);

    let mut lines = vec![match status {
        Some(status) => format!("[工具调用] {} ({})", name, status),
        None => format!("[工具调用] {}", name),
    }];
    if let Some(params) = params {
        lines.push(format!("参数: {}", params));
    }
    if let Some(result) = result {
        lines.push(format!("结果: {}", result));
    }
    Some(lines.join("\n"))
}

fn vscode_extract_rich_text_content(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                    return vscode_extract_rich_text_content(&parsed).or_else(|| Some(trimmed.to_string()));
                }
            }
            Some(trimmed.to_string())
        }
        Value::Array(arr) => {
            let parts: Vec<String> = arr.iter()
                .filter_map(vscode_extract_rich_text_content)
                .filter(|s| !s.is_empty())
                .collect();
            if parts.is_empty() { None } else { Some(parts.join("")) }
        }
        Value::Object(obj) => {
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                return Some(text.to_string());
            }
            let mut parts = Vec::new();
            for key in ["children", "root", "content", "value"] {
                if let Some(val) = obj.get(key).and_then(vscode_extract_rich_text_content) {
                    if !val.is_empty() {
                        parts.push(val);
                    }
                }
            }
            if parts.is_empty() { None } else { Some(parts.join("")) }
        }
        _ => None,
    }
}

fn normalize_session_timestamp_ms(ts: i64) -> i64 {
    if ts.abs() < 10_000_000_000 { ts.saturating_mul(1000) } else { ts }
}

fn vscode_extract_timestamp_ms(msg: &Value) -> Option<i64> {
    msg.get("timestamp").or_else(|| msg.get("createdAt")).or_else(|| msg.get("created_at")).and_then(|v| {
        if let Some(ts) = v.as_i64() {
            Some(normalize_session_timestamp_ms(ts))
        } else if let Some(ts) = v.as_f64() {
            Some(normalize_session_timestamp_ms(ts as i64))
        } else if let Some(ts) = v.as_str() {
            ts.parse::<i64>().ok()
                .map(normalize_session_timestamp_ms)
                .or_else(|| DateTime::parse_from_rfc3339(ts).ok().map(|dt| dt.timestamp_millis()))
        } else {
            None
        }
    })
}

fn vscode_extract_content(msg: &Value) -> Option<String> {
    for field in &["rawText", "text", "content", "message", "value", "displayText", "markdownContent"] {
        if let Some(val) = msg.get(*field) {
            if let Some(s) = val.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
            if let Some(arr) = val.as_array() {
                let parts: Vec<String> = arr.iter().filter_map(|p| {
                    if let Some(s) = p.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
                    for f in &["text", "value", "content", "output_text"] {
                        if let Some(t) = p.get(*f).and_then(|t| t.as_str()) { if !t.is_empty() { return Some(t.to_string()); } }
                    }
                    None
                }).collect();
                if !parts.is_empty() { return Some(parts.join("\n")); }
            }
        }
    }
    if let Some(rich) = msg.get("richText").or_else(|| msg.get("richtext")) {
        if let Some(text) = vscode_extract_rich_text_content(rich) {
            if !text.is_empty() { return Some(text); }
        }
    }
    for field in &["codeBlocks", "suggestedCodeBlocks"] {
        if let Some(blocks) = msg.get(*field).and_then(|b| b.as_array()) {
            let parts: Vec<String> = blocks.iter().filter_map(|b| {
                b.get("code").or_else(|| b.get("text")).or_else(|| b.get("content")).and_then(|c| c.as_str()).map(|s| s.to_string())
            }).collect();
            if !parts.is_empty() { return Some(parts.join("\n\n")); }
        }
    }
    for field in &["thinking", "reasoning"] {
        if let Some(text) = msg.get(*field)
            .and_then(|v| v.get("text").or_else(|| v.get("content")).or_else(|| v.get("value")))
            .and_then(vscode_extract_rich_text_content)
        {
            if !text.is_empty() { return Some(text); }
        }
    }
    if let Some(tool_summary) = vscode_extract_tool_former_content(msg) {
        return Some(tool_summary);
    }
    if let Some(obj) = msg.as_object() {
        let skip = ["type","role","sender","isUser","model","modelId","modelName",
            "timestamp","createdAt","created_at","id","bubbleId","composerId",
            "key","sessionId","tabId","index","version","status","error"];
        let mut best: Option<String> = None;
        let mut best_len = 0usize;
        for (k, v) in obj {
            if skip.contains(&k.as_str()) { continue; }
            if let Some(s) = v.as_str() {
                if s.len() > best_len && s.len() >= 5 { best_len = s.len(); best = Some(s.to_string()); }
            }
        }
        if best.is_some() { return best; }
    }
    None
}

fn vscode_build_session_message(msg: &Value) -> Option<SessionMessage> {
    let role = vscode_determine_role(msg).unwrap_or_else(|| "assistant".to_string());
    let content = vscode_extract_content(msg)?;
    if content.is_empty() {
        return None;
    }
    Some(SessionMessage {
        role,
        content,
        timestamp: vscode_extract_timestamp_ms(msg),
    })
}

fn vscode_load_bubble_messages(
    conn: &rusqlite::Connection,
    session_id: &str,
    ordered_bubble_ids: Option<&[String]>,
) -> Vec<SessionMessage> {
    if !vscode_table_exists(conn, "cursorDiskKV") {
        return Vec::new();
    }

    if let Some(bubble_ids) = ordered_bubble_ids {
        let mut messages = Vec::new();
        for bubble_id in bubble_ids {
            let key = format!("bubbleId:{}:{}", session_id, bubble_id);
            let Ok(value) = conn.query_row("SELECT value FROM cursorDiskKV WHERE key = ?1", [&key], |row| {
                row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
            }) else {
                continue;
            };
            let Some(json) = vscode_parse_json_bytes(&value) else { continue };
            if let Some(message) = vscode_build_session_message(&json) {
                messages.push(message);
            }
        }
        return messages;
    }

    let key_pattern = format!("bubbleId:{}:%", session_id);
    let mut entries: Vec<(i64, usize, SessionMessage)> = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT value FROM cursorDiskKV WHERE key LIKE ?1") {
        if let Ok(rows) = stmt.query_map([&key_pattern], |row| {
            row.get::<_, Vec<u8>>(0).ok().or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes())).ok_or(rusqlite::Error::QueryReturnedNoRows)
        }) {
            for (idx, row_result) in rows.enumerate() {
                let Ok(vb) = row_result else { continue };
                let Some(json) = vscode_parse_json_bytes(&vb) else { continue };
                let Some(message) = vscode_build_session_message(&json) else { continue };
                entries.push((message.timestamp.unwrap_or(i64::MAX), idx, message));
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    entries.into_iter().map(|(_, _, msg)| msg).collect()
}

fn vscode_determine_role(msg: &Value) -> Option<String> {
    if let Some(v) = msg.get("type") {
        if let Some(n) = v.as_u64() { return Some(if n == 1 { "user" } else { "assistant" }.to_string()); }
        if let Some(s) = v.as_str() {
            let l = s.to_lowercase();
            if l == "user" || l == "human" { return Some("user".to_string()); }
            if l == "assistant" || l == "ai" || l == "bot" || l == "agent" { return Some("assistant".to_string()); }
        }
    }
    if let Some(v) = msg.get("role") {
        if let Some(n) = v.as_u64() { return Some(if n == 1 { "user" } else { "assistant" }.to_string()); }
        if let Some(s) = v.as_str() { return Some(s.to_lowercase()); }
    }
    if let Some(b) = msg.get("isUser").and_then(|v| v.as_bool()) { return Some(if b { "user" } else { "assistant" }.to_string()); }
    if let Some(s) = msg.get("sender").and_then(|v| v.as_str()) { return Some(if s == "user" { "user" } else { "assistant" }.to_string()); }
    None
}

fn vscode_get_str_field(json: &Value, fields: &[&str]) -> Option<String> {
    for f in fields {
        if let Some(s) = json.get(*f).and_then(|v| v.as_str()) { if !s.is_empty() { return Some(s.to_string()); } }
    }
    None
}

fn vscode_get_composer_meta(conn: &rusqlite::Connection, composer_id: &str) -> (Option<String>, Option<i64>) {
    let key = format!("composerData:{}", composer_id);
    if let Ok(value) = conn.query_row("SELECT value FROM cursorDiskKV WHERE key = ?1", [&key], |row| {
        row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
    }) {
        if let Some(json) = vscode_parse_json_bytes(&value) {
            return (
                vscode_get_str_field(&json, &["name", "title"]),
                json.get("createdAt").or_else(|| json.get("created_at")).and_then(|v| v.as_i64()),
            );
        }
    }
    (None, None)
}

/// 从 VSCode 风格 state.vscdb 中扫描会话
fn scan_vscode_sessions(app_names: &[&str], platform_name: &str) -> Vec<SessionInfo> {
    let db_paths = get_vscode_db_paths(app_names);
    let mut sessions = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for db_path in &db_paths {
        let Ok(conn) = rusqlite::Connection::open_with_flags(db_path, VSCODE_OPEN_FLAGS) else { continue };

        let mut inline_ids: HashSet<String> = HashSet::new();

        // Phase 1: cursorDiskKV composerData (内联消息)
        if vscode_table_exists(&conn, "cursorDiskKV") {
            if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'") {
                if let Ok(rows) = stmt.query_map([], |row| {
                    let key: String = row.get(0)?;
                    let value = row.get::<_, Vec<u8>>(1).ok().or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
                    Ok((key, value))
                }) {
                    for row_result in rows.flatten() {
                        let (key, value_opt) = row_result;
                        let Some(vb) = value_opt else { continue };
                        let Some(json) = vscode_parse_json_bytes(&vb) else { continue };

                        let composer_id = json.get("composerId").and_then(|v| v.as_str()).map(|s| s.to_string())
                            .unwrap_or_else(|| key.strip_prefix("composerData:").unwrap_or(&key).to_string());
                        if seen_ids.contains(&composer_id) { continue; }

                        let title = vscode_get_str_field(&json, &["name", "title"]);
                        let created_at = json.get("createdAt").or_else(|| json.get("created_at")).and_then(|v| v.as_i64());

                        let mut msg_count = 0u32;
                        let mut summary: Option<String> = None;

                        if let Some(conv) = json.get("conversation").or_else(|| json.get("messages")).and_then(|v| v.as_array()) {
                            msg_count = conv.len() as u32;
                            summary = conv.first().and_then(|m| vscode_extract_content(m)).map(|s| truncate_summary(&s, 160));
                        }
                        if msg_count == 0 {
                            if let Some(bubbles) = json.get("bubbles").and_then(|v| v.as_array()) {
                                msg_count = bubbles.len() as u32;
                                summary = bubbles.first().and_then(|m| vscode_extract_content(m)).map(|s| truncate_summary(&s, 160));
                            }
                        }

                        if msg_count > 0 {
                            seen_ids.insert(composer_id.clone());
                            inline_ids.insert(composer_id.clone());
                            sessions.push(SessionInfo {
                                id: composer_id.clone(),
                                platform: platform_name.to_string(),
                                title,
                                summary,
                                working_directory: None,
                                created_at,
                                updated_at: created_at,
                                message_count: msg_count,
                                file_path: format!("composerData:{}", composer_id),
                                resume_command: None,
                            });
                        }
                    }
                }
            }

            // Phase 2: bubbleId 分离消息 — 查找没有内联消息的 composer
            let mut bubble_composer_ids: HashSet<String> = HashSet::new();
            if let Ok(mut stmt) = conn.prepare("SELECT key FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'") {
                if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
                    for key_result in rows.flatten() {
                        let parts: Vec<&str> = key_result.split(':').collect();
                        if parts.len() >= 3 {
                            let cid = parts[1].to_string();
                            if !inline_ids.contains(&cid) && !seen_ids.contains(&cid) {
                                bubble_composer_ids.insert(cid);
                            }
                        }
                    }
                }
            }
            for composer_id in &bubble_composer_ids {
                seen_ids.insert(composer_id.clone());
                let key_pattern = format!("bubbleId:{}:%", composer_id);
                let bubble_count = conn.query_row(
                    "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE ?1",
                    [&key_pattern],
                    |row| row.get::<_, u32>(0),
                ).unwrap_or(0);
                if bubble_count == 0 { continue; }

                let (title, created_at) = vscode_get_composer_meta(&conn, composer_id);

                let summary = {
                    let first_key = format!("bubbleId:{}:0", composer_id);
                    conn.query_row("SELECT value FROM cursorDiskKV WHERE key = ?1", [&first_key], |row| {
                        row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
                    }).ok().and_then(|vb| vscode_parse_json_bytes(&vb)).and_then(|j| vscode_extract_content(&j)).map(|s| truncate_summary(&s, 160))
                };

                sessions.push(SessionInfo {
                    id: composer_id.clone(),
                    platform: platform_name.to_string(),
                    title,
                    summary,
                    working_directory: None,
                    created_at,
                    updated_at: created_at,
                    message_count: bubble_count,
                    file_path: format!("bubble:{}", composer_id),
                    resume_command: None,
                });
            }
        }

        // Phase 3: ItemTable chat tabs
        if vscode_table_exists(&conn, "ItemTable") {
            if let Some(vb) = vscode_query_item_table(&conn, "workbench.panel.aichat.view.aichat.chatdata") {
                if let Some(json) = vscode_parse_json_bytes(&vb) {
                    if let Some(tabs) = json.get("tabs").and_then(|v| v.as_array()) {
                        for tab in tabs {
                            let tab_id = tab.get("id").or_else(|| tab.get("tabId")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                            if tab_id.is_empty() || seen_ids.contains(&tab_id) { continue; }
                            seen_ids.insert(tab_id.clone());

                            let title = vscode_get_str_field(tab, &["chatTitle", "title"]);
                            let created_at = tab.get("createdAt").or_else(|| tab.get("timestamp")).and_then(|v| v.as_i64());
                            let msg_count = tab.get("bubbles").and_then(|v| v.as_array()).map(|a| a.len() as u32).unwrap_or(0);
                            if msg_count == 0 { continue; }

                            let summary = tab.get("bubbles").and_then(|v| v.as_array())
                                .and_then(|msgs| msgs.first()).and_then(|m| vscode_extract_content(m)).map(|s| truncate_summary(&s, 160));

                            sessions.push(SessionInfo {
                                id: tab_id.clone(),
                                platform: platform_name.to_string(),
                                title,
                                summary,
                                working_directory: None,
                                created_at,
                                updated_at: created_at,
                                message_count: msg_count,
                                file_path: format!("chatdata:{}", tab_id),
                                resume_command: None,
                            });
                        }
                    }
                }
            }

            // Phase 4: ItemTable composer.composerData
            if let Some(vb) = vscode_query_item_table(&conn, "composer.composerData") {
                if let Some(json) = vscode_parse_json_bytes(&vb) {
                    if let Some(all) = json.get("allComposers").and_then(|a| a.as_array()) {
                        for composer in all {
                            let cid = composer.get("composerId").or_else(|| composer.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                            if cid.is_empty() || seen_ids.contains(&cid) { continue; }
                            seen_ids.insert(cid.clone());

                            let title = vscode_get_str_field(composer, &["name", "title"]);
                            let created_at = composer.get("createdAt").and_then(|v| v.as_i64());
                            let msg_count = composer.get("conversation").and_then(|c| c.as_array()).map(|a| a.len() as u32).unwrap_or(0);
                            if msg_count == 0 { continue; }

                            let summary = composer.get("conversation").and_then(|c| c.as_array())
                                .and_then(|msgs| msgs.first()).and_then(|m| vscode_extract_content(m)).map(|s| truncate_summary(&s, 160));

                            sessions.push(SessionInfo {
                                id: cid.clone(),
                                platform: platform_name.to_string(),
                                title,
                                summary,
                                working_directory: None,
                                created_at,
                                updated_at: created_at,
                                message_count: msg_count,
                                file_path: format!("composer:{}", cid),
                                resume_command: None,
                            });
                        }
                    }
                }
            }

            // Phase 5: Trae icube 存储
            if let Some(vb) = vscode_query_item_table(&conn, "memento/icube-ai-agent-storage") {
                if let Some(json) = vscode_parse_json_bytes(&vb) {
                    if let Some(list) = json.get("list").and_then(|v| v.as_array()) {
                        for item in list {
                            if let Some(sid) = item.get("sessionId").and_then(|v| v.as_str()) {
                                if !sid.is_empty() && !seen_ids.contains(sid) {
                                    seen_ids.insert(sid.to_string());
                                    let title = item.get("title").or_else(|| item.get("name"))
                                        .and_then(|v| v.as_str()).map(|s| s.to_string());
                                    let created_at = item.get("createdAt").or_else(|| item.get("timestamp"))
                                        .and_then(|v| v.as_i64());

                                    sessions.push(SessionInfo {
                                        id: sid.to_string(),
                                        platform: platform_name.to_string(),
                                        title,
                                        summary: None,
                                        working_directory: None,
                                        created_at,
                                        updated_at: created_at,
                                        message_count: 1,
                                        file_path: format!("icube:{}", sid),
                                        resume_command: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            // Phase 6: Trae icube 输入历史
            if let Some(vb) = vscode_query_item_table(&conn, "icube-ai-agent-storage-input-history") {
                if let Some(json) = vscode_parse_json_bytes(&vb) {
                    if let Some(arr) = json.as_array() {
                        let valid_entries: Vec<&Value> = arr.iter().filter(|item| {
                            item.get("inputText").and_then(|v| v.as_str()).map_or(false, |s| !s.is_empty())
                        }).collect();
                        if !valid_entries.is_empty() {
                            let icube_hist_id = format!("{}-input-history", platform_name);
                            if !seen_ids.contains(&icube_hist_id) {
                                seen_ids.insert(icube_hist_id.clone());
                                let summary = valid_entries.first()
                                    .and_then(|item| item.get("inputText").and_then(|v| v.as_str()))
                                    .map(|s| truncate_summary(s, 160));
                                sessions.push(SessionInfo {
                                    id: icube_hist_id.clone(),
                                    platform: platform_name.to_string(),
                                    title: Some("Input History".to_string()),
                                    summary,
                                    working_directory: None,
                                    created_at: None,
                                    updated_at: None,
                                    message_count: valid_entries.len() as u32,
                                    file_path: format!("icube-history:{}", icube_hist_id),
                                    resume_command: None,
                                });
                            }
                        }
                    }
                }
            }

            // Phase 7: aiService.prompts — Cursor/Windsurf/Trae 等平台标准 AI 聊天键
            let ai_keys = [
                "aiService.prompts",
                "aiService.chatHistory",
                "aiService.conversations",
            ];
            for ai_key in &ai_keys {
                if let Some(vb) = vscode_query_item_table(&conn, ai_key) {
                    if let Some(json) = vscode_parse_json_bytes(&vb) {
                        let items = vscode_extract_prompt_items(&json);
                        if items.is_empty() { continue; }
                        let db_key_short = ai_key.replace('.', "_");
                        let ws_hash = db_path.parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("global");
                        let sid = format!("{}:{}", db_key_short, ws_hash);
                        if seen_ids.contains(&sid) { continue; }
                        seen_ids.insert(sid.clone());

                        let msg_count = items.len() as u32;
                        let summary = items.first()
                            .and_then(|item| vscode_extract_prompt_content(item))
                            .map(|s| truncate_summary(&s, 160));
                        let title = items.first()
                            .and_then(|item| vscode_extract_prompt_content(item))
                            .map(|s| truncate_summary(&s, 80));

                        sessions.push(SessionInfo {
                            id: sid.clone(),
                            platform: platform_name.to_string(),
                            title,
                            summary,
                            working_directory: None,
                            created_at: None,
                            updated_at: None,
                            message_count: msg_count,
                            file_path: format!("prompts:{}:{}", ai_key, ws_hash),
                            resume_command: None,
                        });
                    }
                }
            }
        }
    }

    sessions
}

/// 从 aiService.prompts 风格的 JSON 中提取消息条目
fn vscode_extract_prompt_items(json: &Value) -> Vec<&Value> {
    if let Some(arr) = json.as_array() {
        return arr.iter().collect();
    }
    if let Some(obj) = json.as_object() {
        for key in &["prompts", "messages", "items", "history", "chatHistory", "threads", "sessions"] {
            if let Some(arr) = obj.get(*key).and_then(|v| v.as_array()) {
                if !arr.is_empty() { return arr.iter().collect(); }
            }
        }
        // 深度搜索：找最大的 dict 数组
        let mut best: Vec<&Value> = Vec::new();
        fn find_best_array<'a>(val: &'a Value, best: &mut Vec<&'a Value>) {
            match val {
                Value::Array(arr) if !arr.is_empty() => {
                    if arr[0].is_object() && arr.len() > best.len() {
                        *best = arr.iter().collect();
                    }
                    for item in arr { find_best_array(item, best); }
                }
                Value::Object(obj) => {
                    for (_, v) in obj { find_best_array(v, best); }
                }
                _ => {}
            }
        }
        find_best_array(json, &mut best);
        return best;
    }
    Vec::new()
}

/// 从单个 prompt 条目中提取文本内容
fn vscode_extract_prompt_content(item: &Value) -> Option<String> {
    for field in &["content", "text", "prompt", "message", "inputText", "outputText", "body", "textContent", "value"] {
        if let Some(val) = item.get(*field) {
            if let Some(s) = val.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
            if let Some(arr) = val.as_array() {
                let parts: Vec<String> = arr.iter().filter_map(|p| {
                    if let Some(s) = p.as_str() { return Some(s.to_string()); }
                    p.get("text").or_else(|| p.get("content")).or_else(|| p.get("value"))
                        .and_then(|v| v.as_str()).map(|s| s.to_string())
                }).collect();
                if !parts.is_empty() { return Some(parts.join("\n")); }
            }
        }
    }
    // value 可能是嵌套 JSON 字符串
    if let Some(v_str) = item.get("value").and_then(|v| v.as_str()) {
        if let Ok(nested) = serde_json::from_str::<Value>(v_str) {
            return vscode_extract_prompt_content(&nested);
        }
    }
    None
}

fn vscode_extract_prompt_role(item: &Value) -> String {
    if let Some(role) = item.get("role").and_then(|v| v.as_str()) {
        return role.to_lowercase();
    }
    if let Some(from) = item.get("from").and_then(|v| v.as_str()) {
        return if from.eq_ignore_ascii_case("user") { "user" } else { "assistant" }.to_string();
    }
    if let Some(is_user) = item.get("isUser").and_then(|v| v.as_bool()) {
        return if is_user { "user" } else { "assistant" }.to_string();
    }
    if item.get("outputText").is_some() { return "assistant".to_string(); }
    "user".to_string()
}

/// 从 VSCode 系平台加载会话消息 — file_path 格式: "type:session_id"，根据 platform 动态查找 db
fn load_vscode_messages(platform: &str, source_path: &str) -> Result<Vec<SessionMessage>, String> {
    let normalized_source_path = normalize_vscode_source_path(source_path);
    let (data_type, rest) = normalized_source_path.split_once(':')
        .ok_or_else(|| format!("无效的会话路径格式: {}", source_path))?;

    // prompts 类型格式: "prompts:aiService.prompts:workspace_hash"
    let (ai_key, session_id) = if data_type == "prompts" {
        if let Some((k, ws)) = rest.split_once(':') { (Some(k), ws) } else { (None, rest) }
    } else {
        (None, rest)
    };

    let app_names = platform_to_app_names(platform);
    if app_names.is_empty() {
        return Err(format!("未知平台: {}", platform));
    }
    let db_paths = get_vscode_db_paths(&app_names);
    if db_paths.is_empty() {
        return Err(format!("未找到 {} 的数据库文件", platform));
    }

    for db_path in &db_paths {
        let Ok(conn) = rusqlite::Connection::open_with_flags(db_path, VSCODE_OPEN_FLAGS) else { continue };

        // 对于 prompts 类型，需要匹配 workspace hash
        if data_type == "prompts" {
            let ws_hash = db_path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if ws_hash != session_id { continue; }
        }

        let result = match data_type {
            "composerData" | "bubble" => load_vscode_composer_msgs(&conn, session_id),
            "chatdata" => load_vscode_chatdata_msgs(&conn, session_id),
            "composer" => load_vscode_composer_table_msgs(&conn, session_id),
            "icube" => load_vscode_icube_msgs(&conn, session_id),
            "icube-history" => load_vscode_icube_history_msgs(&conn),
            "prompts" => load_vscode_prompts_msgs(&conn, ai_key.unwrap_or("aiService.prompts")),
            _ => continue,
        };

        if let Ok(msgs) = &result {
            if !msgs.is_empty() { return result; }
        }
    }

    Err(format!("未找到会话数据: platform={}, path={}", platform, source_path))
}

fn normalize_vscode_source_path(source_path: &str) -> String {
    if source_path.contains(':') {
        return source_path.to_string();
    }

    if let Some(session_id) = source_path.strip_prefix("aicube-history.") {
        return format!("icube-history:{session_id}");
    }
    if let Some(session_id) = source_path.strip_prefix("icube-history.") {
        return format!("icube-history:{session_id}");
    }
    if let Some(session_id) = source_path.strip_prefix("aicube.") {
        return format!("icube:{session_id}");
    }
    if let Some(session_id) = source_path.strip_prefix("icube.") {
        return format!("icube:{session_id}");
    }

    source_path.to_string()
}

fn load_vscode_composer_msgs(conn: &rusqlite::Connection, session_id: &str) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "cursorDiskKV") { return Ok(Vec::new()); }

    let mut messages = Vec::new();

    // 尝试 1: composerData 内联消息
    let key = format!("composerData:{}", session_id);
    if let Ok(value) = conn.query_row("SELECT value FROM cursorDiskKV WHERE key = ?1", [&key], |row| {
        row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
    }) {
        if let Some(json) = vscode_parse_json_bytes(&value) {
            for arr_key in &["conversation", "messages", "bubbles"] {
                if let Some(arr) = json.get(*arr_key).and_then(|v| v.as_array()) {
                    for msg in arr {
                        if let Some(message) = vscode_build_session_message(msg) { messages.push(message); }
                    }
                    if !messages.is_empty() { break; }
                }
            }
            if messages.is_empty() {
                if let Some(headers) = json.get("fullConversationHeadersOnly").and_then(|v| v.as_array()) {
                    let bubble_ids: Vec<String> = headers.iter()
                        .filter_map(|header| header.get("bubbleId").and_then(|v| v.as_str()).map(|s| s.to_string()))
                        .collect();
                    if !bubble_ids.is_empty() {
                        messages = vscode_load_bubble_messages(conn, session_id, Some(&bubble_ids));
                    }
                }
            }
        }
    }
    if !messages.is_empty() { return Ok(messages); }

    // 尝试 2: bubbleId 分离消息
    messages = vscode_load_bubble_messages(conn, session_id, None);
    if !messages.is_empty() { return Ok(messages); }

    // 尝试 3: 直接读取所有与 session_id 相关的 cursorDiskKV 条目
    let fallback_pattern = format!("%{}%", session_id);
    let mut fallback_entries: Vec<(i64, usize, SessionMessage)> = Vec::new();
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE ?1") {
        if let Ok(rows) = stmt.query_map([&fallback_pattern], |row| {
            let key: String = row.get(0)?;
            let value = row.get::<_, Vec<u8>>(1).ok().or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for (idx, row_result) in rows.flatten().enumerate() {
                let (key, value_opt) = row_result;
                if key.starts_with("composerData:") { continue; }
                let Some(vb) = value_opt else { continue };
                if vb.is_empty() { continue; }
                let Some(json) = vscode_parse_json_bytes(&vb) else { continue };
                let Some(message) = vscode_build_session_message(&json) else { continue };
                fallback_entries.push((message.timestamp.unwrap_or(i64::MAX), idx, message));
            }
        }
    }
    fallback_entries.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    messages = fallback_entries.into_iter().map(|(_, _, msg)| msg).collect();

    Ok(messages)
}

fn load_vscode_chatdata_msgs(conn: &rusqlite::Connection, session_id: &str) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "ItemTable") { return Ok(Vec::new()); }
    let Some(vb) = vscode_query_item_table(conn, "workbench.panel.aichat.view.aichat.chatdata") else { return Ok(Vec::new()) };
    let Some(json) = vscode_parse_json_bytes(&vb) else { return Ok(Vec::new()) };
    let mut messages = Vec::new();
    if let Some(tabs) = json.get("tabs").and_then(|v| v.as_array()) {
        for tab in tabs {
            let tab_id = tab.get("id").or_else(|| tab.get("tabId")).and_then(|v| v.as_str()).unwrap_or("");
            if tab_id != session_id { continue; }
            if let Some(bubbles) = tab.get("bubbles").and_then(|v| v.as_array()) {
                for msg in bubbles {
                    if let Some(message) = vscode_build_session_message(msg) {
                        messages.push(message);
                    }
                }
            }
            break;
        }
    }
    Ok(messages)
}

fn load_vscode_composer_table_msgs(conn: &rusqlite::Connection, session_id: &str) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "ItemTable") { return Ok(Vec::new()); }
    let Some(vb) = vscode_query_item_table(conn, "composer.composerData") else { return Ok(Vec::new()) };
    let Some(json) = vscode_parse_json_bytes(&vb) else { return Ok(Vec::new()) };
    let mut messages = Vec::new();
    if let Some(all) = json.get("allComposers").and_then(|a| a.as_array()) {
        for composer in all {
            let cid = composer.get("composerId").or_else(|| composer.get("id")).and_then(|v| v.as_str()).unwrap_or("");
            if cid != session_id { continue; }
            if let Some(conv) = composer.get("conversation").and_then(|c| c.as_array()) {
                for msg in conv {
                    if let Some(message) = vscode_build_session_message(msg) {
                        messages.push(message);
                    }
                }
            }
            break;
        }
    }
    Ok(messages)
}

fn vscode_icube_matches_session(item: &Value, session_id: &str) -> bool {
    [
        "sessionId",
        "id",
        "chatId",
        "conversationId",
        "tabId",
    ]
    .iter()
    .any(|field| item.get(*field).and_then(|v| v.as_str()) == Some(session_id))
}

fn vscode_build_icube_message(item: &Value) -> Option<SessionMessage> {
    let content = item
        .get("inputText")
        .or_else(|| item.get("question"))
        .or_else(|| item.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| vscode_extract_content(item))?;

    let role = if item.get("outputText").is_some() || item.get("answer").is_some() {
        "assistant".to_string()
    } else if item.get("inputText").is_some() || item.get("question").is_some() {
        "user".to_string()
    } else {
        vscode_determine_role(item).unwrap_or_else(|| "user".to_string())
    };

    Some(SessionMessage {
        role,
        content,
        timestamp: vscode_extract_timestamp_ms(item),
    })
}

fn load_vscode_icube_msgs(conn: &rusqlite::Connection, session_id: &str) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "ItemTable") { return Ok(Vec::new()); }
    let Some(vb) = vscode_query_item_table(conn, "memento/icube-ai-agent-storage") else { return Ok(Vec::new()) };
    let Some(json) = vscode_parse_json_bytes(&vb) else { return Ok(Vec::new()) };
    let mut messages = Vec::new();
    if let Some(list) = json.get("list").and_then(|v| v.as_array()) {
        for item in list {
            if !vscode_icube_matches_session(item, session_id) {
                continue;
            }

            for arr_key in &["messages", "conversation", "history", "chatHistory"] {
                if let Some(arr) = item.get(*arr_key).and_then(|v| v.as_array()) {
                    for entry in arr {
                        if let Some(message) = vscode_build_icube_message(entry).or_else(|| vscode_build_session_message(entry)) {
                            messages.push(message);
                        }
                    }
                }
            }

            if messages.is_empty() {
                if let Some(message) = vscode_build_icube_message(item) {
                    messages.push(message);
                }
            }
        }
    }
    Ok(messages)
}

fn load_vscode_icube_history_msgs(conn: &rusqlite::Connection) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "ItemTable") { return Ok(Vec::new()); }
    let Some(vb) = vscode_query_item_table(conn, "icube-ai-agent-storage-input-history") else { return Ok(Vec::new()) };
    let Some(json) = vscode_parse_json_bytes(&vb) else { return Ok(Vec::new()) };
    let mut messages = Vec::new();
    if let Some(arr) = json.as_array() {
        for item in arr {
            if let Some(text) = item.get("inputText").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    messages.push(SessionMessage { role: "user".to_string(), content: text.to_string(), timestamp: None });
                }
            }
        }
    }
    Ok(messages)
}

fn load_vscode_prompts_msgs(conn: &rusqlite::Connection, ai_key: &str) -> Result<Vec<SessionMessage>, String> {
    if !vscode_table_exists(conn, "ItemTable") { return Ok(Vec::new()); }
    let Some(vb) = vscode_query_item_table(conn, ai_key) else { return Ok(Vec::new()) };
    let Some(json) = vscode_parse_json_bytes(&vb) else { return Ok(Vec::new()) };

    let items = vscode_extract_prompt_items(&json);
    let mut messages = Vec::new();
    for item in items {
        // 解析嵌套 JSON 字符串（value 字段可能是 JSON 文本）
        let payload = if let Some(v_str) = item.get("value").and_then(|v| v.as_str()) {
            serde_json::from_str::<Value>(v_str).unwrap_or_else(|_| item.clone())
        } else {
            item.clone()
        };

        let content = match vscode_extract_prompt_content(&payload) {
            Some(c) if !c.is_empty() => c,
            _ => continue,
        };
        let role = vscode_extract_prompt_role(&payload);
        let ts = payload.get("timestamp")
            .or_else(|| payload.get("time"))
            .or_else(|| payload.get("createdAt"))
            .and_then(|v| v.as_i64());
        messages.push(SessionMessage { role, content, timestamp: ts });
    }
    Ok(messages)
}

fn delete_vscode_session(_session_id: &str, _source_path: &str) -> Result<bool, String> {
    Err("VSCode 系平台会话删除暂不支持（数据存储在 SQLite 中，直接删除可能影响应用稳定性）".to_string())
}

// 各 VSCode 平台的扫描入口
fn scan_cursor_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Cursor"], "cursor") }
fn scan_windsurf_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Windsurf", "WindSurf"], "windsurf") }
fn scan_kiro_sessions() -> Vec<SessionInfo> {
    let mut sessions = scan_vscode_sessions(&["Kiro"], "kiro");
    sessions.extend(scan_kiro_json_sessions());
    sessions
}
fn scan_antigravity_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Antigravity"], "antigravity") }
fn scan_codebuddy_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["CodeBuddy"], "codebuddy") }
fn scan_codebuddy_cn_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["CodeBuddy CN", "codebuddy cn", "codebuddy-cn"], "codebuddy_cn") }
fn scan_qoder_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Qoder"], "qoder") }
fn scan_trae_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Trae"], "trae") }
fn scan_workbuddy_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["WorkBuddy"], "workbuddy") }
fn scan_github_copilot_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Code", "Code - Insiders"], "github-copilot") }
fn scan_augment_sessions() -> Vec<SessionInfo> { scan_vscode_sessions(&["Code", "Code - Insiders", "Augment"], "augment") }

// ──────────────────────────────────────────────
//  Kiro JSON 会话文件扫描 (globalStorage)
// ──────────────────────────────────────────────

fn get_kiro_sessions_base() -> Option<PathBuf> {
    for base_dir in get_appdata_dirs() {
        let gs = base_dir.join("Kiro").join("User").join("globalStorage")
            .join("kiro.kiroagent").join("workspace-sessions");
        if gs.exists() { return Some(gs); }
    }
    None
}

fn scan_kiro_json_sessions() -> Vec<SessionInfo> {
    let Some(base) = get_kiro_sessions_base() else { return Vec::new() };
    let mut sessions = Vec::new();

    let Ok(ws_entries) = std::fs::read_dir(&base) else { return Vec::new() };
    for ws_entry in ws_entries.filter_map(|e| e.ok()) {
        let ws_dir = ws_entry.path();
        if !ws_dir.is_dir() { continue; }
        let sessions_json = ws_dir.join("sessions.json");
        if !sessions_json.exists() { continue; }

        let Ok(sessions_data) = std::fs::read_to_string(&sessions_json)
            .map_err(|_| ())
            .and_then(|s| serde_json::from_str::<Value>(&s).map_err(|_| ()))
        else { continue };

        let Some(arr) = sessions_data.as_array() else { continue };
        for session_info in arr {
            let Some(sid) = session_info.get("sessionId").and_then(|v| v.as_str()) else { continue };
            let session_file = ws_dir.join(format!("{}.json", sid));
            if !session_file.exists() { continue; }

            let msg_count = std::fs::read_to_string(&session_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .and_then(|json| json.get("history").and_then(|h| h.as_array()).map(|a| a.len() as u32))
                .unwrap_or(0);

            let title = session_info.get("title").and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some("Kiro Session".to_string()));

            sessions.push(SessionInfo {
                id: sid.to_string(),
                platform: "kiro".to_string(),
                title,
                summary: None,
                working_directory: None,
                created_at: None,
                updated_at: session_file.metadata().ok().and_then(|m| {
                    m.modified().ok().map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                }),
                message_count: msg_count,
                file_path: format!("kiro-json:{}", session_file.to_string_lossy()),
                resume_command: None,
            });
        }
    }
    sessions
}

fn load_kiro_json_messages(session_file: &str) -> Result<Vec<SessionMessage>, String> {
    let content = std::fs::read_to_string(session_file)
        .map_err(|e| format!("读取 Kiro 会话文件失败: {}", e))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("解析 Kiro JSON 失败: {}", e))?;

    let Some(history) = json.get("history").and_then(|h| h.as_array()) else {
        return Ok(Vec::new());
    };

    let mut messages = Vec::new();
    for entry in history {
        let message = entry.get("message").unwrap_or(entry);
        let role = message.get("role").and_then(|v| v.as_str()).unwrap_or("assistant").to_string();
        let content_items = message.get("content").and_then(|v| v.as_array());
        let content = if let Some(items) = content_items {
            items.iter().filter_map(|item| {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) { Some(text.to_string()) }
                else if let Some(s) = item.as_str() { Some(s.to_string()) }
                else { None }
            }).collect::<Vec<_>>().join("\n\n")
        } else if let Some(s) = message.get("content").and_then(|v| v.as_str()) {
            s.to_string()
        } else {
            continue;
        };
        if content.is_empty() { continue; }
        messages.push(SessionMessage { role, content, timestamp: None });
    }
    Ok(messages)
}

// ──────────────────────────────────────────────
//  Warp (SQLite)
// ──────────────────────────────────────────────

fn get_warp_chat_db_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for env_var in &["LOCALAPPDATA", "APPDATA"] {
            if let Ok(base) = std::env::var(env_var) {
                let db = PathBuf::from(&base).join("Warp").join("Warp").join("data").join("warp.sqlite");
                if db.exists() { return Some(db); }
                let db_alt = PathBuf::from(&base).join("Warp").join("data").join("warp.sqlite");
                if db_alt.exists() { return Some(db_alt); }
            }
        }
        if let Some(home) = dirs::home_dir() {
            let db = home.join(".warp").join("warp.sqlite");
            if db.exists() { return Some(db); }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir()?;
        let db = home.join("Library").join("Group Containers").join("2BBY89MBSN.dev.warp")
            .join("Library").join("Application Support").join("dev.warp.Warp-Stable").join("warp.sqlite");
        if db.exists() { return Some(db); }
    }
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir()?;
        let db = home.join(".local").join("share").join("warp").join("warp.sqlite");
        if db.exists() { return Some(db); }
    }
    None
}

fn scan_warp_sessions() -> Vec<SessionInfo> {
    let Some(db_path) = get_warp_chat_db_path() else { return Vec::new() };
    let Ok(conn) = rusqlite::Connection::open_with_flags(&db_path, VSCODE_OPEN_FLAGS) else { return Vec::new() };

    let has_table = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='agent_conversations'")
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
        .is_ok();
    if !has_table { return Vec::new(); }

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, conversation_data FROM agent_conversations WHERE conversation_data IS NOT NULL"
    ) else { return Vec::new() };

    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let data: String = row.get(1)?;
        Ok((id, data))
    }) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let mut sessions = Vec::new();
    for row in rows.flatten() {
        let (conv_id, data_str) = row;
        let Ok(data) = serde_json::from_str::<Value>(&data_str) else { continue };

        let title = data.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
        let created_at = data.get("created_at").and_then(|v| v.as_i64());

        let turns = data.get("turns").and_then(|v| v.as_array());
        let msg_count = turns.map(|t| t.len() as u32).unwrap_or(0);
        if msg_count == 0 { continue; }

        let summary = turns.and_then(|t| t.first())
            .and_then(|turn| turn.get("content").or_else(|| turn.get("text")).and_then(|v| v.as_str()))
            .map(|s| truncate_summary(s, 160));

        sessions.push(SessionInfo {
            id: conv_id.clone(),
            platform: "warp".to_string(),
            title,
            summary,
            working_directory: None,
            created_at,
            updated_at: created_at,
            message_count: msg_count,
            file_path: format!("warp:{}", conv_id),
            resume_command: None,
        });
    }

    sessions
}

fn load_warp_messages(source_path: &str) -> Result<Vec<SessionMessage>, String> {
    let conv_id = source_path.strip_prefix("warp:").unwrap_or(source_path);
    let db_path = get_warp_chat_db_path().ok_or("未找到 Warp 数据库")?;
    let conn = rusqlite::Connection::open_with_flags(&db_path, VSCODE_OPEN_FLAGS)
        .map_err(|e| format!("打开 Warp 数据库失败: {}", e))?;

    let data_str: String = conn.query_row(
        "SELECT conversation_data FROM agent_conversations WHERE id = ?1",
        [conv_id],
        |row| row.get(0),
    ).map_err(|e| format!("查询会话失败: {}", e))?;

    let data: Value = serde_json::from_str(&data_str).map_err(|e| format!("解析会话数据失败: {}", e))?;

    let turns = data.get("turns").and_then(|v| v.as_array()).ok_or("未找到 turns 数据")?;
    let mut messages = Vec::new();

    for turn in turns {
        let role = turn.get("role").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
        let content = turn.get("content").or_else(|| turn.get("text")).and_then(|v| v.as_str()).unwrap_or("");
        if content.is_empty() { continue; }
        messages.push(SessionMessage {
            role,
            content: content.to_string(),
            timestamp: None,
        });
    }

    Ok(messages)
}

fn delete_warp_session(session_id: &str) -> Result<bool, String> {
    let db_path = get_warp_chat_db_path().ok_or("未找到 Warp 数据库")?;
    let conn = rusqlite::Connection::open(&db_path).map_err(|e| format!("打开 Warp 数据库失败: {}", e))?;
    conn.execute("DELETE FROM agent_conversations WHERE id = ?1", [session_id])
        .map_err(|e| format!("删除 Warp 会话失败: {}", e))?;
    Ok(true)
}
