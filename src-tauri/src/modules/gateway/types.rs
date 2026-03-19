use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayAccount {
    pub id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
    pub status: AccountStatus,
    pub tags: Option<String>,
    pub group_name: Option<String>,
    pub proxy_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
    pub cooldown_until: Option<i64>,
    pub error_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Inactive,
    Cooldown,
    Error,
    Expired,
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStatus::Active => write!(f, "active"),
            AccountStatus::Inactive => write!(f, "inactive"),
            AccountStatus::Cooldown => write!(f, "cooldown"),
            AccountStatus::Error => write!(f, "error"),
            AccountStatus::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayApiKey {
    pub id: String,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub allowed_models: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub usage_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogEntry {
    pub id: i64,
    pub trace_id: String,
    pub timestamp: i64,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub duration_ms: i64,
    pub account_email: Option<String>,
    pub model: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub error_message: Option<String>,
    pub api_key_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub running: bool,
    pub port: u16,
    pub total_accounts: usize,
    pub active_accounts: usize,
    pub total_api_keys: usize,
    pub total_requests: i64,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub enabled: bool,
    pub port: u16,
    pub upstream_base_url: String,
    pub upstream_proxy_url: Option<String>,
    pub route_strategy: RouteStrategy,
    pub auto_start: bool,
    pub cors_enabled: bool,
    pub max_concurrent_per_account: u32,
    pub cooldown_seconds: u32,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 48760,
            upstream_base_url: "https://api.openai.com".to_string(),
            upstream_proxy_url: None,
            route_strategy: RouteStrategy::RoundRobin,
            auto_start: false,
            cors_enabled: true,
            max_concurrent_per_account: 5,
            cooldown_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RouteStrategy {
    RoundRobin,
    LeastUsed,
    Random,
    Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub api_key_id: Option<String>,
    pub trace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountImportPayload {
    pub email: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub tags: Option<String>,
    pub group_name: Option<String>,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreatePayload {
    pub name: String,
    pub allowed_models: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status_code: Option<u16>,
    pub model: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogSummary {
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_duration_ms: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
}
