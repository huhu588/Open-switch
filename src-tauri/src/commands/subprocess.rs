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
