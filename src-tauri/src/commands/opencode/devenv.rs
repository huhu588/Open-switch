// 编程环境管理模块
// 支持检测、切换、安装全球前10编程语言环境
// Node.js / Python / Rust / Go / Java / C/C++ / C#(.NET) / PHP / Kotlin / Swift

use serde::{Deserialize, Serialize};
use std::process::Command;

// Windows 平台特定：隐藏命令行窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 版本管理器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManagerInfo {
    /// 管理器名称，如 "nvm", "pyenv", "rustup"
    pub name: String,
    /// 是否已安装
    pub installed: bool,
    /// 管理器版本
    pub version: Option<String>,
    /// 安装命令提示
    pub install_hint: String,
    /// 是否支持通过应用卸载（共享管理器如 scoop/winget 不支持单独卸载）
    pub can_uninstall: bool,
}

/// 推荐版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedVersion {
    /// 版本号
    pub version: String,
    /// 标签说明
    pub label: String,
    /// 是否为 Claude 工具适配版本
    pub for_claude: bool,
}

/// 环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevEnvInfo {
    /// 环境名称，如 "Node.js"
    pub name: String,
    /// 环境标识符，如 "nodejs"
    pub id: String,
    /// 是否已安装
    pub installed: bool,
    /// 当前版本
    pub current_version: Option<String>,
    /// 已安装的版本列表
    pub installed_versions: Vec<String>,
    /// 版本管理器信息
    pub version_manager: VersionManagerInfo,
    /// 推荐版本列表
    pub recommended_versions: Vec<RecommendedVersion>,
    /// 图标标识
    pub icon: String,
}

// ========== 推荐版本配置 ==========

/// NVM 默认安装版本
const NVM_DEFAULT_VERSION: &str = "1.2.2";

/// 获取 Node.js 推荐版本
fn nodejs_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "24.10.0".to_string(),
            label: "适配 Claude/Codex 工具".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "20.18.1".to_string(),
            label: "LTS 长期支持，最稳定".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Python 推荐版本
fn python_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "3.13.1".to_string(),
            label: "新版稳定，AI 工具兼容".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "3.11.11".to_string(),
            label: "用户最多，兼容性最佳".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Rust 推荐版本
fn rust_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "stable".to_string(),
            label: "最新稳定版，完整工具链".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "1.82.0".to_string(),
            label: "广泛使用的稳定版本".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Go 推荐版本
fn go_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "1.23.4".to_string(),
            label: "新版稳定，性能优化".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "1.21.13".to_string(),
            label: "长期支持，最稳定".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Java 推荐版本
fn java_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "21".to_string(),
            label: "LTS 新版，现代 Java 特性".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "17".to_string(),
            label: "用户最多的 LTS 版本".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 C/C++ 推荐版本
fn cpp_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "14.2".to_string(),
            label: "最新版，C++23 支持".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "13.3".to_string(),
            label: "稳定版，广泛使用".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 C#(.NET) 推荐版本
fn dotnet_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "9.0".to_string(),
            label: "最新版，性能提升显著".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "8.0".to_string(),
            label: "LTS 长期支持，生产首选".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 PHP 推荐版本
fn php_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "8.4".to_string(),
            label: "最新版，属性钩子等新特性".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "8.3".to_string(),
            label: "稳定版，生态兼容性佳".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Kotlin 推荐版本
fn kotlin_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "2.1.0".to_string(),
            label: "最新版，K2 编译器默认".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "2.0.21".to_string(),
            label: "稳定版，广泛使用".to_string(),
            for_claude: false,
        },
    ]
}

/// 获取 Swift 推荐版本
fn swift_recommended() -> Vec<RecommendedVersion> {
    vec![
        RecommendedVersion {
            version: "6.0.3".to_string(),
            label: "最新版，严格并发".to_string(),
            for_claude: true,
        },
        RecommendedVersion {
            version: "5.10.1".to_string(),
            label: "稳定版，兼容性佳".to_string(),
            for_claude: false,
        },
    ]
}

// ========== 命令执行工具 ==========

/// 解码命令输出（Windows 中文系统 cmd 输出 GBK 编码，需要正确解码）
fn decode_output(bytes: &[u8]) -> String {
    // 先尝试 UTF-8 解码
    match String::from_utf8(bytes.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            // Windows 中文系统 cmd.exe 默认输出 GBK 编码
            #[cfg(target_os = "windows")]
            {
                let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
                decoded.to_string()
            }
            #[cfg(not(target_os = "windows"))]
            {
                String::from_utf8_lossy(bytes).to_string()
            }
        }
    }
}

/// 执行命令并返回标准输出（隐藏窗口）
fn run_cmd(program: &str, args: &[&str]) -> Option<String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", program])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = decode_output(&out.stdout).trim().to_string();
            if !stdout.is_empty() {
                return Some(stdout);
            }
            // 某些命令（如 java -version）成功但输出到 stderr
            let stderr = decode_output(&out.stderr).trim().to_string();
            if stderr.is_empty() { None } else { Some(stderr) }
        }
        // 命令失败（包括 "不是内部或外部命令"）一律返回 None
        _ => None,
    }
}

/// 执行命令并返回完整输出（stdout + stderr，用于安装等操作）
fn run_cmd_full(program: &str, args: &[&str]) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", program])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(program)
        .args(args)
        .output();

    match output {
        Ok(out) => {
            let stdout = decode_output(&out.stdout);
            let stderr = decode_output(&out.stderr);
            let combined = format!("{}{}", stdout, stderr);
            if out.status.success() {
                Ok(combined)
            } else {
                Err(combined)
            }
        }
        Err(e) => Err(format!("执行命令失败: {}", e)),
    }
}

#[cfg(target_os = "windows")]
fn scoop_uninstall_manager_safely() -> Result<String, String> {
    // 1) 如果 PATH 中可用，直接调用
    if run_cmd("scoop", &["--version"]).is_some() {
        return run_cmd_full("scoop", &["uninstall", "scoop"]);
    }
    // 2) 尝试用默认 shims 路径调用（即使不在 PATH）
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let shim = format!("{}\\scoop\\shims\\scoop.cmd", user_profile);
    if std::path::Path::new(&shim).exists() {
        return run_cmd_full(&shim, &["uninstall", "scoop"]);
    }
    // 3) 视为已卸载
    Ok("Scoop 已卸载或不在 PATH 中".to_string())
}

// ========== 依赖自动安装工具 ==========

/// 检查并自动安装 Scoop（Windows 包管理器）
#[cfg(target_os = "windows")]
fn ensure_scoop_installed() -> Result<(), String> {
    // 已安装且在 PATH 中，直接返回
    if run_cmd("scoop", &["--version"]).is_some() {
        return Ok(());
    }

    // 检查 Scoop 是否已安装但未在 PATH 中（默认路径）
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let scoop_shim_dir = format!("{}\\scoop\\shims", user_profile);
    let scoop_cmd_path = format!("{}\\scoop.cmd", scoop_shim_dir);

    if std::path::Path::new(&scoop_cmd_path).exists() {
        // Scoop 已安装但 PATH 未包含，手动添加
        let current_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{};{}", scoop_shim_dir, current_path));
        if run_cmd("scoop", &["--version"]).is_some() {
            return Ok(());
        }
    }
    
    // 通过 PowerShell 自动安装 Scoop（启用 TLS 1.2，使用 -RunAsAdmin 允许管理员身份安装）
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command",
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; iex \"& {$(irm get.scoop.sh)} -RunAsAdmin\""
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    
    match output {
        Ok(out) => {
            let stdout_text = decode_output(&out.stdout);
            let stderr_text = decode_output(&out.stderr);
            let combined = format!("{}{}", stdout_text, stderr_text);

            if !out.status.success() {
                return Err(format!("Scoop 安装失败: {}", combined.trim()));
            }

            // 刷新当前进程的环境变量
            let _ = refresh_env_path();

            // 再次检查默认路径并手动补充 PATH（避免注册表刷新延迟）
            if std::path::Path::new(&scoop_cmd_path).exists() {
                let current_path = std::env::var("PATH").unwrap_or_default();
                if !current_path.to_lowercase().contains(&scoop_shim_dir.to_lowercase()) {
                    std::env::set_var("PATH", format!("{};{}", scoop_shim_dir, current_path));
                }
            }

            // 最终验证
            if run_cmd("scoop", &["--version"]).is_some() {
                Ok(())
            } else {
                Err(format!("Scoop 安装后仍无法使用（输出: {}），请手动在 PowerShell 中运行: irm get.scoop.sh | iex", combined.trim()))
            }
        }
        Err(e) => Err(format!("Scoop 安装失败: {}", e)),
    }
}

/// 检查并自动安装 MSYS2（C/C++ 工具链）
#[cfg(target_os = "windows")]
fn ensure_msys2_installed() -> Result<(), String> {
    // 检查 pacman 是否可用（MSYS2 已安装且在 PATH 中）
    if run_cmd("pacman", &["--version"]).is_some() {
        return Ok(());
    }
    
    // 检查 MSYS2 是否已安装但未在 PATH 中
    let msys2_paths = [
        "C:\\msys64\\usr\\bin\\pacman.exe",
        "C:\\msys64\\mingw64\\bin\\gcc.exe",
    ];
    
    let msys2_exists = msys2_paths.iter().any(|p| std::path::Path::new(p).exists());
    
    if msys2_exists {
        // MSYS2 已安装但 PATH 未配置，返回提示
        return Err("MSYS2 已安装但未配置 PATH。请将 C:\\msys64\\mingw64\\bin 添加到系统环境变量，或从 MSYS2 终端运行: pacman -S mingw-w64-x86_64-gcc".to_string());
    }
    
    // 通过 winget 静默安装 MSYS2（指定 winget 源，避免 msstore 证书问题）
    let output = run_cmd_full("winget", &[
        "install", "MSYS2.MSYS2",
        "--source", "winget",
        "--silent",
        "--accept-package-agreements",
        "--accept-source-agreements"
    ]);
    
    match output {
        Ok(_) => {
            // 安装后刷新 PATH
            let _ = refresh_env_path();
            // MSYS2 安装后需要用户手动安装 gcc
            Err("MSYS2 已安装完成。请打开 MSYS2 MINGW64 终端运行: pacman -S --noconfirm mingw-w64-x86_64-gcc，然后将 C:\\msys64\\mingw64\\bin 添加到 PATH".to_string())
        }
        Err(e) => Err(format!("MSYS2 安装失败: {}", e)),
    }
}

/// 刷新当前进程的 PATH 环境变量（从注册表读取最新值）
#[cfg(target_os = "windows")]
fn refresh_env_path() -> Result<(), String> {
    // 通过 PowerShell 读取最新的系统+用户 PATH
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')"
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let new_path = decode_output(&out.stdout).trim().to_string();
            if !new_path.is_empty() {
                // 更新当前进程的 PATH，使后续子进程能找到新安装的工具
                std::env::set_var("PATH", &new_path);
            }
            Ok(())
        }
        _ => Err("刷新环境变量失败".to_string()),
    }
}

// ========== 非 Windows 平台 stub 函数 ==========

#[cfg(not(target_os = "windows"))]
fn refresh_env_path() -> Result<(), String> { Ok(()) }

#[cfg(not(target_os = "windows"))]
fn ensure_scoop_installed() -> Result<(), String> {
    Err("Scoop 仅在 Windows 上可用".to_string())
}

#[cfg(not(target_os = "windows"))]
fn ensure_msys2_installed() -> Result<(), String> {
    Err("MSYS2 仅在 Windows 上可用".to_string())
}

#[cfg(not(target_os = "windows"))]
fn scoop_uninstall_manager_safely() -> Result<String, String> {
    Err("Scoop 仅在 Windows 上可用".to_string())
}

/// 跨平台辅助：在 Windows 上以隐藏窗口运行 PowerShell 命令
#[cfg(target_os = "windows")]
fn run_powershell_hidden(cmd: &str) -> std::io::Result<std::process::Output> {
    Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
}

#[cfg(not(target_os = "windows"))]
fn run_powershell_hidden(_cmd: &str) -> std::io::Result<std::process::Output> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "PowerShell 仅在 Windows 上可用"))
}

// ========== PATH 检查工具 ==========

/// Windows 下检查已知目录并加入当前进程 PATH（解决安装后不在 PATH 中的问题）
#[cfg(target_os = "windows")]
fn ensure_dir_in_path(dir: &str) {
    if dir.is_empty() || !std::path::Path::new(dir).is_dir() {
        return;
    }
    let current_path = std::env::var("PATH").unwrap_or_default();
    if !current_path.to_lowercase().contains(&dir.to_lowercase()) {
        std::env::set_var("PATH", format!("{};{}", dir, current_path));
    }
}

/// Windows 下确保 Scoop shim 目录在 PATH 中
#[cfg(target_os = "windows")]
fn ensure_scoop_in_path() {
    if run_cmd("scoop", &["--version"]).is_some() {
        return;
    }
    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
    let scoop_shim = format!("{}\\scoop\\shims", user_profile);
    if std::path::Path::new(&format!("{}\\scoop.cmd", scoop_shim)).exists() {
        ensure_dir_in_path(&scoop_shim);
    }
}

// ========== 各环境检测逻辑 ==========

/// 提取版本号（支持 vX.Y.Z、goX.Y.Z、X.Y.Z 等格式，多行输出也能匹配）
fn extract_version(raw: &str) -> String {
    // 遍历所有行寻找版本号（处理多行输出如 scoop --version）
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        for word in line.split_whitespace() {
            // 去除常见前缀: v, go
            let trimmed = word.trim_start_matches('v').trim_start_matches("go");
            if trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return trimmed.to_string();
            }
        }
    }
    raw.lines().next().unwrap_or(raw).trim().to_string()
}

/// 解析 nvm list 输出为版本列表（Windows nvm-windows 格式）
fn parse_nvm_list(output: &str) -> Vec<String> {
    let mut versions = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("No ") {
            continue;
        }
        // nvm-windows 格式: "  * 20.18.1 (Currently using...)" 或 "    18.19.0"
        let cleaned = line.trim_start_matches('*').trim();
        // 提取版本号部分（空格前）
        if let Some(ver) = cleaned.split_whitespace().next() {
            let ver = ver.trim_start_matches('v');
            if ver.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                versions.push(ver.to_string());
            }
        }
    }
    versions
}

/// 解析 rustup toolchain list 输出
fn parse_rustup_toolchains(output: &str) -> Vec<String> {
    let mut toolchains = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 格式: "stable-x86_64-pc-windows-msvc (default)" 或 "1.82.0-x86_64-..."
        let name = line.split_whitespace().next().unwrap_or(line);
        // 提取主要标识（去除平台后缀）
        let short = name.split('-').next().unwrap_or(name);
        toolchains.push(short.to_string());
    }
    toolchains
}

/// 检测 Node.js 环境
fn detect_nodejs() -> DevEnvInfo {
    // Windows 下检查 nvm-windows 已知路径
    #[cfg(target_os = "windows")]
    {
        let nvm_home = std::env::var("NVM_HOME").unwrap_or_default();
        let appdata_nvm = format!("{}\\nvm", std::env::var("APPDATA").unwrap_or_default());
        for dir in &[&nvm_home, &appdata_nvm] {
            if !dir.is_empty() && std::path::Path::new(&format!("{}\\nvm.exe", dir)).exists() {
                ensure_dir_in_path(dir);
                // 同时确保 NVM_SYMLINK 在 PATH 中
                let symlink = std::env::var("NVM_SYMLINK").unwrap_or_else(|_| "C:\\Program Files\\nodejs".to_string());
                ensure_dir_in_path(&symlink);
                break;
            }
        }
    }

    // 检测 Node.js
    let node_version = run_cmd("node", &["--version"])
        .map(|v| extract_version(&v));
    let installed = node_version.is_some();

    // 检测 NVM（Windows: nvm-windows）
    let nvm_version = run_cmd("nvm", &["version"])
        .or_else(|| run_cmd("nvm", &["--version"]))
        .map(|v| extract_version(&v));
    let nvm_installed = nvm_version.is_some();

    // 获取已安装的 Node.js 版本列表
    let installed_versions = if nvm_installed {
        run_cmd("nvm", &["list"])
            .map(|out| parse_nvm_list(&out))
            .unwrap_or_default()
    } else {
        // 没有 nvm 时，只有当前版本
        node_version.clone().into_iter().collect()
    };

    DevEnvInfo {
        name: "Node.js".to_string(),
        id: "nodejs".to_string(),
        installed,
        current_version: node_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: "nvm".to_string(),
            installed: nvm_installed,
            version: nvm_version,
            install_hint: format!("推荐安装 nvm-windows v{}", NVM_DEFAULT_VERSION),
            can_uninstall: true,
        },
        recommended_versions: nodejs_recommended(),
        icon: "nodejs".to_string(),
    }
}

/// 检测 Python 环境
fn detect_python() -> DevEnvInfo {
    // 检测 Python（Windows 上优先 python，再试 python3）
    let py_version = run_cmd("python", &["--version"])
        .or_else(|| run_cmd("python3", &["--version"]))
        .map(|v| extract_version(&v));
    let installed = py_version.is_some();

    // 检测 pyenv（先检查 PATH，Windows 下再检查已知安装路径）
    #[allow(unused_mut)]
    let mut pyenv_version = run_cmd("pyenv", &["--version"])
        .map(|v| extract_version(&v));

    // Windows 下 pyenv-win 可能已安装但未在 PATH 中
    #[cfg(target_os = "windows")]
    if pyenv_version.is_none() {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        // 检查标准路径和旧路径
        let known_bins = [
            format!("{}\\.pyenv\\pyenv-win\\bin", user_profile),
            format!("{}\\pyenv-win\\pyenv-win\\bin", user_profile),
        ];
        for bin_dir in &known_bins {
            let pyenv_bat = format!("{}\\pyenv.bat", bin_dir);
            if std::path::Path::new(&pyenv_bat).exists() {
                // 找到 pyenv，将其加入当前进程 PATH
                let shims_dir = bin_dir.replace("\\bin", "\\shims");
                let current_path = std::env::var("PATH").unwrap_or_default();
                if !current_path.to_lowercase().contains(&bin_dir.to_lowercase()) {
                    std::env::set_var("PATH", format!("{};{};{}", bin_dir, shims_dir, current_path));
                }
                pyenv_version = run_cmd("pyenv", &["--version"])
                    .map(|v| extract_version(&v));
                break;
            }
        }
    }

    let pyenv_installed = pyenv_version.is_some();

    // 获取已安装版本列表
    let installed_versions = if pyenv_installed {
        run_cmd("pyenv", &["versions", "--bare"])
            .map(|out| {
                out.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        py_version.clone().into_iter().collect()
    };

    DevEnvInfo {
        name: "Python".to_string(),
        id: "python".to_string(),
        installed,
        current_version: py_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: "pyenv".to_string(),
            installed: pyenv_installed,
            version: pyenv_version,
            install_hint: "推荐安装 pyenv-win".to_string(),
            can_uninstall: true,
        },
        recommended_versions: python_recommended(),
        icon: "python".to_string(),
    }
}

/// 检测 Rust 环境
fn detect_rust() -> DevEnvInfo {
    // Windows 下检查 cargo bin 已知路径
    #[cfg(target_os = "windows")]
    {
        let cargo_home = std::env::var("CARGO_HOME")
            .unwrap_or_else(|_| format!("{}\\.cargo", std::env::var("USERPROFILE").unwrap_or_default()));
        let cargo_bin = format!("{}\\bin", cargo_home);
        if std::path::Path::new(&format!("{}\\rustup.exe", cargo_bin)).exists() {
            ensure_dir_in_path(&cargo_bin);
        }
    }

    // 检测 rustc
    let rust_version = run_cmd("rustc", &["--version"])
        .map(|v| extract_version(&v));
    let installed = rust_version.is_some();

    // 检测 rustup
    let rustup_version = run_cmd("rustup", &["--version"])
        .map(|v| extract_version(&v));
    let rustup_installed = rustup_version.is_some();

    // 获取已安装工具链列表
    let installed_versions = if rustup_installed {
        run_cmd("rustup", &["toolchain", "list"])
            .map(|out| parse_rustup_toolchains(&out))
            .unwrap_or_default()
    } else {
        rust_version.clone().into_iter().collect()
    };

    DevEnvInfo {
        name: "Rust".to_string(),
        id: "rust".to_string(),
        installed,
        current_version: rust_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: "rustup".to_string(),
            installed: rustup_installed,
            version: rustup_version,
            install_hint: "推荐从 https://rustup.rs 安装".to_string(),
            can_uninstall: true,
        },
        recommended_versions: rust_recommended(),
        icon: "rust".to_string(),
    }
}

/// 检测 Go 环境
fn detect_go() -> DevEnvInfo {
    // 检测 go
    let go_version = run_cmd("go", &["version"])
        .map(|v| extract_version(&v));
    let installed = go_version.is_some();

    // Go 通常不需要版本管理器，但检测 goenv
    let goenv_version = run_cmd("goenv", &["--version"])
        .map(|v| extract_version(&v));
    let goenv_installed = goenv_version.is_some();

    let installed_versions = if goenv_installed {
        run_cmd("goenv", &["versions", "--bare"])
            .map(|out| {
                out.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        go_version.clone().into_iter().collect()
    };

    DevEnvInfo {
        name: "Go".to_string(),
        id: "go".to_string(),
        installed,
        current_version: go_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: "goenv".to_string(),
            installed: goenv_installed,
            version: goenv_version,
            install_hint: "可通过 scoop 或手动安装 goenv".to_string(),
            can_uninstall: false, // Go 本身通过 winget 安装，与环境卸载等价
        },
        recommended_versions: go_recommended(),
        icon: "go".to_string(),
    }
}

/// 检测 Java 环境
fn detect_java() -> DevEnvInfo {
    // Windows 下确保 Scoop 在 PATH 中
    #[cfg(target_os = "windows")]
    ensure_scoop_in_path();

    // 检测 java（java --version 输出到 stderr）
    let java_version = run_cmd("java", &["--version"])
        .or_else(|| run_cmd("java", &["-version"]))
        .map(|v| extract_version(&v));
    let installed = java_version.is_some();

    // 检测版本管理器（Windows: scoop, 其他: sdkman）
    let (mgr_name, mgr_installed, mgr_version, mgr_hint) = if cfg!(target_os = "windows") {
        let output = run_cmd("scoop", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("scoop", installed, version, "推荐安装 Scoop 包管理器")
    } else {
        let output = run_cmd("sdk", &["version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("sdkman", installed, version, "推荐安装 SDKMAN")
    };

    let installed_versions = java_version.clone().into_iter().collect();

    DevEnvInfo {
        name: "Java".to_string(),
        id: "java".to_string(),
        installed,
        current_version: java_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: mgr_name.to_string(),
            installed: mgr_installed,
            version: mgr_version,
            install_hint: mgr_hint.to_string(),
            can_uninstall: true, // 共享管理器：允许卸载（卸载 Scoop，会影响 Java/PHP/Kotlin）
        },
        recommended_versions: java_recommended(),
        icon: "java".to_string(),
    }
}

/// 检测 C/C++ 环境
fn detect_cpp() -> DevEnvInfo {
    // Windows 下检查 MSYS2 已知路径
    #[cfg(target_os = "windows")]
    {
        ensure_dir_in_path("C:\\msys64\\mingw64\\bin");
        ensure_dir_in_path("C:\\msys64\\usr\\bin");
    }

    // 优先检测 gcc，再尝试 clang
    let gcc_version = run_cmd("gcc", &["--version"])
        .map(|v| extract_version(&v));
    let clang_version = if gcc_version.is_none() {
        run_cmd("clang", &["--version"]).map(|v| extract_version(&v))
    } else {
        None
    };
    let current_version = gcc_version.or(clang_version);
    let installed = current_version.is_some();

    // Windows 上检测 MSYS2/MinGW，其他平台检测包管理器
    let (mgr_name, mgr_installed, mgr_version, mgr_hint) = if cfg!(target_os = "windows") {
        let output = run_cmd("pacman", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("MSYS2", installed, version, "推荐安装 MSYS2 (MinGW-w64)")
    } else if cfg!(target_os = "macos") {
        let output = run_cmd("brew", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("Homebrew", installed, version, "推荐安装 Homebrew")
    } else {
        let output = run_cmd("apt", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("apt", installed, version, "使用 sudo apt install gcc g++")
    };

    let installed_versions = current_version.clone().into_iter().collect();

    DevEnvInfo {
        name: "C/C++".to_string(),
        id: "cpp".to_string(),
        installed,
        current_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: mgr_name.to_string(),
            installed: mgr_installed,
            version: mgr_version,
            install_hint: mgr_hint.to_string(),
            can_uninstall: true, // MSYS2 可单独卸载
        },
        recommended_versions: cpp_recommended(),
        icon: "cpp".to_string(),
    }
}

/// 检测 C#(.NET) 环境
fn detect_dotnet() -> DevEnvInfo {
    let dotnet_version = run_cmd("dotnet", &["--version"])
        .map(|v| extract_version(&v));
    let installed = dotnet_version.is_some();

    // 获取已安装的 SDK 列表
    let installed_versions = run_cmd("dotnet", &["--list-sdks"])
        .map(|out| {
            out.lines()
                .filter_map(|l| l.split_whitespace().next())
                .map(|v| v.to_string())
                .collect()
        })
        .unwrap_or_else(|| dotnet_version.clone().into_iter().collect());

    DevEnvInfo {
        name: "C# (.NET)".to_string(),
        id: "dotnet".to_string(),
        installed,
        current_version: dotnet_version.clone(),
        installed_versions,
        version_manager: VersionManagerInfo {
            name: "dotnet".to_string(),
            installed, // .NET SDK 自带版本管理
            version: dotnet_version,
            install_hint: "通过 winget/apt 安装 .NET SDK".to_string(),
            can_uninstall: false, // dotnet SDK 本身就是环境，与环境卸载等价
        },
        recommended_versions: dotnet_recommended(),
        icon: "dotnet".to_string(),
    }
}

/// 检测 PHP 环境
fn detect_php() -> DevEnvInfo {
    #[cfg(target_os = "windows")]
    ensure_scoop_in_path();

    let php_version = run_cmd("php", &["--version"])
        .map(|v| extract_version(&v));
    let installed = php_version.is_some();

    let (mgr_name, mgr_installed, mgr_version, mgr_hint) = if cfg!(target_os = "windows") {
        let output = run_cmd("scoop", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("scoop", installed, version, "推荐通过 Scoop 安装 PHP")
    } else {
        let output = run_cmd("phpbrew", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("phpbrew", installed, version, "推荐安装 phpbrew")
    };

    let installed_versions = php_version.clone().into_iter().collect();

    DevEnvInfo {
        name: "PHP".to_string(),
        id: "php".to_string(),
        installed,
        current_version: php_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: mgr_name.to_string(),
            installed: mgr_installed,
            version: mgr_version,
            install_hint: mgr_hint.to_string(),
            can_uninstall: true, // 共享管理器：允许卸载（卸载 Scoop，会影响 Java/PHP/Kotlin）
        },
        recommended_versions: php_recommended(),
        icon: "php".to_string(),
    }
}

/// 检测 Kotlin 环境
fn detect_kotlin() -> DevEnvInfo {
    #[cfg(target_os = "windows")]
    ensure_scoop_in_path();

    let kotlin_version = run_cmd("kotlin", &["-version"])
        .or_else(|| run_cmd("kotlinc", &["-version"]))
        .map(|v| extract_version(&v));
    let installed = kotlin_version.is_some();

    let (mgr_name, mgr_installed, mgr_version, mgr_hint) = if cfg!(target_os = "windows") {
        let output = run_cmd("scoop", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("scoop", installed, version, "推荐通过 Scoop 安装 Kotlin")
    } else {
        let output = run_cmd("sdk", &["version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("sdkman", installed, version, "推荐安装 SDKMAN")
    };

    let installed_versions = kotlin_version.clone().into_iter().collect();

    DevEnvInfo {
        name: "Kotlin".to_string(),
        id: "kotlin".to_string(),
        installed,
        current_version: kotlin_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: mgr_name.to_string(),
            installed: mgr_installed,
            version: mgr_version,
            install_hint: mgr_hint.to_string(),
            can_uninstall: true, // 共享管理器：允许卸载（卸载 Scoop，会影响 Java/PHP/Kotlin）
        },
        recommended_versions: kotlin_recommended(),
        icon: "kotlin".to_string(),
    }
}

/// 检测 Swift 环境
fn detect_swift() -> DevEnvInfo {
    let swift_version = run_cmd("swift", &["--version"])
        .map(|v| extract_version(&v));
    let installed = swift_version.is_some();

    let (mgr_name, mgr_installed, mgr_version, mgr_hint) = if cfg!(target_os = "macos") {
        let output = run_cmd("xcode-select", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("Xcode", installed, version, "Xcode 已内置 Swift")
    } else if cfg!(target_os = "windows") {
        let output = run_cmd("winget", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("winget", installed, version, "通过 winget 安装 Swift")
    } else {
        let output = run_cmd("swiftly", &["--version"]);
        let installed = output.is_some();
        let version = output.map(|v| extract_version(&v));
        ("swiftly", installed, version, "推荐安装 swiftly")
    };

    let installed_versions = swift_version.clone().into_iter().collect();

    DevEnvInfo {
        name: "Swift".to_string(),
        id: "swift".to_string(),
        installed,
        current_version: swift_version,
        installed_versions,
        version_manager: VersionManagerInfo {
            name: mgr_name.to_string(),
            installed: mgr_installed,
            version: mgr_version,
            install_hint: mgr_hint.to_string(),
            can_uninstall: false, // winget 是共享工具，隐藏“卸载管理器”按钮
        },
        recommended_versions: swift_recommended(),
        icon: "swift".to_string(),
    }
}

// ========== Tauri 命令 ==========

/// 检测所有编程环境
#[tauri::command]
pub async fn detect_all_dev_envs() -> Result<Vec<DevEnvInfo>, String> {
    // 使用 tokio::task::spawn_blocking 避免阻塞异步运行时
    let envs = tokio::task::spawn_blocking(|| {
        vec![
            detect_nodejs(),
            detect_python(),
            detect_rust(),
            detect_go(),
            detect_java(),
            detect_cpp(),
            detect_dotnet(),
            detect_php(),
            detect_kotlin(),
            detect_swift(),
        ]
    })
    .await
    .map_err(|e| format!("检测环境失败: {}", e))?;

    Ok(envs)
}

/// 检测单个环境（用于刷新）
#[tauri::command]
pub async fn detect_single_dev_env(env_name: String) -> Result<DevEnvInfo, String> {
    tokio::task::spawn_blocking(move || -> Result<DevEnvInfo, String> {
        match env_name.as_str() {
            "nodejs" => Ok(detect_nodejs()),
            "python" => Ok(detect_python()),
            "rust" => Ok(detect_rust()),
            "go" => Ok(detect_go()),
            "java" => Ok(detect_java()),
            "cpp" => Ok(detect_cpp()),
            "dotnet" => Ok(detect_dotnet()),
            "php" => Ok(detect_php()),
            "kotlin" => Ok(detect_kotlin()),
            "swift" => Ok(detect_swift()),
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("检测失败: {}", e))?
}

/// 获取已安装版本列表
#[tauri::command]
pub async fn get_installed_versions(env_name: String) -> Result<Vec<String>, String> {
    let versions = tokio::task::spawn_blocking(move || -> Result<Vec<String>, String> {
        match env_name.as_str() {
            "nodejs" => {
                run_cmd("nvm", &["list"])
                    .map(|out| parse_nvm_list(&out))
                    .ok_or_else(|| "无法获取 Node.js 版本列表，请确认 nvm 已安装".to_string())
            }
            "python" => {
                run_cmd("pyenv", &["versions", "--bare"])
                    .map(|out| {
                        out.lines()
                            .map(|l| l.trim().to_string())
                            .filter(|l| !l.is_empty())
                            .collect()
                    })
                    .ok_or_else(|| "无法获取 Python 版本列表，请确认 pyenv 已安装".to_string())
            }
            "rust" => {
                run_cmd("rustup", &["toolchain", "list"])
                    .map(|out| parse_rustup_toolchains(&out))
                    .ok_or_else(|| "无法获取 Rust 工具链列表，请确认 rustup 已安装".to_string())
            }
            "go" => {
                // Go 通常只有一个版本
                run_cmd("go", &["version"])
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "Go 未安装".to_string())
            }
            "java" => {
                run_cmd("java", &["--version"])
                    .or_else(|| run_cmd("java", &["-version"]))
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "Java 未安装".to_string())
            }
            "cpp" => {
                run_cmd("gcc", &["--version"])
                    .or_else(|| run_cmd("clang", &["--version"]))
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "C/C++ 编译器未安装".to_string())
            }
            "dotnet" => {
                run_cmd("dotnet", &["--list-sdks"])
                    .map(|out| {
                        out.lines()
                            .filter_map(|l| l.split_whitespace().next())
                            .map(|v| v.to_string())
                            .collect()
                    })
                    .ok_or_else(|| ".NET SDK 未安装".to_string())
            }
            "php" => {
                run_cmd("php", &["--version"])
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "PHP 未安装".to_string())
            }
            "kotlin" => {
                run_cmd("kotlin", &["-version"])
                    .or_else(|| run_cmd("kotlinc", &["-version"]))
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "Kotlin 未安装".to_string())
            }
            "swift" => {
                run_cmd("swift", &["--version"])
                    .map(|v| vec![extract_version(&v)])
                    .ok_or_else(|| "Swift 未安装".to_string())
            }
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("获取版本列表失败: {}", e))?;

    versions
}

/// 切换环境版本
#[tauri::command]
pub async fn switch_env_version(env_name: String, version: String) -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        match env_name.as_str() {
            "nodejs" => {
                run_cmd_full("nvm", &["use", &version])
                    .map(|out| format!("已切换 Node.js 到 v{}\n{}", version, out))
            }
            "python" => {
                run_cmd_full("pyenv", &["global", &version])
                    .map(|out| format!("已切换 Python 到 v{}\n{}", version, out))
            }
            "rust" => {
                run_cmd_full("rustup", &["default", &version])
                    .map(|out| format!("已切换 Rust 到 {}\n{}", version, out))
            }
            "go" => {
                Err("Go 不支持版本切换，请安装指定版本替换".to_string())
            }
            "java" => {
                if cfg!(target_os = "windows") {
                    // 提取主版本号: "21.0.8" → "21"
                    let major = version.split('.').next().unwrap_or(&version);
                    run_cmd_full("scoop", &["reset", &format!("openjdk{}", major)])
                        .map(|out| format!("已切换 Java 到 JDK {}\n{}", version, out))
                } else {
                    run_cmd_full("sdk", &["default", "java", &version])
                        .map(|out| format!("已切换 Java 到 {}\n{}", version, out))
                }
            }
            "cpp" => Err("C/C++ 版本切换请通过包管理器操作".to_string()),
            "dotnet" => {
                // .NET 通过 global.json 切换
                Err("请通过项目根目录的 global.json 指定 SDK 版本".to_string())
            }
            "php" => Err("PHP 版本切换请通过包管理器操作".to_string()),
            "kotlin" => {
                if cfg!(target_os = "windows") {
                    run_cmd_full("scoop", &["reset", &format!("kotlin@{}", version)])
                        .map(|out| format!("已切换 Kotlin 到 v{}\n{}", version, out))
                } else {
                    run_cmd_full("sdk", &["default", "kotlin", &version])
                        .map(|out| format!("已切换 Kotlin 到 v{}\n{}", version, out))
                }
            }
            "swift" => Err("Swift 版本切换请通过 swiftly 或 Xcode 操作".to_string()),
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("切换版本失败: {}", e))?;

    result
}

/// 安装指定版本
#[tauri::command]
pub async fn install_env_version(env_name: String, version: String) -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        match env_name.as_str() {
            "nodejs" => {
                run_cmd_full("nvm", &["install", &version])
                    .map(|out| format!("Node.js v{} 安装完成\n{}", version, out))
            }
            "python" => {
                run_cmd_full("pyenv", &["install", &version])
                    .map(|out| format!("Python v{} 安装完成\n{}", version, out))
            }
            "rust" => {
                run_cmd_full("rustup", &["toolchain", "install", &version])
                    .map(|out| format!("Rust {} 安装完成\n{}", version, out))
            }
            "go" => {
                // Go 需要下载安装包，提供指引
                Err("Go 版本安装请通过官网下载或使用 scoop install go".to_string())
            }
            "java" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 Scoop（如果未安装）
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}，请手动安装后重试", e));
                    }
                    // 添加 java bucket
                    let _ = run_cmd_full("scoop", &["bucket", "add", "java"]);
                    run_cmd_full("scoop", &["install", &format!("openjdk{}", version)])
                        .map(|out| format!("Java JDK {} 安装完成\n{}", version, out))
                } else {
                    run_cmd_full("sdk", &["install", "java", &version])
                        .map(|out| format!("Java {} 安装完成\n{}", version, out))
                }
            }
            "cpp" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 MSYS2（如果未安装）
                    if let Err(e) = ensure_msys2_installed() {
                        return Err(e);
                    }
                    // 通过 MSYS2 pacman 安装 gcc
                    run_cmd_full("pacman", &["-S", "--noconfirm", "mingw-w64-x86_64-gcc"])
                        .map(|out| format!("GCC 安装完成\n{}", out))
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["install", "gcc"])
                        .map(|out| format!("GCC 安装完成\n{}", out))
                } else {
                    run_cmd_full("sudo", &["apt", "install", "-y", "gcc", "g++"])
                        .map(|out| format!("GCC/G++ 安装完成\n{}", out))
                }
            }
            "dotnet" => {
                if cfg!(target_os = "windows") {
                    // 提取主版本号: "9.0" → "9", "8.0.401" → "8"
                    let major = version.split('.').next().unwrap_or(&version);
                    let result = run_cmd_full("winget", &["install", &format!("Microsoft.DotNet.SDK.{}", major), "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!(".NET SDK {} 安装完成\n{}", version, out))
                } else {
                    Err("请访问 https://dotnet.microsoft.com/download 下载安装".to_string())
                }
            }
            "php" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 Scoop（如果未安装）
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}，请手动安装后重试", e));
                    }
                    // 添加 versions bucket 以支持特定版本
                    let _ = run_cmd_full("scoop", &["bucket", "add", "versions"]);
                    // 构建包名: 8.4 → php84, 8.3 → php83
                    let major_minor: String = version.split('.').take(2).collect::<Vec<&str>>().join("");
                    let pkg = format!("php{}", major_minor);
                    // 尝试安装指定版本，检查 scoop 输出是否含错误
                    let result = run_cmd_full("scoop", &["install", &pkg]);
                    match &result {
                        Ok(out) if out.contains("Couldn't find manifest") || out.contains("ERROR") => {
                            // 具体版本包不存在，尝试安装最新版 php
                            run_cmd_full("scoop", &["install", "php"])
                                .map(|out| format!("PHP 安装完成（最新版）\n{}", out))
                        }
                        Ok(out) => Ok(format!("PHP {} 安装完成\n{}", version, out)),
                        Err(_) => {
                            run_cmd_full("scoop", &["install", "php"])
                                .map(|out| format!("PHP 安装完成（最新版）\n{}", out))
                        }
                    }
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["install", &format!("php@{}", version)])
                        .map(|out| format!("PHP {} 安装完成\n{}", version, out))
                } else {
                    run_cmd_full("sudo", &["apt", "install", "-y", &format!("php{}", version)])
                        .map(|out| format!("PHP {} 安装完成\n{}", version, out))
                }
            }
            "kotlin" => {
                if cfg!(target_os = "windows") {
                    // 仅安装/就绪 Scoop 作为管理器，不安装 Kotlin 包
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}，请手动安装后重试", e));
                    }
                    Ok("Scoop 已就绪".to_string())
                } else {
                    run_cmd_full("sdk", &["install", "kotlin", &version])
                        .map(|out| format!("Kotlin {} 安装完成\n{}", version, out))
                }
            }
            "swift" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "Swift.Toolchain", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("Swift 安装完成\n{}", out))
                } else if cfg!(target_os = "macos") {
                    Ok("Swift 已通过 Xcode 内置，请通过 App Store 更新 Xcode".to_string())
                } else {
                    Err("请访问 https://swift.org/download 下载安装".to_string())
                }
            }
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("安装版本失败: {}", e))?;

    result
}

/// 安装版本管理器
#[tauri::command]
pub async fn install_version_manager(env_name: String) -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        match env_name.as_str() {
            "nodejs" => {
                // 使用 winget 静默安装 nvm-windows
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "CoreyButler.NVMforWindows", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("nvm-windows 安装完成，请重启终端后使用\n{}", out))
                } else {
                    Err("请手动运行: curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash".to_string())
                }
            }
            "python" => {
                if cfg!(target_os = "windows") {
                    let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
                    let pyenv_home = format!("{}\\.pyenv", user_profile);
                    let pyenv_win_dir = format!("{}\\pyenv-win", pyenv_home);
                    let pyenv_bin = format!("{}\\bin", pyenv_win_dir);
                    let pyenv_shims = format!("{}\\shims", pyenv_win_dir);

                    // 安装 pyenv-win 到标准目录
                    let result = run_cmd_full("pip", &["install", "pyenv-win", "--target", &pyenv_home]);

                    if result.is_ok() {
                        // 配置当前进程的 PATH 和环境变量（立即生效）
                        let current_path = std::env::var("PATH").unwrap_or_default();
                        if !current_path.to_lowercase().contains(&pyenv_bin.to_lowercase()) {
                            std::env::set_var("PATH", format!("{};{};{}", pyenv_bin, pyenv_shims, current_path));
                        }
                        std::env::set_var("PYENV", &pyenv_win_dir);
                        std::env::set_var("PYENV_HOME", &pyenv_win_dir);
                        std::env::set_var("PYENV_ROOT", &pyenv_win_dir);

                        // 持久化到用户环境变量（重启后仍然有效）
                        let ps_cmd = format!(
                            "[Environment]::SetEnvironmentVariable('PYENV','{pw}','User');\
                            [Environment]::SetEnvironmentVariable('PYENV_HOME','{pw}','User');\
                            [Environment]::SetEnvironmentVariable('PYENV_ROOT','{pw}','User');\
                            $p=[Environment]::GetEnvironmentVariable('Path','User');\
                            if($p -notlike '*pyenv-win*'){{[Environment]::SetEnvironmentVariable('Path','{bin};{shims};'+$p,'User')}}",
                            pw = pyenv_win_dir, bin = pyenv_bin, shims = pyenv_shims
                        );
                        let _ = run_powershell_hidden(&ps_cmd);
                    }

                    result.map(|out| format!("pyenv-win 安装完成\n{}", out))
                        .map_err(|e| format!("pyenv-win 安装失败: {}", e))
                } else {
                    Err("请手动运行: curl https://pyenv.run | bash".to_string())
                }
            }
            "rust" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "Rustlang.Rustup", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("rustup 安装完成，请重启终端\n{}", out))
                } else {
                    Err("请手动运行: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string())
                }
            }
            "go" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "GoLang.Go", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("Go 安装完成\n{}", out))
                } else {
                    Err("请从 https://go.dev/dl/ 下载安装".to_string())
                }
            }
            "java" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 Scoop
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}", e));
                    }
                    let _ = run_cmd_full("scoop", &["bucket", "add", "java"]);
                    Ok("Scoop 和 Java Bucket 已就绪".to_string())
                } else {
                    Err("请手动运行: curl -s \"https://get.sdkman.io\" | bash".to_string())
                }
            }
            "cpp" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 MSYS2
                    if let Err(e) = ensure_msys2_installed() {
                        return Err(e);
                    }
                    Ok("MSYS2 已就绪".to_string())
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("xcode-select", &["--install"])
                        .map(|out| format!("Xcode 命令行工具安装完成\n{}", out))
                } else {
                    run_cmd_full("sudo", &["apt", "install", "-y", "build-essential"])
                        .map(|out| format!("build-essential 安装完成\n{}", out))
                }
            }
            "dotnet" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "Microsoft.DotNet.SDK.9", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!(".NET SDK 安装完成\n{}", out))
                } else {
                    Err("请访问 https://dotnet.microsoft.com/download 下载安装".to_string())
                }
            }
            "php" => {
                if cfg!(target_os = "windows") {
                    // 仅安装/就绪 Scoop 作为管理器，不安装 PHP 包
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}", e));
                    }
                    Ok("Scoop 已就绪".to_string())
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["install", "php"])
                        .map(|out| format!("PHP 安装完成\n{}", out))
                } else {
                    Err("请手动运行: sudo apt install php".to_string())
                }
            }
            "kotlin" => {
                if cfg!(target_os = "windows") {
                    // 自动安装 Scoop
                    if let Err(e) = ensure_scoop_installed() {
                        return Err(format!("自动安装 Scoop 失败: {}", e));
                    }
                    run_cmd_full("scoop", &["install", "kotlin"])
                        .map(|out| format!("Kotlin 安装完成\n{}", out))
                } else {
                    Err("请手动运行: curl -s \"https://get.sdkman.io\" | bash && sdk install kotlin".to_string())
                }
            }
            "swift" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["install", "Swift.Toolchain", "--source", "winget", "--silent", "--accept-package-agreements", "--accept-source-agreements"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("Swift 安装完成\n{}", out))
                } else if cfg!(target_os = "macos") {
                    Ok("Swift 已通过 Xcode 内置，无需单独安装".to_string())
                } else {
                    Err("请访问 https://swift.org/install 下载安装".to_string())
                }
            }
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("安装版本管理器失败: {}", e))?;

    result
}

/// 卸载环境版本
#[tauri::command]
pub async fn uninstall_env_version(env_name: String, version: String) -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        match env_name.as_str() {
            "nodejs" => {
                run_cmd_full("nvm", &["uninstall", &version])
                    .map(|out| format!("Node.js v{} 已卸载\n{}", version, out))
            }
            "python" => {
                run_cmd_full("pyenv", &["uninstall", "-f", &version])
                    .map(|out| format!("Python v{} 已卸载\n{}", version, out))
            }
            "rust" => {
                run_cmd_full("rustup", &["toolchain", "uninstall", &version])
                    .map(|out| format!("Rust {} 已卸载\n{}", version, out))
            }
            "go" => {
                if cfg!(target_os = "windows") {
                    run_cmd_full("winget", &["uninstall", "GoLang.Go", "--source", "winget"])
                        .map(|out| format!("Go 已卸载\n{}", out))
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["uninstall", "go"])
                        .map(|out| format!("Go 已卸载\n{}", out))
                } else {
                    Err("请手动运行: sudo apt remove golang-go".to_string())
                }
            }
            "java" => {
                if cfg!(target_os = "windows") {
                    // 提取主版本号: "21.0.8" → "21"
                    let major = version.split('.').next().unwrap_or(&version);
                    run_cmd_full("scoop", &["uninstall", &format!("openjdk{}", major)])
                        .map(|out| format!("Java JDK {} 已卸载\n{}", version, out))
                } else {
                    run_cmd_full("sdk", &["uninstall", "java", &version])
                        .map(|out| format!("Java {} 已卸载\n{}", version, out))
                }
            }
            "cpp" => {
                if cfg!(target_os = "windows") {
                    run_cmd_full("pacman", &["-R", "--noconfirm", "mingw-w64-x86_64-gcc"])
                        .map(|out| format!("GCC 已卸载\n{}", out))
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["uninstall", "gcc"])
                        .map(|out| format!("GCC 已卸载\n{}", out))
                } else {
                    run_cmd_full("sudo", &["apt", "remove", "-y", "gcc", "g++"])
                        .map(|out| format!("GCC/G++ 已卸载\n{}", out))
                }
            }
            "dotnet" => {
                if cfg!(target_os = "windows") {
                    // 尝试匹配主版本号卸载
                    let major = version.split('.').next().unwrap_or(&version);
                    run_cmd_full("winget", &["uninstall", &format!("Microsoft.DotNet.SDK.{}", major), "--source", "winget"])
                        .map(|out| format!(".NET SDK {} 已卸载\n{}", version, out))
                } else {
                    Err("请手动卸载: sudo apt remove dotnet-sdk-*".to_string())
                }
            }
            "php" => {
                if cfg!(target_os = "windows") {
                    // PHP 可能以 php 或 php84/php83 等名称安装
                    // 从版本号构建可能的包名: "8.4.16" → "php84"
                    let parts: Vec<&str> = version.split('.').collect();
                    let versioned_pkg = if parts.len() >= 2 {
                        format!("php{}{}", parts[0], parts[1])
                    } else {
                        format!("php{}", version.replace('.', ""))
                    };
                    // 先尝试版本包名 (php84)，再尝试通用包名 (php)
                    let result = run_cmd_full("scoop", &["uninstall", &versioned_pkg]);
                    match &result {
                        Ok(out) if !out.contains("ERROR") => Ok(format!("PHP 已卸载\n{}", out)),
                        _ => {
                            run_cmd_full("scoop", &["uninstall", "php"])
                                .map(|out| format!("PHP 已卸载\n{}", out))
                        }
                    }
                } else if cfg!(target_os = "macos") {
                    run_cmd_full("brew", &["uninstall", "php"])
                        .map(|out| format!("PHP 已卸载\n{}", out))
                } else {
                    Err("请手动运行: sudo apt remove php*".to_string())
                }
            }
            "kotlin" => {
                if cfg!(target_os = "windows") {
                    run_cmd_full("scoop", &["uninstall", "kotlin"])
                        .map(|out| format!("Kotlin 已卸载\n{}", out))
                } else {
                    run_cmd_full("sdk", &["uninstall", "kotlin", &version])
                        .map(|out| format!("Kotlin {} 已卸载\n{}", version, out))
                }
            }
            "swift" => {
                if cfg!(target_os = "windows") {
                    run_cmd_full("winget", &["uninstall", "Swift.Toolchain", "--source", "winget"])
                        .map(|out| format!("Swift 已卸载\n{}", out))
                } else if cfg!(target_os = "macos") {
                    Err("Swift 为 Xcode 内置组件，无法单独卸载".to_string())
                } else {
                    Err("请手动运行: sudo apt remove swiftlang".to_string())
                }
            }
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("卸载失败: {}", e))?;

    result
}

/// 卸载版本管理器
#[tauri::command]
pub async fn uninstall_version_manager(env_name: String) -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        match env_name.as_str() {
            "nodejs" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["uninstall", "CoreyButler.NVMforWindows", "--source", "winget"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("nvm-windows 已卸载\n{}", out))
                } else {
                    Err("请手动卸载 nvm: rm -rf ~/.nvm 并清理 shell 配置".to_string())
                }
            }
            "python" => {
                if cfg!(target_os = "windows") {
                    // 卸载 pyenv-win
                    let result = run_cmd_full("pip", &["uninstall", "pyenv-win", "-y"]);
                    // 清理环境变量
                    let ps_cmd = "[Environment]::SetEnvironmentVariable('PYENV','','User');\
                        [Environment]::SetEnvironmentVariable('PYENV_HOME','','User');\
                        [Environment]::SetEnvironmentVariable('PYENV_ROOT','','User');\
                        $p=[Environment]::GetEnvironmentVariable('Path','User');\
                        $p=($p -split ';' | Where-Object { $_ -notlike '*pyenv*' }) -join ';';\
                        [Environment]::SetEnvironmentVariable('Path',$p,'User')";
                    let _ = run_powershell_hidden(ps_cmd);
                    let _ = refresh_env_path();
                    result.map(|out| format!("pyenv-win 已卸载\n{}", out))
                        .map_err(|e| format!("pyenv-win 卸载失败: {}", e))
                } else {
                    Err("请手动卸载 pyenv: rm -rf ~/.pyenv".to_string())
                }
            }
            "rust" => {
                // rustup 自带卸载命令
                run_cmd_full("rustup", &["self", "uninstall", "-y"])
                    .map(|out| format!("rustup 已卸载\n{}", out))
            }
            "go" => {
                if cfg!(target_os = "windows") {
                    let result = run_cmd_full("winget", &["uninstall", "GoLang.Go", "--source", "winget"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("Go 已卸载\n{}", out))
                } else {
                    Err("请手动卸载 Go 或 goenv".to_string())
                }
            }
            "java" => {
                if cfg!(target_os = "windows") {
                    // 卸载 scoop（若不在 PATH 则用 shims 路径调用；若不存在则视为已卸载）
                    let result = scoop_uninstall_manager_safely();
                    if result.is_ok() {
                        let ps_cmd = "$p=[Environment]::GetEnvironmentVariable('Path','User');$p=($p -split ';' | Where-Object { $_ -notlike '*\\\\scoop\\\\shims*' }) -join ';';[Environment]::SetEnvironmentVariable('Path',$p,'User')";
                        let _ = run_powershell_hidden(ps_cmd);
                        let _ = refresh_env_path();
                    }
                    result.map(|out| format!("Scoop 已卸载\n{}", out))
                } else {
                    Err("请手动卸载 SDKMAN: rm -rf ~/.sdkman".to_string())
                }
            }
            "cpp" => {
                if cfg!(target_os = "windows") {
                    // 卸载 MSYS2
                    let result = run_cmd_full("winget", &["uninstall", "MSYS2.MSYS2", "--source", "winget"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!("MSYS2 已卸载\n{}", out))
                } else if cfg!(target_os = "macos") {
                    Err("Xcode 命令行工具请通过系统设置卸载".to_string())
                } else {
                    Err("请手动运行: sudo apt remove build-essential".to_string())
                }
            }
            "dotnet" => {
                if cfg!(target_os = "windows") {
                    // .NET SDK 自身就是版本管理器，卸载最新版
                    let result = run_cmd_full("winget", &["uninstall", "Microsoft.DotNet.SDK.9", "--source", "winget"]);
                    if result.is_ok() { let _ = refresh_env_path(); }
                    result.map(|out| format!(".NET SDK 已卸载\n{}", out))
                } else {
                    Err("请参考 https://dotnet.microsoft.com 卸载指南".to_string())
                }
            }
            "php" => {
                if cfg!(target_os = "windows") {
                    let result = scoop_uninstall_manager_safely();
                    if result.is_ok() {
                        let ps_cmd = "$p=[Environment]::GetEnvironmentVariable('Path','User');$p=($p -split ';' | Where-Object { $_ -notlike '*\\\\scoop\\\\shims*' }) -join ';';[Environment]::SetEnvironmentVariable('Path',$p,'User')";
                        let _ = run_powershell_hidden(ps_cmd);
                        let _ = refresh_env_path();
                    }
                    result.map(|out| format!("Scoop 已卸载\n{}", out))
                } else {
                    Err("请手动卸载 phpbrew: rm -rf ~/.phpbrew".to_string())
                }
            }
            "kotlin" => {
                if cfg!(target_os = "windows") {
                    let result = scoop_uninstall_manager_safely();
                    if result.is_ok() {
                        let ps_cmd = "$p=[Environment]::GetEnvironmentVariable('Path','User');$p=($p -split ';' | Where-Object { $_ -notlike '*\\\\scoop\\\\shims*' }) -join ';';[Environment]::SetEnvironmentVariable('Path',$p,'User')";
                        let _ = run_powershell_hidden(ps_cmd);
                        let _ = refresh_env_path();
                    }
                    result.map(|out| format!("Scoop 已卸载\n{}", out))
                } else {
                    Err("请手动运行: sdk uninstall kotlin".to_string())
                }
            }
            "swift" => {
                if cfg!(target_os = "windows") {
                    // 若未安装 Swift.Toolchain，则视为已卸载，避免“找不到匹配包”报错
                    let listed = run_cmd("winget", &["list", "--id", "Swift.Toolchain", "--source", "winget"]).unwrap_or_default();
                    if listed.to_lowercase().contains("swift.toolchain") {
                        let result = run_cmd_full("winget", &["uninstall", "Swift.Toolchain", "--source", "winget"]);
                        if result.is_ok() { let _ = refresh_env_path(); }
                        result.map(|out| format!("Swift 已卸载\n{}", out))
                    } else {
                        Ok("Swift 未安装或已移除".to_string())
                    }
                } else if cfg!(target_os = "macos") {
                    Err("Swift 为 Xcode 内置组件，无法单独卸载".to_string())
                } else {
                    Err("请手动运行: sudo apt remove swiftlang".to_string())
                }
            }
            _ => Err(format!("未知的环境: {}", env_name)),
        }
    })
    .await
    .map_err(|e| format!("卸载版本管理器失败: {}", e))?;

    result
}

