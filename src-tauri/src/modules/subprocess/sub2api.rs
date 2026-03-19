use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Mutex;
use tokio::process::{Child, Command};

static SUB2API_RUNNING: AtomicBool = AtomicBool::new(false);
static SUB2API_PORT: AtomicU16 = AtomicU16::new(0);
static SUB2API_PROCESS: Mutex<Option<u32>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sub2apiStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sub2apiConfig {
    pub port: u16,
    pub auto_start: bool,
    pub mode: String,
    pub db_path: Option<String>,
}

impl Default for Sub2apiConfig {
    fn default() -> Self {
        Self {
            port: 48761,
            auto_start: false,
            mode: "simple".to_string(),
            db_path: None,
        }
    }
}

fn get_sub2api_config() -> Sub2apiConfig {
    let config_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch")
        .join("sub2api_config.json");

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<Sub2apiConfig>(&content) {
                return config;
            }
        }
    }
    Sub2apiConfig::default()
}

pub fn save_sub2api_config(config: &Sub2apiConfig) -> Result<(), String> {
    let config_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch");
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("创建配置目录失败: {}", e))?;

    let config_path = config_dir.join("sub2api_config.json");
    let json =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, json).map_err(|e| format!("写入配置失败: {}", e))?;
    Ok(())
}

fn find_sub2api_binary() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .parent()
        .ok_or("无法获取父目录")?
        .to_path_buf();

    let candidates = vec![
        exe_dir.join("sub2api"),
        exe_dir.join("sub2api.exe"),
        exe_dir.join("binaries").join("sub2api"),
        exe_dir.join("binaries").join("sub2api.exe"),
    ];

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    let resource_dir = exe_dir.parent().unwrap_or(&exe_dir).join("resources");
    let resource_candidates = vec![
        resource_dir.join("sub2api"),
        resource_dir.join("sub2api.exe"),
    ];

    for path in &resource_candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    Err(format!(
        "未找到 sub2api 二进制文件，已搜索: {:?}",
        candidates
    ))
}

pub async fn start_sub2api() -> Result<Sub2apiStatus, String> {
    if SUB2API_RUNNING.load(Ordering::Relaxed) {
        return Err("Sub2api 已在运行".to_string());
    }

    let config = get_sub2api_config();
    let binary_path = find_sub2api_binary()?;
    let port = config.port;

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch")
        .join("sub2api_data");
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("创建数据目录失败: {}", e))?;

    let db_path = config
        .db_path
        .unwrap_or_else(|| data_dir.join("sub2api.db").to_string_lossy().to_string());

    let mut child = Command::new(&binary_path)
        .env("SUB2API_PORT", port.to_string())
        .env("SUB2API_DB_PATH", &db_path)
        .env("SUB2API_MODE", &config.mode)
        .env("SUB2API_DATA_DIR", data_dir.to_string_lossy().to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 sub2api 失败: {}", e))?;

    let pid = child.id();

    SUB2API_RUNNING.store(true, Ordering::Relaxed);
    SUB2API_PORT.store(port, Ordering::Relaxed);

    {
        let mut guard = SUB2API_PROCESS.lock().unwrap();
        *guard = pid;
    }

    tokio::spawn(async move {
        monitor_subprocess(child, port).await;
    });

    tokio::spawn(async move {
        wait_for_health(port, 30).await;
    });

    tracing::info!(
        "[Sub2api] 子进程已启动，PID: {:?}, 端口: {}",
        pid,
        port
    );

    Ok(Sub2apiStatus {
        running: true,
        port,
        pid,
        url: Some(format!("http://localhost:{}", port)),
    })
}

pub async fn stop_sub2api() -> Result<(), String> {
    if !SUB2API_RUNNING.load(Ordering::Relaxed) {
        return Err("Sub2api 未在运行".to_string());
    }

    let pid = {
        let guard = SUB2API_PROCESS.lock().unwrap();
        *guard
    };

    if let Some(pid) = pid {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }

    SUB2API_RUNNING.store(false, Ordering::Relaxed);
    SUB2API_PORT.store(0, Ordering::Relaxed);

    {
        let mut guard = SUB2API_PROCESS.lock().unwrap();
        *guard = None;
    }

    tracing::info!("[Sub2api] 子进程已停止");
    Ok(())
}

pub fn get_sub2api_status() -> Sub2apiStatus {
    let running = SUB2API_RUNNING.load(Ordering::Relaxed);
    let port = SUB2API_PORT.load(Ordering::Relaxed);
    let pid = SUB2API_PROCESS.lock().unwrap().clone();

    Sub2apiStatus {
        running,
        port,
        pid,
        url: if running {
            Some(format!("http://localhost:{}", port))
        } else {
            None
        },
    }
}

pub fn get_sub2api_port() -> u16 {
    SUB2API_PORT.load(Ordering::Relaxed)
}

async fn monitor_subprocess(mut child: Child, _port: u16) {
    match child.wait().await {
        Ok(status) => {
            tracing::warn!(
                "[Sub2api] 子进程退出，状态码: {:?}",
                status.code()
            );
        }
        Err(e) => {
            tracing::error!("[Sub2api] 等待子进程失败: {}", e);
        }
    }

    SUB2API_RUNNING.store(false, Ordering::Relaxed);
    SUB2API_PORT.store(0, Ordering::Relaxed);
    {
        let mut guard = SUB2API_PROCESS.lock().unwrap();
        *guard = None;
    }

    let config = get_sub2api_config();
    if config.auto_start {
        tracing::info!("[Sub2api] 自动重启已启用，但需手动触发重启");
    }
}

async fn wait_for_health(port: u16, timeout_secs: u64) {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/health", port);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!("[Sub2api] 健康检查超时 ({}s)", timeout_secs);
            break;
        }

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[Sub2api] 健康检查通过");
                break;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}
