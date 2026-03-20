use crate::modules::subprocess::sub2api::{get_sub2api_admin_credentials, get_sub2api_port};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

static SUB2API_TOKEN: RwLock<Option<String>> = RwLock::new(None);

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

fn get_sub2api_base_url() -> Result<String, String> {
    let port = get_sub2api_port();
    if port == 0 {
        return Err("Sub2api 未运行".to_string());
    }
    Ok(format!("http://127.0.0.1:{}", port))
}

async fn ensure_auth() -> Result<String, String> {
    {
        let guard = SUB2API_TOKEN.read().map_err(|e| e.to_string())?;
        if let Some(ref token) = *guard {
            return Ok(token.clone());
        }
    }
    do_login().await
}

async fn do_login() -> Result<String, String> {
    let base_url = get_sub2api_base_url()?;
    let (admin_email, admin_password) = get_sub2api_admin_credentials();

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/v1/auth/login", base_url))
        .json(&LoginRequest {
            email: admin_email,
            password: admin_password,
        })
        .send()
        .await
        .map_err(|e| format!("登录请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("登录失败 ({}): {}", status, body));
    }

    let login_resp: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析登录响应失败: {}", e))?;

    let token = login_resp
        .get("data")
        .and_then(|d| d.get("token").or(d.get("access_token")))
        .or(login_resp.get("token").or(login_resp.get("access_token")))
        .and_then(|t| t.as_str())
        .ok_or("登录响应中未找到 token")?
        .to_string();

    {
        let mut guard = SUB2API_TOKEN.write().map_err(|e| e.to_string())?;
        *guard = Some(token.clone());
    }

    tracing::info!("[Sub2api Proxy] 自动登录成功");
    Ok(token)
}

#[tauri::command]
pub async fn sub2api_proxy(
    method: String,
    path: String,
    body: Option<String>,
    query: Option<String>,
) -> Result<String, String> {
    let base_url = get_sub2api_base_url()?;
    let token = ensure_auth().await?;

    let full_path = if path.starts_with("/api/v1") {
        path.clone()
    } else {
        format!("/api/v1{}", path)
    };

    let mut url = format!("{}{}", base_url, full_path);
    if let Some(ref q) = query {
        if !q.is_empty() {
            url = format!("{}?{}", url, q);
        }
    }

    let client = reqwest::Client::new();
    let mut req_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => return Err(format!("不支持的 HTTP 方法: {}", method)),
    };

    req_builder = req_builder
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json");

    if let Some(ref b) = body {
        req_builder = req_builder.body(b.clone());
    }

    let resp = req_builder
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = resp.status();

    if status.as_u16() == 401 {
        tracing::warn!("[Sub2api Proxy] Token 过期，重新登录");
        {
            let mut guard = SUB2API_TOKEN.write().map_err(|e| e.to_string())?;
            *guard = None;
        }
        let new_token = do_login().await?;

        let mut retry_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            "PATCH" => client.patch(&url),
            _ => return Err(format!("不支持的 HTTP 方法: {}", method)),
        };

        retry_builder = retry_builder
            .header("Authorization", format!("Bearer {}", new_token))
            .header("Content-Type", "application/json");

        if let Some(ref b) = body {
            retry_builder = retry_builder.body(b.clone());
        }

        let retry_resp = retry_builder
            .send()
            .await
            .map_err(|e| format!("重试请求失败: {}", e))?;

        let retry_status = retry_resp.status();
        let retry_body = retry_resp.text().await.unwrap_or_default();

        if !retry_status.is_success() {
            return Err(format!("请求失败 ({}): {}", retry_status, retry_body));
        }
        return Ok(retry_body);
    }

    let resp_body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("请求失败 ({}): {}", status, resp_body));
    }
    Ok(resp_body)
}

#[tauri::command]
pub async fn sub2api_login() -> Result<String, String> {
    {
        let mut guard = SUB2API_TOKEN.write().map_err(|e| e.to_string())?;
        *guard = None;
    }
    do_login().await
}

#[tauri::command]
pub fn sub2api_clear_auth() {
    if let Ok(mut guard) = SUB2API_TOKEN.write() {
        *guard = None;
    }
}
