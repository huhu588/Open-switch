use super::types::GatewayConfig;
use crate::modules::logger;
use std::path::PathBuf;
use std::sync::Mutex;

static GATEWAY_CONFIG: Mutex<Option<GatewayConfig>> = Mutex::new(None);

fn config_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("gateway_config.json")
}

pub fn get_gateway_config() -> GatewayConfig {
    let mut guard = GATEWAY_CONFIG.lock().unwrap();
    if let Some(ref config) = *guard {
        return config.clone();
    }

    let path = config_path();
    let config = if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(e) => {
                logger::log_error(&format!("[Gateway] 读取配置失败: {}", e));
                GatewayConfig::default()
            }
        }
    } else {
        GatewayConfig::default()
    };

    *guard = Some(config.clone());
    config
}

pub fn save_gateway_config(config: &GatewayConfig) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入配置失败: {}", e))?;

    let mut guard = GATEWAY_CONFIG.lock().unwrap();
    *guard = Some(config.clone());

    Ok(())
}
