use super::{account_pool, api_key, config, protocol_adapter, request_log, types};
use bytes::Bytes;
use reqwest::Client;
use std::sync::OnceLock;
use std::time::Instant;

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .pool_max_idle_per_host(20)
            .build()
            .expect("创建 HTTP 客户端失败")
    })
}

pub async fn handle_proxy_request(
    method: &str,
    path: &str,
    headers: &[(String, String)],
    body: Bytes,
    api_key_info: Option<&types::GatewayApiKey>,
) -> Result<ProxyResponse, String> {
    let start = Instant::now();
    let trace_id = uuid::Uuid::new_v4().to_string();
    let gw_config = config::get_gateway_config();

    let model = protocol_adapter::extract_model_from_body(&body);
    if let (Some(ref key_info), Some(ref model_name)) = (api_key_info, &model) {
        if !api_key::is_model_allowed(key_info, model_name) {
            return Err(format!("API Key 不允许使用模型: {}", model_name));
        }
    }

    let account =
        account_pool::select_account(&gw_config.route_strategy)?;

    let (upstream_url, rewritten_body) =
        protocol_adapter::rewrite_request_for_upstream(path, &body, &gw_config.upstream_base_url)?;

    let is_stream = protocol_adapter::is_streaming_request(&body);
    let client = get_client();

    let mut req_builder = match method.to_uppercase().as_str() {
        "POST" => client.post(&upstream_url),
        "GET" => client.get(&upstream_url),
        "PUT" => client.put(&upstream_url),
        "DELETE" => client.delete(&upstream_url),
        "PATCH" => client.patch(&upstream_url),
        _ => client.post(&upstream_url),
    };

    for (key, value) in headers {
        let lower_key = key.to_lowercase();
        if lower_key == "host" || lower_key == "content-length" || lower_key == "authorization" {
            continue;
        }
        req_builder = req_builder.header(key.as_str(), value.as_str());
    }

    req_builder = req_builder
        .header("Authorization", format!("Bearer {}", account.access_token))
        .header("Content-Type", "application/json");

    if let Some(ref proxy_url) = gw_config.upstream_proxy_url {
        if !proxy_url.is_empty() {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                let proxy_client = Client::builder()
                    .proxy(proxy)
                    .timeout(std::time::Duration::from_secs(300))
                    .build()
                    .map_err(|e| format!("创建代理客户端失败: {}", e))?;

                req_builder = match method.to_uppercase().as_str() {
                    "POST" => proxy_client.post(&upstream_url),
                    "GET" => proxy_client.get(&upstream_url),
                    _ => proxy_client.post(&upstream_url),
                };

                for (key, value) in headers {
                    let lower_key = key.to_lowercase();
                    if lower_key == "host"
                        || lower_key == "content-length"
                        || lower_key == "authorization"
                    {
                        continue;
                    }
                    req_builder = req_builder.header(key.as_str(), value.as_str());
                }

                req_builder = req_builder
                    .header("Authorization", format!("Bearer {}", account.access_token))
                    .header("Content-Type", "application/json");
            }
        }
    }

    if method.to_uppercase() != "GET" {
        req_builder = req_builder.body(rewritten_body);
    }

    let response = req_builder
        .send()
        .await
        .map_err(|e| {
            account_pool::report_account_error(&account.id).ok();
            format!("上游请求失败: {}", e)
        })?;

    let status = response.status().as_u16();
    let duration_ms = start.elapsed().as_millis() as i64;

    let resp_headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    if status >= 429 {
        account_pool::cooldown_account(&account.id, gw_config.cooldown_seconds).ok();
    } else if status >= 400 {
        account_pool::report_account_error(&account.id).ok();
    } else {
        account_pool::reset_account_errors(&account.id).ok();
    }

    if let Some(ref key_info) = api_key_info {
        api_key::increment_usage(&key_info.key_hash).ok();
    }

    let resp_body = response
        .bytes()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let log_entry = request_log::create_log_entry(
        trace_id,
        method.to_string(),
        path.to_string(),
        status,
        duration_ms,
        Some(account.email.clone()),
        model,
        None,
        None,
        if status >= 400 {
            Some(String::from_utf8_lossy(&resp_body[..resp_body.len().min(500)]).to_string())
        } else {
            None
        },
        api_key_info.map(|k| k.key_prefix.clone()),
    );
    request_log::log_request(&log_entry).ok();

    Ok(ProxyResponse {
        status,
        headers: resp_headers,
        body: resp_body,
        is_stream,
    })
}

pub struct ProxyResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Bytes,
    pub is_stream: bool,
}
