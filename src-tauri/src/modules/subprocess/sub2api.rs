use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Mutex;
use tokio::process::{Child, Command};

static SUB2API_RUNNING: AtomicBool = AtomicBool::new(false);
static SUB2API_PORT: AtomicU16 = AtomicU16::new(0);
static SUB2API_PROCESS: Mutex<Option<u32>> = Mutex::new(None);
static SUB2API_ADMIN_PASSWORD: Mutex<Option<String>> = Mutex::new(None);

const SUB2API_ADMIN_EMAIL: &str = "admin@sub2api.local";
const SUB2API_DEFAULT_PASSWORD: &str = "AiSwitch2024!@#Local";

pub fn get_sub2api_admin_credentials() -> (String, String) {
    let password = SUB2API_ADMIN_PASSWORD
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| SUB2API_DEFAULT_PASSWORD.to_string());
    (SUB2API_ADMIN_EMAIL.to_string(), password)
}

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

fn is_valid_binary(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => meta.len() > 0 && meta.is_file(),
        Err(_) => false,
    }
}

fn find_sub2api_binary() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .parent()
        .ok_or("无法获取父目录")?
        .to_path_buf();

    let target_triple = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-pc-windows-msvc"
        } else {
            "aarch64-pc-windows-msvc"
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else {
        "x86_64-unknown-linux-gnu"
    };

    let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let triple_name = format!("sub2api-{}{}", target_triple, ext);

    let mut candidates: Vec<PathBuf> = vec![
        exe_dir.join(format!("sub2api{}", ext)),
        exe_dir.join(&triple_name),
        exe_dir.join("binaries").join(format!("sub2api{}", ext)),
        exe_dir.join("binaries").join(&triple_name),
    ];

    let resource_dir = exe_dir.parent().unwrap_or(&exe_dir).join("resources");
    candidates.push(resource_dir.join(format!("sub2api{}", ext)));
    candidates.push(resource_dir.join(&triple_name));

    // 开发模式：从项目 src-tauri/binaries/ 搜索
    if let Some(project_binaries) = exe_dir
        .ancestors()
        .find(|p| p.join("tauri.conf.json").exists())
        .map(|p| p.join("binaries"))
    {
        candidates.push(project_binaries.join(format!("sub2api{}", ext)));
        candidates.push(project_binaries.join(&triple_name));
    }

    let mut found_but_empty: Vec<PathBuf> = Vec::new();

    for path in &candidates {
        if path.exists() {
            if is_valid_binary(path) {
                return Ok(path.clone());
            } else {
                found_but_empty.push(path.clone());
            }
        }
    }

    if !found_but_empty.is_empty() {
        return Err(format!(
            "找到 sub2api 文件但无法使用（文件为空或无效）: {:?}。请下载或编译对应平台的 sub2api 可执行文件并放置到正确位置。",
            found_but_empty
        ));
    }

    Err(format!(
        "未找到 sub2api 二进制文件。请将 sub2api 可执行文件放置到以下任一位置: {:?}",
        candidates
    ))
}

/// 确保 sub2api 的 config.yaml 和 .installed 文件存在，
/// 跳过首次运行的 setup wizard，直接以 simple 模式启动主服务。
fn ensure_sub2api_config(data_dir: &Path, port: u16, mode: &str) -> Result<(), String> {
    let config_path = data_dir.join("config.yaml");
    let lock_path = data_dir.join(".installed");

    if !config_path.exists() {
        let config_content = format!(
            r#"run_mode: {mode}
server:
  host: "127.0.0.1"
  port: {port}
  mode: "release"
log:
  level: "info"
  format: "console"
  output:
    to_stdout: true
    to_file: true
    file_path: ""
"#,
            mode = mode,
            port = port,
        );
        std::fs::write(&config_path, config_content)
            .map_err(|e| format!("写入 sub2api config.yaml 失败: {}", e))?;
        tracing::info!("[Sub2api] 已生成初始 config.yaml: {}", config_path.display());
    }

    if !lock_path.exists() {
        let lock_content = format!(
            "installed_at={}\n",
            chrono::Utc::now().to_rfc3339()
        );
        std::fs::write(&lock_path, lock_content)
            .map_err(|e| format!("写入 .installed 锁文件失败: {}", e))?;
    }

    Ok(())
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

    ensure_sub2api_config(&data_dir, port, &config.mode)?;

    tracing::info!(
        "[Sub2api] 使用二进制文件: {}, 数据目录: {}, 端口: {}",
        binary_path.display(),
        data_dir.display(),
        port
    );

    let admin_password = SUB2API_ADMIN_PASSWORD
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| {
            let pw = SUB2API_DEFAULT_PASSWORD.to_string();
            if let Ok(mut guard) = SUB2API_ADMIN_PASSWORD.lock() {
                *guard = Some(pw.clone());
            }
            pw
        });

    let child = Command::new(&binary_path)
        .env("SERVER_PORT", port.to_string())
        .env("SERVER_HOST", "127.0.0.1")
        .env("DATA_DIR", data_dir.to_string_lossy().to_string())
        .env("SUB2API_DB_PATH", &db_path)
        .env("SUB2API_DATA_DIR", data_dir.to_string_lossy().to_string())
        .env("ADMIN_EMAIL", SUB2API_ADMIN_EMAIL)
        .env("ADMIN_PASSWORD", &admin_password)
        .env("AUTO_SETUP", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            let hint = if e.raw_os_error() == Some(193) {
                "（文件不是有效的可执行程序，可能是空文件或架构不匹配）"
            } else if e.raw_os_error() == Some(2) {
                "（文件不存在）"
            } else {
                ""
            };
            format!(
                "启动 sub2api 失败: {}{}\n二进制路径: {}",
                e,
                hint,
                binary_path.display()
            )
        })?;

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
