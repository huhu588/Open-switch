use super::db;
use super::types::{RequestLogEntry, RequestLogQuery, RequestLogSummary};

pub fn log_request(entry: &RequestLogEntry) -> Result<(), String> {
    db::insert_request_log(entry)
}

pub fn query_logs(query: &RequestLogQuery) -> Result<Vec<RequestLogEntry>, String> {
    db::query_request_logs(query)
}

pub fn get_summary() -> Result<RequestLogSummary, String> {
    db::get_request_log_summary()
}

pub fn clear_logs() -> Result<(), String> {
    db::clear_request_logs()
}

pub fn create_log_entry(
    trace_id: String,
    method: String,
    path: String,
    status_code: u16,
    duration_ms: i64,
    account_email: Option<String>,
    model: Option<String>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    error_message: Option<String>,
    api_key_prefix: Option<String>,
) -> RequestLogEntry {
    RequestLogEntry {
        id: 0,
        trace_id,
        timestamp: chrono::Utc::now().timestamp(),
        method,
        path,
        status_code,
        duration_ms,
        account_email,
        model,
        input_tokens,
        output_tokens,
        error_message,
        api_key_prefix,
    }
}
