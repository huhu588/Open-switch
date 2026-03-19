//! 对话迁移模块
//!
//! 从 Cursor、Claude Code、Codex、Windsurf、Trae(海外版)、Trae CN 提取完整对话历史，
//! 导出为 JSONL 文件，并支持在另一台电脑导入（自动去重）。
//! 参考 https://github.com/0xSero/ai-data-extraction

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tauri::Emitter;

// ============================================================================
// 数据结构
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSourceInfo {
    pub name: String,
    pub key: String,
    pub path: Option<String>,
    pub conversation_count: u32,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatScanResult {
    pub sources: Vec<ChatSourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedConversation {
    pub messages: Vec<ExtractedMessage>,
    pub source: String,
    /// 唯一标识符，用于去重（source + session_id 组合唯一）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionResult {
    pub source: String,
    pub conversations: Vec<ExtractedConversation>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub exported: u32,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationImportResult {
    pub imported: u32,
    pub skipped: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrationProgress {
    pub phase: String,
    pub source: String,
    pub current: u32,
    pub total: u32,
    pub message: String,
}

fn emit_progress(window: &tauri::Window, phase: &str, source: &str, current: u32, total: u32, message: &str) {
    let _ = window.emit("chat-migration-progress", MigrationProgress {
        phase: phase.to_string(), source: source.to_string(), current, total, message: message.to_string(),
    });
}

// ============================================================================
// 数据源路径
// ============================================================================

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

/// 通用：扫描 VSCode 风格的数据库路径
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

fn get_cursor_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Cursor"]) }
fn get_windsurf_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Windsurf", "WindSurf"]) }
fn get_trae_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Trae"]) }
fn get_trae_cn_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Trae CN", "TraeCN", "trae-cn"]) }
fn get_kiro_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Kiro"]) }
fn get_antigravity_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Antigravity"]) }
fn get_augment_paths() -> Vec<PathBuf> { get_vscode_db_paths(&["Code", "Code - Insiders"]) }

/// 获取 Warp 数据库路径
fn get_warp_chat_db_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .unwrap_or_else(|_| home.join("AppData").join("Local").to_string_lossy().to_string());
        let db = PathBuf::from(&local_app_data).join("Warp").join("Warp").join("data").join("warp.sqlite");
        if db.exists() { return Some(db); }
    }
    #[cfg(target_os = "macos")]
    {
        let db = home.join("Library").join("Group Containers").join("2BBY89MBSN.dev.warp")
            .join("Library").join("Application Support").join("dev.warp.Warp-Stable").join("warp.sqlite");
        if db.exists() { return Some(db); }
    }
    #[cfg(target_os = "linux")]
    {
        let db = home.join(".local").join("share").join("warp").join("warp.sqlite");
        if db.exists() { return Some(db); }
    }
    None
}

/// 从 Warp 数据库提取对话
fn extract_warp_conversations() -> Vec<ExtractedConversation> {
    let mut conversations = Vec::new();
    let Some(db_path) = get_warp_chat_db_path() else { return conversations };
    let Ok(conn) = rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { return conversations };

    // 检查表是否存在
    let has_table = conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='agent_conversations'")
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
        .is_ok();
    if !has_table { return conversations; }

    let Ok(mut stmt) = conn.prepare(
        "SELECT id, conversation_data FROM agent_conversations WHERE conversation_data IS NOT NULL"
    ) else { return conversations };

    let rows = match stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let data: String = row.get(1)?;
        Ok((id, data))
    }) {
        Ok(r) => r,
        Err(_) => return conversations,
    };

    for row in rows.flatten() {
        let (conv_id, data_str) = row;
        let Ok(data) = serde_json::from_str::<serde_json::Value>(&data_str) else { continue };

        let mut messages = Vec::new();

        // 提取对话内容
        if let Some(turns) = data.get("turns").and_then(|v| v.as_array()) {
            for turn in turns {
                let role = turn.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                let content = turn.get("content").and_then(|v| v.as_str())
                    .or_else(|| turn.get("text").and_then(|v| v.as_str()))
                    .unwrap_or("");
                if content.is_empty() { continue; }
                messages.push(ExtractedMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                    model: turn.get("model").and_then(|v| v.as_str()).map(String::from),
                    timestamp: None,
                    tool_use: None,
                });
            }
        }

        if messages.is_empty() { continue; }

        let name = data.get("title").and_then(|v| v.as_str()).map(String::from);
        let created_at = data.get("created_at").and_then(|v| v.as_i64());

        conversations.push(ExtractedConversation {
            messages,
            source: "warp".to_string(),
            session_id: Some(conv_id),
            name,
            created_at,
        });
    }

    conversations
}

/// 统计 Warp 对话数量
fn count_warp_conversations_chat() -> u32 {
    let Some(db_path) = get_warp_chat_db_path() else { return 0 };
    let Ok(conn) = rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { return 0 };
    conn.query_row(
        "SELECT COUNT(*) FROM agent_conversations WHERE conversation_data IS NOT NULL",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) as u32
}

fn get_claude_paths() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Some(home) = dirs::home_dir() else { return files };
    for dir in &[home.join(".claude").join("projects"), home.join(".claude-code")] {
        if !dir.exists() { continue; }
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub) = fs::read_dir(&path) {
                        for se in sub.flatten() {
                            let fp = se.path();
                            if fp.extension().map_or(false, |ext| ext == "jsonl") { files.push(fp); }
                        }
                    }
                }
            }
        }
    }
    files
}

fn get_codex_paths() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Some(home) = dirs::home_dir() else { return files };
    let codex_dir = home.join(".codex");
    if !codex_dir.exists() { return files; }
    fn scan(dir: &PathBuf, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() { scan(&p, files); }
                else if p.extension().map_or(false, |ext| ext == "jsonl") { files.push(p); }
            }
        }
    }
    scan(&codex_dir, &mut files);
    files
}

// ============================================================================
// VSCode 风格 SQLite 对话提取（Cursor / Windsurf / Trae 共用）
// ============================================================================

fn extract_vscode_conversations(db_paths: &[PathBuf], source_name: &str) -> Vec<ExtractedConversation> {
    let mut conversations = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();
    for db_path in db_paths {
        let Ok(conn) = rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { continue };
        extract_from_cursor_disk_kv(&conn, source_name, &mut conversations, &mut seen_ids);
        extract_from_item_table_chat(&conn, source_name, &mut conversations, &mut seen_ids);
        extract_from_item_table_composer(&conn, source_name, &mut conversations, &mut seen_ids);
        // Trae/Trae CN/Windsurf: 提取 icube 输入历史和会话数据
        extract_from_trae_icube(&conn, source_name, &mut conversations, &mut seen_ids);
    }
    conversations
}

/// 从 Trae/Trae CN 的 icube 存储中提取用户输入历史
fn extract_from_trae_icube(
    conn: &rusqlite::Connection,
    source_name: &str,
    conversations: &mut Vec<ExtractedConversation>,
    seen_ids: &mut HashSet<String>,
) {
    if !table_exists(conn, "ItemTable") { return; }

    // 获取会话 ID 列表
    let mut session_ids: Vec<String> = Vec::new();
    if let Some(vb) = query_item_table(conn, "memento/icube-ai-agent-storage") {
        if let Some(json) = parse_json_bytes(&vb) {
            if let Some(list) = json.get("list").and_then(|v| v.as_array()) {
                for item in list {
                    if let Some(sid) = item.get("sessionId").and_then(|v| v.as_str()) {
                        if !sid.is_empty() && !seen_ids.contains(sid) {
                            session_ids.push(sid.to_string());
                        }
                    }
                }
            }
        }
    }

    // 提取用户输入历史
    if let Some(vb) = query_item_table(conn, "icube-ai-agent-storage-input-history") {
        if let Some(json) = parse_json_bytes(&vb) {
            if let Some(arr) = json.as_array() {
                let mut messages = Vec::new();
                for item in arr {
                    // inputText 字段包含用户输入
                    if let Some(input_text) = item.get("inputText").and_then(|v| v.as_str()) {
                        if !input_text.is_empty() {
                            messages.push(ExtractedMessage {
                                role: "user".to_string(),
                                content: input_text.to_string(),
                                model: None,
                                timestamp: None,
                                tool_use: None,
                            });
                        }
                    }
                }
                if !messages.is_empty() {
                    let session_id = session_ids.first().cloned()
                        .unwrap_or_else(|| format!("{}-input-history", source_name));
                    if !seen_ids.contains(&session_id) {
                        seen_ids.insert(session_id.clone());
                        conversations.push(ExtractedConversation {
                            messages,
                            source: format!("{}-input-history", source_name),
                            session_id: Some(session_id),
                            name: Some("Input History".to_string()),
                            created_at: None,
                        });
                    }
                }
            }
        }
    }

    // 按工作区分别提取（每个工作区可能有不同的输入历史）
    // 通过前缀匹配所有 icube-ai-agent-storage-input-history 键
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM ItemTable WHERE key = 'icube-ai-agent-storage-input-history-query'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = row.get::<_, Vec<u8>>(1).ok().or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for row_result in rows.flatten() {
                let (_key, value_opt) = row_result;
                let Some(vb) = value_opt else { continue };
                let Some(json) = parse_json_bytes(&vb) else { continue };
                if let Some(arr) = json.as_array() {
                    for item in arr {
                        if let Some(text) = item.get("text").or_else(|| item.get("inputText")).and_then(|v| v.as_str()) {
                            if !text.is_empty() && text.len() > 5 {
                                // 这些会被包含在上面的主提取中，跳过以避免重复
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_from_cursor_disk_kv(conn: &rusqlite::Connection, source_name: &str, conversations: &mut Vec<ExtractedConversation>, seen_ids: &mut HashSet<String>) {
    if !table_exists(conn, "cursorDiskKV") { return; }

    let mut inline_ids: HashSet<String> = HashSet::new();

    // 阶段1: composerData inline 消息
    if let Ok(mut stmt) = conn.prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'") {
        if let Ok(rows) = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value = row.get::<_, Vec<u8>>(1).ok().or_else(|| row.get::<_, String>(1).ok().map(|s| s.into_bytes()));
            Ok((key, value))
        }) {
            for row_result in rows.flatten() {
                let (key, value_opt) = row_result;
                let Some(value_bytes) = value_opt else { continue };
                let Some(json) = parse_json_bytes(&value_bytes) else { continue };

                let composer_id = json.get("composerId").and_then(|v| v.as_str()).map(|s| s.to_string())
                    .unwrap_or_else(|| key.strip_prefix("composerData:").unwrap_or(&key).to_string());
                if seen_ids.contains(&composer_id) { continue; }

                let name = get_str_field(&json, &["name", "title"]);
                let created_at = json.get("createdAt").or_else(|| json.get("created_at")).and_then(|v| v.as_i64());

                let mut messages = Vec::new();
                // inline conversation
                if let Some(conv) = json.get("conversation").or_else(|| json.get("messages")).and_then(|v| v.as_array()) {
                    for msg in conv { if let Some(e) = parse_vscode_message(msg) { messages.push(e); } }
                }
                if messages.is_empty() {
                    if let Some(bubbles) = json.get("bubbles").and_then(|v| v.as_array()) {
                        for b in bubbles { if let Some(e) = parse_bubble_message(b) { messages.push(e); } }
                    }
                }
                if !messages.is_empty() {
                    inline_ids.insert(composer_id.clone());
                    seen_ids.insert(composer_id.clone());
                    conversations.push(ExtractedConversation {
                        messages, source: format!("{}-composer", source_name), name, created_at, session_id: Some(composer_id),
                    });
                }
            }
        }
    }

    // 阶段2: bubbleId 分离消息
    let mut bubble_composer_ids: HashSet<String> = HashSet::new();
    if let Ok(mut stmt) = conn.prepare("SELECT key FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'") {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for key_result in rows.flatten() {
                let parts: Vec<&str> = key_result.split(':').collect();
                if parts.len() >= 3 {
                    let cid = parts[1].to_string();
                    if !inline_ids.contains(&cid) && !seen_ids.contains(&cid) { bubble_composer_ids.insert(cid); }
                }
            }
        }
    }

    for composer_id in bubble_composer_ids {
        if seen_ids.contains(&composer_id) { continue; }
        seen_ids.insert(composer_id.clone());
        let key_pattern = format!("bubbleId:{}:%", composer_id);
        let mut messages = Vec::new();
        if let Ok(mut stmt) = conn.prepare("SELECT value FROM cursorDiskKV WHERE key LIKE ?1") {
            if let Ok(rows) = stmt.query_map([&key_pattern], |row| {
                let value = row.get::<_, Vec<u8>>(0).ok().or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes()));
                Ok(value)
            }) {
                for row_result in rows.flatten() {
                    let Some(vb) = row_result else { continue };
                    let Some(json) = parse_json_bytes(&vb) else { continue };
                    if let Some(e) = parse_bubble_message(&json) { messages.push(e); }
                }
            }
        }
        if !messages.is_empty() {
            let (name, created_at) = get_composer_meta(conn, &composer_id);
            conversations.push(ExtractedConversation {
                messages, source: format!("{}-composer", source_name), name, created_at, session_id: Some(composer_id),
            });
        }
    }
}

fn get_composer_meta(conn: &rusqlite::Connection, composer_id: &str) -> (Option<String>, Option<i64>) {
    let key = format!("composerData:{}", composer_id);
    if let Ok(value) = conn.query_row("SELECT value FROM cursorDiskKV WHERE key = ?1", [&key], |row| {
        row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
    }) {
        if let Some(json) = parse_json_bytes(&value) {
            return (get_str_field(&json, &["name", "title"]), json.get("createdAt").and_then(|v| v.as_i64()));
        }
    }
    (None, None)
}

fn extract_from_item_table_chat(conn: &rusqlite::Connection, source_name: &str, conversations: &mut Vec<ExtractedConversation>, seen_ids: &mut HashSet<String>) {
    if !table_exists(conn, "ItemTable") { return; }
    let Some(vb) = query_item_table(conn, "workbench.panel.aichat.view.aichat.chatdata") else { return };
    let Some(json) = parse_json_bytes(&vb) else { return };
    if let Some(tabs) = json.get("tabs").and_then(|v| v.as_array()) {
        for tab in tabs {
            let tab_id = tab.get("id").or_else(|| tab.get("tabId")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !tab_id.is_empty() && seen_ids.contains(&tab_id) { continue; }
            if !tab_id.is_empty() { seen_ids.insert(tab_id.clone()); }
            let name = get_str_field(tab, &["chatTitle", "title"]);
            let created_at = tab.get("createdAt").or_else(|| tab.get("timestamp")).and_then(|v| v.as_i64());
            let mut messages = Vec::new();
            if let Some(bubbles) = tab.get("bubbles").and_then(|v| v.as_array()) {
                for b in bubbles { if let Some(e) = parse_bubble_message(b) { messages.push(e); } }
            }
            if !messages.is_empty() {
                conversations.push(ExtractedConversation {
                    messages, source: format!("{}-chat", source_name), name, created_at,
                    session_id: if tab_id.is_empty() { None } else { Some(tab_id) },
                });
            }
        }
    }
}

fn extract_from_item_table_composer(conn: &rusqlite::Connection, source_name: &str, conversations: &mut Vec<ExtractedConversation>, seen_ids: &mut HashSet<String>) {
    if !table_exists(conn, "ItemTable") { return; }
    let Some(vb) = query_item_table(conn, "composer.composerData") else { return };
    let Some(json) = parse_json_bytes(&vb) else { return };
    if let Some(all) = json.get("allComposers").and_then(|a| a.as_array()) {
        for composer in all {
            let cid = composer.get("composerId").or_else(|| composer.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !cid.is_empty() && seen_ids.contains(&cid) { continue; }
            if !cid.is_empty() { seen_ids.insert(cid.clone()); }
            let name = get_str_field(composer, &["name", "title"]);
            let created_at = composer.get("createdAt").and_then(|v| v.as_i64());
            let mut messages = Vec::new();
            if let Some(conv) = composer.get("conversation").and_then(|c| c.as_array()) {
                for msg in conv { if let Some(e) = parse_vscode_message(msg) { messages.push(e); } }
            }
            if !messages.is_empty() {
                conversations.push(ExtractedConversation {
                    messages, source: format!("{}-composer", source_name), name, created_at,
                    session_id: if cid.is_empty() { None } else { Some(cid) },
                });
            }
        }
    }
}

// ============================================================================
// 消息解析
// ============================================================================

fn parse_vscode_message(msg: &serde_json::Value) -> Option<ExtractedMessage> {
    let role = determine_role(msg)?;
    let content = extract_content_text(msg)?;
    if content.is_empty() { return None; }
    Some(ExtractedMessage {
        role, content,
        model: get_str_field(msg, &["model", "modelId", "modelName"]),
        timestamp: extract_timestamp(msg),
        tool_use: msg.get("toolCalls").or_else(|| msg.get("toolResults")).or_else(|| msg.get("capabilityResults")).cloned(),
    })
}

fn parse_bubble_message(bubble: &serde_json::Value) -> Option<ExtractedMessage> {
    let role = determine_role(bubble).unwrap_or_else(|| "assistant".to_string());
    let content = extract_content_text(bubble)?;
    if content.is_empty() { return None; }
    Some(ExtractedMessage {
        role, content,
        model: get_str_field(bubble, &["model", "modelId", "modelName"]),
        timestamp: extract_timestamp(bubble),
        tool_use: bubble.get("toolCalls").or_else(|| bubble.get("toolResults")).or_else(|| bubble.get("capabilityResults")).cloned(),
    })
}

fn determine_role(msg: &serde_json::Value) -> Option<String> {
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

fn extract_timestamp(msg: &serde_json::Value) -> Option<String> {
    msg.get("timestamp").or_else(|| msg.get("createdAt")).or_else(|| msg.get("created_at")).and_then(|v| {
        if let Some(ts) = v.as_i64() { Some(ts.to_string()) }
        else if let Some(ts) = v.as_f64() { Some((ts as i64).to_string()) }
        else { v.as_str().map(|s| s.to_string()) }
    })
}

/// 内容提取：rawText > text > content > message > value > richText > codeBlocks > 最长字符串
fn extract_content_text(msg: &serde_json::Value) -> Option<String> {
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
    // richText
    if let Some(rich) = msg.get("richText").or_else(|| msg.get("richtext")) {
        if let Some(s) = rich.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
        if let Some(arr) = rich.as_array() {
            let parts: Vec<String> = arr.iter().filter_map(|p| {
                if let Some(s) = p.as_str() { Some(s.to_string()) }
                else { p.get("text").or_else(|| p.get("value")).and_then(|t| t.as_str()).map(|s| s.to_string()) }
            }).collect();
            if !parts.is_empty() { return Some(parts.join("")); }
        }
    }
    // codeBlocks
    for field in &["codeBlocks", "suggestedCodeBlocks"] {
        if let Some(blocks) = msg.get(*field).and_then(|b| b.as_array()) {
            let parts: Vec<String> = blocks.iter().filter_map(|b| {
                b.get("code").or_else(|| b.get("text")).or_else(|| b.get("content")).and_then(|c| c.as_str()).map(|s| s.to_string())
            }).collect();
            if !parts.is_empty() { return Some(parts.join("\n\n")); }
        }
    }
    // 最后手段：最长字符串
    if let Some(obj) = msg.as_object() {
        let mut best: Option<String> = None;
        let mut best_len = 0usize;
        for (k, v) in obj {
            if matches!(k.as_str(), "type"|"role"|"sender"|"isUser"|"model"|"modelId"|"modelName"|"timestamp"|"createdAt"|"id"|"bubbleId"|"composerId"|"key"|"sessionId"|"tabId") { continue; }
            if let Some(s) = v.as_str() { if s.len() > best_len && s.len() >= 5 { best_len = s.len(); best = Some(s.to_string()); } }
        }
        if best.is_some() { return best; }
    }
    None
}

// ============================================================================
// Claude Code 对话提取
// ============================================================================

fn extract_claude_conversations(files: &[PathBuf]) -> Vec<ExtractedConversation> {
    let mut conversations = Vec::new();
    for file in files {
        let Ok(content) = fs::read_to_string(file) else { continue };
        let project_name = file.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(|s| s.to_string());
        let session_id = file.file_stem().and_then(|n| n.to_str()).map(|s| s.to_string());
        let mut messages = Vec::new();
        for line in content.lines() {
            let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) else { continue };
            let role = match msg_type {
                "human" | "user" => "user",
                "assistant" => "assistant",
                _ => continue,
            };
            let content_text = json.get("message").and_then(|m| extract_claude_content(m))
                .or_else(|| extract_claude_content(&json));
            let Some(ct) = content_text else { continue };
            if ct.is_empty() { continue; }
            messages.push(ExtractedMessage {
                role: role.to_string(), content: ct,
                model: json.get("message").and_then(|m| m.get("model")).or_else(|| json.get("model")).and_then(|v| v.as_str()).map(|s| s.to_string()),
                timestamp: json.get("timestamp").and_then(|v| v.as_str().map(|s| s.to_string()).or_else(|| v.as_i64().map(|n| n.to_string()))),
                tool_use: json.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_array()).and_then(|arr| {
                    let tools: Vec<&serde_json::Value> = arr.iter().filter(|i| i.get("type").and_then(|t| t.as_str()) == Some("tool_use")).collect();
                    if tools.is_empty() { None } else { Some(serde_json::json!(tools)) }
                }),
            });
        }
        if !messages.is_empty() {
            let created_at = messages.first().and_then(|m| m.timestamp.as_ref()).and_then(|ts| {
                ts.parse::<i64>().ok().or_else(|| chrono::DateTime::parse_from_rfc3339(ts).ok().map(|dt| dt.timestamp_millis()))
            });
            conversations.push(ExtractedConversation { messages, source: "claude-code".to_string(), name: project_name, created_at, session_id });
        }
    }
    conversations
}

fn extract_claude_content(msg: &serde_json::Value) -> Option<String> {
    if let Some(content) = msg.get("content") {
        if let Some(s) = content.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
        if let Some(arr) = content.as_array() {
            let parts: Vec<String> = arr.iter().filter_map(|part| {
                match part.get("type").and_then(|t| t.as_str()).unwrap_or("") {
                    "text" => part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()),
                    "tool_use" => Some(format!("[tool_use: {}]", part.get("name").and_then(|n| n.as_str()).unwrap_or("tool"))),
                    "tool_result" => part.get("content").and_then(|c| {
                        if let Some(s) = c.as_str() { Some(s.to_string()) }
                        else if let Some(a) = c.as_array() { Some(a.iter().filter_map(|p| p.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())).collect::<Vec<_>>().join("\n")) }
                        else { None }
                    }),
                    _ => part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()),
                }
            }).collect();
            if !parts.is_empty() { return Some(parts.join("\n")); }
        }
    }
    msg.get("text").and_then(|t| t.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string())
}

// ============================================================================
// Codex 对话提取
// ============================================================================

fn extract_codex_conversations(files: &[PathBuf]) -> Vec<ExtractedConversation> {
    let mut conversations = Vec::new();
    for file in files {
        let Ok(content) = fs::read_to_string(file) else { continue };
        let session_id = file.file_stem().and_then(|n| n.to_str()).map(|s| s.to_string());
        let mut messages = Vec::new();
        let mut current_model: Option<String> = None;

        for line in content.lines() {
            let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else { continue };
            let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match msg_type {
                // 模型上下文
                "turn_context" => {
                    if let Some(m) = json.get("payload").and_then(|p| p.get("model")).and_then(|v| v.as_str()) {
                        current_model = Some(m.to_string());
                    }
                }
                // 直接消息类型
                "message" => {
                    let role = json.get("role").and_then(|v| v.as_str()).unwrap_or("unknown");
                    if role == "user" || role == "assistant" || role == "agent" || role == "system" {
                        if let Some(ct) = extract_codex_content(&json) {
                            if !ct.is_empty() {
                                messages.push(ExtractedMessage {
                                    role: if role == "agent" { "assistant".to_string() } else { role.to_string() },
                                    content: ct,
                                    model: json.get("model").and_then(|v| v.as_str()).map(|s| s.to_string()).or_else(|| current_model.clone()),
                                    timestamp: extract_timestamp(&json),
                                    tool_use: None,
                                });
                            }
                        }
                    }
                }
                // 事件消息中的实际内容
                "event_msg" => {
                    if let Some(payload) = json.get("payload") {
                        let pt = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match pt {
                            "input_text" | "user_message" => {
                                if let Some(ct) = payload.get("text").or_else(|| payload.get("content")).and_then(|v| v.as_str()) {
                                    if !ct.is_empty() {
                                        messages.push(ExtractedMessage {
                                            role: "user".to_string(), content: ct.to_string(),
                                            model: current_model.clone(), timestamp: extract_timestamp(&json), tool_use: None,
                                        });
                                    }
                                }
                            }
                            "output_text" | "assistant_message" | "response" => {
                                if let Some(ct) = payload.get("text").or_else(|| payload.get("content")).and_then(|v| v.as_str()) {
                                    if !ct.is_empty() {
                                        messages.push(ExtractedMessage {
                                            role: "assistant".to_string(), content: ct.to_string(),
                                            model: current_model.clone(), timestamp: extract_timestamp(&json), tool_use: None,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // 顶级 role 字段（兼容老格式）
                _ => {
                    if let Some(role) = json.get("role").and_then(|v| v.as_str()) {
                        if role == "user" || role == "assistant" || role == "agent" {
                            if let Some(ct) = extract_codex_content(&json) {
                                if !ct.is_empty() {
                                    messages.push(ExtractedMessage {
                                        role: if role == "agent" { "assistant".to_string() } else { role.to_string() },
                                        content: ct, model: current_model.clone(), timestamp: extract_timestamp(&json), tool_use: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        if !messages.is_empty() {
            conversations.push(ExtractedConversation { messages, source: "codex".to_string(), name: None, created_at: None, session_id });
        }
    }
    conversations
}

fn extract_codex_content(json: &serde_json::Value) -> Option<String> {
    // content 字段 (字符串或数组)
    if let Some(val) = json.get("content") {
        if let Some(s) = val.as_str() { if !s.is_empty() { return Some(s.to_string()); } }
        if let Some(arr) = val.as_array() {
            let parts: Vec<String> = arr.iter().filter_map(|p| {
                if let Some(s) = p.as_str() { return Some(s.to_string()); }
                p.get("text").or_else(|| p.get("output_text")).or_else(|| p.get("value"))
                    .and_then(|t| t.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string())
            }).collect();
            if !parts.is_empty() { return Some(parts.join("\n")); }
        }
    }
    // text 字段
    json.get("text").and_then(|t| t.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string())
}

// ============================================================================
// 辅助函数
// ============================================================================

fn parse_json_bytes(bytes: &[u8]) -> Option<serde_json::Value> {
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) { return Some(json); }
    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
        let trimmed = text.trim_matches('\u{0}').trim();
        if !trimmed.is_empty() { return serde_json::from_str::<serde_json::Value>(trimmed).ok(); }
    }
    None
}

fn table_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    conn.prepare(&format!("SELECT name FROM sqlite_master WHERE type='table' AND name='{}'", name))
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(true))).unwrap_or(false)
}

fn query_item_table(conn: &rusqlite::Connection, key: &str) -> Option<Vec<u8>> {
    conn.query_row("SELECT value FROM ItemTable WHERE key = ?1", [key], |row| {
        row.get::<_, Vec<u8>>(0).ok().or_else(|| row.get::<_, String>(0).ok().map(|s| s.into_bytes())).ok_or(rusqlite::Error::QueryReturnedNoRows)
    }).ok()
}

fn get_str_field(json: &serde_json::Value, fields: &[&str]) -> Option<String> {
    for f in fields {
        if let Some(s) = json.get(*f).and_then(|v| v.as_str()) { if !s.is_empty() { return Some(s.to_string()); } }
    }
    None
}

fn count_vscode_conversations(db_paths: &[PathBuf]) -> u32 {
    let mut count = 0u32;
    for db_path in db_paths {
        let Ok(conn) = rusqlite::Connection::open_with_flags(db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY) else { continue };
        // Cursor: composerData 数量
        if let Ok(n) = conn.query_row("SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%'", [], |row| row.get::<_, u32>(0)) { count += n; }
        // Cursor: bubbleId 中不同 composer 数量（补充无 inline 的）
        if let Ok(n) = conn.query_row("SELECT COUNT(DISTINCT substr(key, 10, instr(substr(key, 10), ':')-1)) FROM cursorDiskKV WHERE key LIKE 'bubbleId:%'", [], |row| row.get::<_, u32>(0)) { count += n; }
        // ItemTable chat tabs
        if let Ok(value) = conn.query_row("SELECT value FROM ItemTable WHERE key = 'workbench.panel.aichat.view.aichat.chatdata'", [], |row| {
            row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
        }) {
            if let Some(json) = parse_json_bytes(&value) {
                if let Some(tabs) = json.get("tabs").and_then(|v| v.as_array()) { count += tabs.len() as u32; }
            }
        }
        // ItemTable workspace composers
        if let Ok(value) = conn.query_row("SELECT value FROM ItemTable WHERE key = 'composer.composerData'", [], |row| {
            row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
        }) {
            if let Some(json) = parse_json_bytes(&value) {
                if let Some(all) = json.get("allComposers").and_then(|a| a.as_array()) { count += all.len() as u32; }
            }
        }
        // Trae/Trae CN: icube 输入历史条目数
        if let Ok(value) = conn.query_row("SELECT value FROM ItemTable WHERE key = 'icube-ai-agent-storage-input-history'", [], |row| {
            row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
        }) {
            if let Some(json) = parse_json_bytes(&value) {
                if let Some(arr) = json.as_array() {
                    let input_count = arr.iter().filter(|item| {
                        item.get("inputText").and_then(|v| v.as_str()).map(|s| !s.is_empty()).unwrap_or(false)
                    }).count() as u32;
                    if input_count > 0 && count == 0 { count += 1; } // 算作1个对话（输入历史集合）
                }
            }
        }
        // Trae/Trae CN: icube 会话数
        if let Ok(value) = conn.query_row("SELECT value FROM ItemTable WHERE key = 'memento/icube-ai-agent-storage'", [], |row| {
            row.get::<_, Vec<u8>>(0).or_else(|_| row.get::<_, String>(0).map(|s| s.into_bytes()))
        }) {
            if let Some(json) = parse_json_bytes(&value) {
                if let Some(list) = json.get("list").and_then(|v| v.as_array()) {
                    let session_count = list.len() as u32;
                    if session_count > 0 && count == 0 { count += session_count; }
                }
            }
        }
    }
    count
}

/// 生成去重 key：source + session_id + 首条消息内容前100字符的哈希
fn dedup_key(conv: &ExtractedConversation) -> String {
    let first_content = conv.messages.first().map(|m| &m.content[..m.content.len().min(100)]).unwrap_or("");
    let session = conv.session_id.as_deref().unwrap_or("");
    format!("{}|{}|{:x}", conv.source, session, simple_hash(first_content))
}

fn simple_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() { h = h.wrapping_mul(33).wrapping_add(b as u64); }
    h
}

// ============================================================================
// Tauri 命令
// ============================================================================

#[tauri::command]
pub async fn scan_chat_sources(window: tauri::Window) -> Result<ChatScanResult, String> {
    let mut sources = Vec::new();
    let total_steps = 11u32;

    // 1. Cursor
    emit_progress(&window, "scan", "cursor", 1, total_steps, "扫描 Cursor");
    let cursor_paths = get_cursor_paths();
    sources.push(ChatSourceInfo {
        name: "Cursor".to_string(), key: "cursor".to_string(),
        path: cursor_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !cursor_paths.is_empty() { count_vscode_conversations(&cursor_paths) } else { 0 },
        available: !cursor_paths.is_empty(),
    });

    // 2. Claude Code
    emit_progress(&window, "scan", "claude", 2, total_steps, "扫描 Claude Code");
    let claude_files = get_claude_paths();
    sources.push(ChatSourceInfo {
        name: "Claude Code".to_string(), key: "claude".to_string(),
        path: dirs::home_dir().map(|h| h.join(".claude").join("projects").to_string_lossy().to_string()),
        conversation_count: claude_files.len() as u32,
        available: !claude_files.is_empty(),
    });

    // 3. Codex
    emit_progress(&window, "scan", "codex", 3, total_steps, "扫描 Codex");
    let codex_files = get_codex_paths();
    sources.push(ChatSourceInfo {
        name: "Codex".to_string(), key: "codex".to_string(),
        path: dirs::home_dir().map(|h| h.join(".codex").to_string_lossy().to_string()),
        conversation_count: codex_files.len() as u32,
        available: !codex_files.is_empty(),
    });

    // 4. Windsurf
    emit_progress(&window, "scan", "windsurf", 4, total_steps, "扫描 Windsurf");
    let windsurf_paths = get_windsurf_paths();
    sources.push(ChatSourceInfo {
        name: "Windsurf".to_string(), key: "windsurf".to_string(),
        path: windsurf_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !windsurf_paths.is_empty() { count_vscode_conversations(&windsurf_paths) } else { 0 },
        available: !windsurf_paths.is_empty(),
    });

    // 5. Trae (海外版)
    emit_progress(&window, "scan", "trae", 5, total_steps, "扫描 Trae");
    let trae_paths = get_trae_paths();
    sources.push(ChatSourceInfo {
        name: "Trae".to_string(), key: "trae".to_string(),
        path: trae_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !trae_paths.is_empty() { count_vscode_conversations(&trae_paths) } else { 0 },
        available: !trae_paths.is_empty(),
    });

    // 6. Trae CN (国内版)
    emit_progress(&window, "scan", "trae_cn", 6, total_steps, "扫描 Trae CN");
    let trae_cn_paths = get_trae_cn_paths();
    sources.push(ChatSourceInfo {
        name: "Trae CN".to_string(), key: "trae_cn".to_string(),
        path: trae_cn_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !trae_cn_paths.is_empty() { count_vscode_conversations(&trae_cn_paths) } else { 0 },
        available: !trae_cn_paths.is_empty(),
    });

    // 7. Kiro
    emit_progress(&window, "scan", "kiro", 7, total_steps, "扫描 Kiro");
    let kiro_paths = get_kiro_paths();
    sources.push(ChatSourceInfo {
        name: "Kiro".to_string(), key: "kiro".to_string(),
        path: kiro_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !kiro_paths.is_empty() { count_vscode_conversations(&kiro_paths) } else { 0 },
        available: !kiro_paths.is_empty(),
    });

    // 8. Antigravity
    emit_progress(&window, "scan", "antigravity", 8, total_steps, "扫描 Antigravity");
    let antigravity_paths = get_antigravity_paths();
    sources.push(ChatSourceInfo {
        name: "Antigravity".to_string(), key: "antigravity".to_string(),
        path: antigravity_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !antigravity_paths.is_empty() { count_vscode_conversations(&antigravity_paths) } else { 0 },
        available: !antigravity_paths.is_empty(),
    });

    // 9. Warp
    emit_progress(&window, "scan", "warp", 9, total_steps, "扫描 Warp");
    let warp_available = get_warp_chat_db_path().is_some();
    sources.push(ChatSourceInfo {
        name: "Warp".to_string(), key: "warp".to_string(),
        path: get_warp_chat_db_path().map(|p| p.to_string_lossy().to_string()),
        conversation_count: if warp_available { count_warp_conversations_chat() } else { 0 },
        available: warp_available,
    });

    // 10. Augment
    emit_progress(&window, "scan", "augment", 10, total_steps, "扫描 Augment");
    let augment_paths = get_augment_paths();
    sources.push(ChatSourceInfo {
        name: "Augment".to_string(), key: "augment".to_string(),
        path: augment_paths.first().and_then(|p| p.parent()).and_then(|p| p.parent()).map(|p| p.to_string_lossy().to_string()),
        conversation_count: if !augment_paths.is_empty() { count_vscode_conversations(&augment_paths) } else { 0 },
        available: !augment_paths.is_empty(),
    });

    // 11. 完成
    emit_progress(&window, "scan", "done", total_steps, total_steps, "扫描完成");
    Ok(ChatScanResult { sources })
}

#[tauri::command]
pub async fn extract_conversations(window: tauri::Window, source: String) -> Result<ExtractionResult, String> {
    emit_progress(&window, "extract", &source, 0, 1, &format!("正在提取 {} 对话...", source));
    let conversations = match source.as_str() {
        "cursor"      => extract_vscode_conversations(&get_cursor_paths(), "cursor"),
        "claude"      => extract_claude_conversations(&get_claude_paths()),
        "codex"       => extract_codex_conversations(&get_codex_paths()),
        "windsurf"    => extract_vscode_conversations(&get_windsurf_paths(), "windsurf"),
        "trae"        => extract_vscode_conversations(&get_trae_paths(), "trae"),
        "trae_cn"     => extract_vscode_conversations(&get_trae_cn_paths(), "trae-cn"),
        "kiro"        => extract_vscode_conversations(&get_kiro_paths(), "kiro"),
        "antigravity" => extract_vscode_conversations(&get_antigravity_paths(), "antigravity"),
        "warp"        => extract_warp_conversations(),
        "augment"     => extract_vscode_conversations(&get_augment_paths(), "augment"),
        _ => return Err(format!("不支持的数据源: {}", source)),
    };
    let total = conversations.len() as u32;
    emit_progress(&window, "extract", &source, 1, 1, &format!("提取完成，共 {} 个对话", total));
    Ok(ExtractionResult { source, conversations, total })
}

#[tauri::command]
pub async fn export_conversations(conversations: Vec<ExtractedConversation>, file_path: String) -> Result<ExportResult, String> {
    let mut lines = Vec::new();
    for conv in &conversations {
        match serde_json::to_string(conv) {
            Ok(line) => lines.push(line),
            Err(e) => return Err(format!("序列化失败: {}", e)),
        }
    }
    let content = lines.join("\n");
    fs::write(&file_path, content).map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(ExportResult { exported: conversations.len() as u32, file_path })
}

/// 导入迁移文件（JSONL），自动去重
#[tauri::command]
pub async fn import_migration_file(file_path: String) -> Result<MigrationImportResult, String> {
    let content = fs::read_to_string(&file_path).map_err(|e| format!("读取文件失败: {}", e))?;

    // 读取目标路径（导入到 ~/.ai-switch/migrated_conversations.jsonl）
    let target_path = get_migration_store_path()?;

    // 加载已有数据的去重 key
    let mut existing_keys: HashSet<String> = HashSet::new();
    let mut existing_lines: Vec<String> = Vec::new();
    if target_path.exists() {
        if let Ok(existing) = fs::read_to_string(&target_path) {
            for line in existing.lines() {
                if let Ok(conv) = serde_json::from_str::<ExtractedConversation>(line) {
                    existing_keys.insert(dedup_key(&conv));
                    existing_lines.push(line.to_string());
                }
            }
        }
    }

    let mut imported = 0u32;
    let mut skipped = 0u32;
    let mut total = 0u32;
    let mut new_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() { continue; }
        total += 1;
        match serde_json::from_str::<ExtractedConversation>(line) {
            Ok(conv) => {
                let key = dedup_key(&conv);
                if existing_keys.contains(&key) {
                    skipped += 1;
                } else {
                    existing_keys.insert(key);
                    new_lines.push(line.to_string());
                    imported += 1;
                }
            }
            Err(_) => { skipped += 1; }
        }
    }

    // 追加写入
    if !new_lines.is_empty() {
        existing_lines.extend(new_lines);
        let all_content = existing_lines.join("\n");
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&target_path, all_content).map_err(|e| format!("写入失败: {}", e))?;
    }

    Ok(MigrationImportResult { imported, skipped, total })
}

/// 获取已导入的迁移对话列表
#[tauri::command]
pub async fn get_migrated_conversations() -> Result<Vec<ExtractedConversation>, String> {
    let target_path = get_migration_store_path()?;
    if !target_path.exists() { return Ok(Vec::new()); }
    let content = fs::read_to_string(&target_path).map_err(|e| format!("读取失败: {}", e))?;
    let mut conversations = Vec::new();
    for line in content.lines() {
        if let Ok(conv) = serde_json::from_str::<ExtractedConversation>(line) {
            conversations.push(conv);
        }
    }
    Ok(conversations)
}

/// 清除所有已导入的迁移对话
#[tauri::command]
pub async fn clear_migrated_conversations() -> Result<(), String> {
    let target_path = get_migration_store_path()?;
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| format!("删除失败: {}", e))?;
    }
    Ok(())
}

fn get_migration_store_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户目录")?;
    Ok(home.join(".ai-switch").join("migrated_conversations.jsonl"))
}
