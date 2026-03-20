use crate::modules::session_manager::{SessionInfo, SessionManager, SessionMessage};

#[tauri::command]
pub fn list_sessions(platform: Option<String>) -> Vec<SessionInfo> {
    SessionManager::list_sessions(platform)
}

#[tauri::command]
pub fn get_session_messages(
    platform: String,
    source_path: String,
) -> Result<Vec<SessionMessage>, String> {
    SessionManager::load_messages(&platform, &source_path)
}

#[tauri::command]
pub fn search_sessions(query: String, platform: Option<String>) -> Vec<SessionInfo> {
    SessionManager::search_sessions(&query, platform)
}

#[tauri::command]
pub fn delete_session(
    platform: String,
    session_id: String,
    source_path: String,
) -> Result<bool, String> {
    SessionManager::delete_session(&platform, &session_id, &source_path)
}
