use super::{api_key, protocol_adapter, proxy};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, Method, Response, StatusCode},
    routing::{any, get},
    Router,
};
use bytes::Bytes;

pub fn create_router() -> Router {
    Router::new()
        .route("/v1/chat/completions", any(gateway_handler))
        .route("/v1/responses", any(gateway_handler))
        .route("/v1/responses/{*rest}", any(gateway_handler))
        .route("/v1/messages", any(gateway_handler))
        .route("/v1/models", get(models_handler))
        .route("/health", get(health_handler))
        .fallback(any(gateway_handler))
}

async fn health_handler() -> &'static str {
    "ok"
}

async fn models_handler(headers: HeaderMap) -> Response<Body> {
    if let Err(resp) = verify_auth(&headers) {
        return resp;
    }

    let models = protocol_adapter::build_models_response();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(serde_json::to_vec(&models).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}

async fn gateway_handler(req: Request) -> Response<Body> {
    if req.method() == Method::OPTIONS {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
            .header("Access-Control-Allow-Headers", "Authorization, Content-Type, X-Request-Id")
            .header("Access-Control-Max-Age", "86400")
            .body(Body::empty())
            .unwrap();
    }

    let headers = req.headers().clone();

    let api_key_info = match verify_auth_and_get_key(&headers) {
        Ok(key) => key,
        Err(resp) => return resp,
    };

    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let header_pairs: Vec<(String, String)> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = match axum::body::to_bytes(req.into_body(), 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return json_error_response(
                StatusCode::BAD_REQUEST,
                &format!("读取请求体失败: {}", e),
            );
        }
    };

    match proxy::handle_proxy_request(
        &method,
        &path,
        &header_pairs,
        body,
        api_key_info.as_ref(),
    )
    .await
    {
        Ok(proxy_resp) => {
            let mut resp_builder = Response::builder().status(proxy_resp.status);

            for (key, value) in &proxy_resp.headers {
                let lower_key = key.to_lowercase();
                if lower_key == "transfer-encoding" || lower_key == "content-length" {
                    continue;
                }
                resp_builder = resp_builder.header(key.as_str(), value.as_str());
            }

            resp_builder = resp_builder
                .header("Access-Control-Allow-Origin", "*");

            resp_builder
                .body(Body::from(proxy_resp.body))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap()
                })
        }
        Err(e) => json_error_response(StatusCode::BAD_GATEWAY, &e),
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

fn verify_auth(headers: &HeaderMap) -> Result<(), Response<Body>> {
    let token = extract_bearer_token(headers).ok_or_else(|| {
        json_error_response(StatusCode::UNAUTHORIZED, "缺少 Authorization 头")
    })?;

    match api_key::verify_api_key(&token) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(json_error_response(
            StatusCode::UNAUTHORIZED,
            "无效的 API Key",
        )),
        Err(e) => Err(json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("验证 API Key 失败: {}", e),
        )),
    }
}

fn verify_auth_and_get_key(
    headers: &HeaderMap,
) -> Result<Option<super::types::GatewayApiKey>, Response<Body>> {
    let token = match extract_bearer_token(headers) {
        Some(t) => t,
        None => {
            return Err(json_error_response(
                StatusCode::UNAUTHORIZED,
                "缺少 Authorization 头",
            ))
        }
    };

    match api_key::verify_api_key(&token) {
        Ok(key) => {
            if key.is_none() {
                return Err(json_error_response(
                    StatusCode::UNAUTHORIZED,
                    "无效的 API Key",
                ));
            }
            Ok(key)
        }
        Err(e) => Err(json_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("验证 API Key 失败: {}", e),
        )),
    }
}

fn json_error_response(status: StatusCode, message: &str) -> Response<Body> {
    let body = serde_json::json!({
        "error": {
            "message": message,
            "type": "gateway_error",
            "code": status.as_u16()
        }
    });

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(serde_json::to_vec(&body).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}
