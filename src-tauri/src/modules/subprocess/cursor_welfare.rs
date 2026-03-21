use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

static CURSOR_WELFARE_RUNNING: AtomicBool = AtomicBool::new(false);
static CURSOR_WELFARE_PORT: AtomicU16 = AtomicU16::new(0);
static CURSOR_WELFARE_PROCESS: Mutex<Option<u32>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorWelfareStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorWelfareConfig {
    pub port: u16,
    pub api_key: String,
    pub models: String,
    pub timeout: u32,
    pub debug: bool,
    pub auto_start: bool,
    #[serde(default = "default_script_url")]
    pub script_url: String,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

fn default_script_url() -> String {
    "https://cursor.com/149e9513-01fa-4fb0-aad4-566afd725d1b/2d206a39-8ed7-437e-a3be-862e0f06eea3/a-4-a/c.js?i=0&v=3&h=cursor.com".to_string()
}

fn default_user_agent() -> String {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36".to_string()
}

impl Default for CursorWelfareConfig {
    fn default() -> Self {
        Self {
            port: 8002,
            api_key: "0000".to_string(),
            models: "claude-sonnet-4.6".to_string(),
            timeout: 60,
            debug: false,
            auto_start: false,
            script_url: default_script_url(),
            user_agent: default_user_agent(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.jlcodes.ai-switch")
        .join("cursor_welfare_config.json")
}

fn get_cursor_welfare_config() -> CursorWelfareConfig {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<CursorWelfareConfig>(&content) {
                return config;
            }
        }
    }
    CursorWelfareConfig::default()
}

pub fn save_cursor_welfare_config(config: &CursorWelfareConfig) -> Result<(), String> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("写入配置失败: {}", e))?;
    Ok(())
}

fn is_valid_binary(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => meta.len() > 0 && meta.is_file(),
        Err(_) => false,
    }
}

fn find_cursor_welfare_binary() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("获取可执行文件路径失败: {}", e))?
        .parent()
        .ok_or("无法获取父目录")?
        .to_path_buf();

    let ext = if cfg!(target_os = "windows") { ".exe" } else { "" };

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

    let triple_name = format!("cursor2api-go-{}{}", target_triple, ext);

    let mut candidates: Vec<PathBuf> = vec![
        exe_dir.join(format!("cursor2api-go{}", ext)),
        exe_dir.join(&triple_name),
        exe_dir.join("binaries").join(format!("cursor2api-go{}", ext)),
        exe_dir.join("binaries").join(&triple_name),
    ];

    let resource_dir = exe_dir.parent().unwrap_or(&exe_dir).join("resources");
    candidates.push(resource_dir.join(format!("cursor2api-go{}", ext)));
    candidates.push(resource_dir.join(&triple_name));

    if let Some(project_binaries) = exe_dir
        .ancestors()
        .find(|p| p.join("tauri.conf.json").exists())
        .map(|p| p.join("binaries"))
    {
        candidates.push(project_binaries.join(format!("cursor2api-go{}", ext)));
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
            "找到 cursor2api-go 文件但无法使用（文件为空或无效）: {:?}。请下载对应平台的 cursor2api-go 可执行文件。",
            found_but_empty
        ));
    }

    Err(format!(
        "未找到 cursor2api-go 二进制文件。请将 cursor2api-go 可执行文件放置到以下任一位置: {:?}",
        candidates
    ))
}

pub async fn start_cursor_welfare() -> Result<CursorWelfareStatus, String> {
    if CURSOR_WELFARE_RUNNING.load(Ordering::Relaxed) {
        return Err("Cursor 福利服务已在运行".to_string());
    }

    let config = get_cursor_welfare_config();
    let binary_path = find_cursor_welfare_binary()?;
    let port = config.port;

    tracing::info!(
        "[CursorWelfare] 使用二进制文件: {}, 端口: {}",
        binary_path.display(),
        port
    );

    let work_dir = binary_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    tracing::info!(
        "[CursorWelfare] 工作目录: {}, jscode存在: {}",
        work_dir.display(),
        work_dir.join("jscode").join("main.js").exists()
    );

    let mut child = Command::new(&binary_path)
        .current_dir(&work_dir)
        .env("PORT", port.to_string())
        .env("API_KEY", &config.api_key)
        .env("MODELS", &config.models)
        .env("TIMEOUT", config.timeout.to_string())
        .env("DEBUG", if config.debug { "true" } else { "false" })
        .env("SCRIPT_URL", &config.script_url)
        .env("USER_AGENT", &config.user_agent)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            let hint = if e.raw_os_error() == Some(193) {
                "（文件不是有效的可执行程序）"
            } else if e.raw_os_error() == Some(2) {
                "（文件不存在）"
            } else {
                ""
            };
            format!(
                "启动 cursor2api-go 失败: {}{}\n二进制路径: {}",
                e, hint, binary_path.display()
            )
        })?;

    let pid = child.id();

    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!("[CursorWelfare:stdout] {}", line);
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::warn!("[CursorWelfare:stderr] {}", line);
            }
        });
    }

    CURSOR_WELFARE_RUNNING.store(true, Ordering::Relaxed);
    CURSOR_WELFARE_PORT.store(port, Ordering::Relaxed);

    {
        let mut guard = CURSOR_WELFARE_PROCESS.lock().unwrap();
        *guard = pid;
    }

    tokio::spawn(async move {
        monitor_subprocess(child).await;
    });

    tokio::spawn(async move {
        wait_for_health(port, 30).await;
    });

    tracing::info!(
        "[CursorWelfare] 子进程已启动，PID: {:?}, 端口: {}",
        pid, port
    );

    Ok(CursorWelfareStatus {
        running: true,
        port,
        pid,
        url: Some(format!("http://localhost:{}", port)),
    })
}

pub async fn stop_cursor_welfare() -> Result<(), String> {
    if !CURSOR_WELFARE_RUNNING.load(Ordering::Relaxed) {
        return Err("Cursor 福利服务未在运行".to_string());
    }

    let pid = {
        let guard = CURSOR_WELFARE_PROCESS.lock().unwrap();
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

    CURSOR_WELFARE_RUNNING.store(false, Ordering::Relaxed);
    CURSOR_WELFARE_PORT.store(0, Ordering::Relaxed);

    {
        let mut guard = CURSOR_WELFARE_PROCESS.lock().unwrap();
        *guard = None;
    }

    tracing::info!("[CursorWelfare] 子进程已停止");
    Ok(())
}

pub fn get_cursor_welfare_status() -> CursorWelfareStatus {
    let running = CURSOR_WELFARE_RUNNING.load(Ordering::Relaxed);
    let port = CURSOR_WELFARE_PORT.load(Ordering::Relaxed);
    let pid = CURSOR_WELFARE_PROCESS.lock().unwrap().clone();

    CursorWelfareStatus {
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

pub fn get_cursor_welfare_port() -> u16 {
    CURSOR_WELFARE_PORT.load(Ordering::Relaxed)
}

pub fn get_cursor_welfare_config_public() -> CursorWelfareConfig {
    get_cursor_welfare_config()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryCheckResult {
    pub available: bool,
    pub path: Option<String>,
    pub error: Option<String>,
    pub download_url: String,
}

pub fn check_cursor_welfare_binary() -> BinaryCheckResult {
    match find_cursor_welfare_binary() {
        Ok(path) => BinaryCheckResult {
            available: true,
            path: Some(path.to_string_lossy().to_string()),
            error: None,
            download_url: "https://github.com/libaxuan/cursor2api-go/releases".to_string(),
        },
        Err(e) => BinaryCheckResult {
            available: false,
            path: None,
            error: Some(e),
            download_url: "https://github.com/libaxuan/cursor2api-go/releases".to_string(),
        },
    }
}

async fn monitor_subprocess(mut child: Child) {
    match child.wait().await {
        Ok(status) => {
            tracing::warn!(
                "[CursorWelfare] 子进程退出，状态码: {:?}",
                status.code()
            );
        }
        Err(e) => {
            tracing::error!("[CursorWelfare] 等待子进程失败: {}", e);
        }
    }

    CURSOR_WELFARE_RUNNING.store(false, Ordering::Relaxed);
    CURSOR_WELFARE_PORT.store(0, Ordering::Relaxed);
    {
        let mut guard = CURSOR_WELFARE_PROCESS.lock().unwrap();
        *guard = None;
    }
}

async fn wait_for_health(port: u16, timeout_secs: u64) {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:{}/health", port);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        if tokio::time::Instant::now() >= deadline {
            tracing::warn!("[CursorWelfare] 健康检查超时 ({}s)", timeout_secs);
            break;
        }

        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("[CursorWelfare] 健康检查通过");
                break;
            }
            _ => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}
