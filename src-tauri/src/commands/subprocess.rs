use crate::modules::subprocess;

#[tauri::command]
pub async fn start_sub2api() -> Result<subprocess::Sub2apiStatus, String> {
    subprocess::start_sub2api().await
}

#[tauri::command]
pub async fn stop_sub2api() -> Result<(), String> {
    subprocess::stop_sub2api().await
}

#[tauri::command]
pub fn get_sub2api_status() -> subprocess::Sub2apiStatus {
    subprocess::get_sub2api_status()
}

#[tauri::command]
pub fn get_sub2api_port() -> u16 {
    subprocess::get_sub2api_port()
}

#[tauri::command]
pub fn save_sub2api_config(config: subprocess::sub2api::Sub2apiConfig) -> Result<(), String> {
    subprocess::sub2api::save_sub2api_config(&config)
}

#[tauri::command]
pub async fn start_cursor_welfare() -> Result<subprocess::CursorWelfareStatus, String> {
    subprocess::start_cursor_welfare().await
}

#[tauri::command]
pub async fn stop_cursor_welfare() -> Result<(), String> {
    subprocess::stop_cursor_welfare().await
}

#[tauri::command]
pub fn get_cursor_welfare_status() -> subprocess::CursorWelfareStatus {
    subprocess::get_cursor_welfare_status()
}

#[tauri::command]
pub fn get_cursor_welfare_port() -> u16 {
    subprocess::get_cursor_welfare_port()
}

#[tauri::command]
pub fn get_cursor_welfare_config() -> subprocess::cursor_welfare::CursorWelfareConfig {
    subprocess::cursor_welfare::get_cursor_welfare_config_public()
}

#[tauri::command]
pub fn check_cursor_welfare_binary() -> subprocess::cursor_welfare::BinaryCheckResult {
    subprocess::cursor_welfare::check_cursor_welfare_binary()
}

#[tauri::command]
pub fn save_cursor_welfare_config(
    config: subprocess::cursor_welfare::CursorWelfareConfig,
) -> Result<(), String> {
    subprocess::cursor_welfare::save_cursor_welfare_config(&config)
}

/// 把 Cursor 福利注册为 Sub2api 的代理服务，生成可分享的地址
#[derive(serde::Serialize)]
pub struct ShareCursorWelfareResult {
    pub success: bool,
    pub share_url: Option<String>,
    pub message: String,
}

#[tauri::command]
pub async fn share_cursor_welfare_to_sub2api() -> Result<ShareCursorWelfareResult, String> {
    let cw_status = subprocess::get_cursor_welfare_status();
    if !cw_status.running {
        return Ok(ShareCursorWelfareResult {
            success: false,
            share_url: None,
            message: "Cursor 福利服务未运行，请先启动".to_string(),
        });
    }

    let sub2api_port = subprocess::get_sub2api_port();
    if sub2api_port == 0 {
        return Ok(ShareCursorWelfareResult {
            success: false,
            share_url: None,
            message: "Sub2api 服务未运行，请先启动".to_string(),
        });
    }

    let (admin_email, admin_password) = subprocess::sub2api::get_sub2api_admin_credentials();
    let sub2api_base = format!("http://127.0.0.1:{}", sub2api_port);

    let client = reqwest::Client::new();

    // 登录获取 token
    let login_resp = client
        .post(format!("{}/api/v1/auth/login", sub2api_base))
        .json(&serde_json::json!({
            "email": admin_email,
            "password": admin_password,
        }))
        .send()
        .await
        .map_err(|e| format!("Sub2api 登录失败: {}", e))?;

    if !login_resp.status().is_success() {
        return Ok(ShareCursorWelfareResult {
            success: false,
            share_url: None,
            message: format!("Sub2api 登录失败: {}", login_resp.status()),
        });
    }

    let login_json: serde_json::Value = login_resp.json().await.map_err(|e| e.to_string())?;
    let token = login_json
        .get("data")
        .and_then(|d| d.get("token").or(d.get("access_token")))
        .or(login_json.get("token").or(login_json.get("access_token")))
        .and_then(|t| t.as_str())
        .ok_or("Sub2api 登录响应中未找到 token")?
        .to_string();

    let proxy_name = "Cursor 福利 (cursor2api-go)";
    let proxy_url = format!("http://127.0.0.1:{}", cw_status.port);

    // 注册代理
    let add_resp = client
        .post(format!("{}/api/v1/admin/proxies", sub2api_base))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "name": proxy_name,
            "url": proxy_url,
            "type": "http",
        }))
        .send()
        .await
        .map_err(|e| format!("注册代理失败: {}", e))?;

    if add_resp.status().is_success() {
        let share_url = format!("{}/v1/chat/completions", sub2api_base);
        Ok(ShareCursorWelfareResult {
            success: true,
            share_url: Some(share_url),
            message: format!("已成功将 Cursor 福利注册为 Sub2api 代理: {}", proxy_name),
        })
    } else {
        let status = add_resp.status();
        let body = add_resp.text().await.unwrap_or_default();
        if body.contains("already exists") || body.contains("duplicate") {
            let share_url = format!("{}/v1/chat/completions", sub2api_base);
            Ok(ShareCursorWelfareResult {
                success: true,
                share_url: Some(share_url),
                message: "Cursor 福利代理已存在于 Sub2api".to_string(),
            })
        } else {
            Ok(ShareCursorWelfareResult {
                success: false,
                share_url: None,
                message: format!("注册代理失败 ({}): {}", status, body),
            })
        }
    }
}
