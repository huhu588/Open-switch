// oh-my-opencode 配置和安装管理模块
// 支持一键安装 Bun 和 oh-my-opencode，以及配置 7 个 Agent 的模型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// Windows 平台特定：用于隐藏命令行窗口
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows CREATE_NO_WINDOW 标志
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// oh-my-opencode 安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhMyStatus {
    /// Bun 是否已安装
    pub bun_installed: bool,
    /// Bun 版本
    pub bun_version: Option<String>,
    /// npm 是否已安装
    pub npm_installed: bool,
    /// oh-my-opencode 是否已安装（通过检测配置文件）
    pub ohmy_installed: bool,
    /// 当前配置
    pub config: Option<OhMyConfig>,
}

/// oh-my-opencode 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhMyConfig {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub agents: HashMap<String, AgentConfig>,
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
}

/// 可用模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableModel {
    /// provider 名称
    pub provider_name: String,
    /// 模型 ID
    pub model_id: String,
    /// 显示名称（provider/model 格式）
    pub display_name: String,
}

/// Agent 信息（用于前端显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent 配置键名
    pub key: String,
    /// Agent 显示名称
    pub name: String,
    /// Agent 描述
    pub description: String,
    /// 用法示例
    pub usage: Option<String>,
}

impl Default for OhMyConfig {
    fn default() -> Self {
        Self {
            schema: Some("https://raw.githubusercontent.com/code-yeongyu/oh-my-opencode/master/assets/oh-my-opencode.schema.json".to_string()),
            agents: HashMap::new(),
        }
    }
}

/// 获取 oh-my-opencode 配置文件路径
fn get_ohmy_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".config").join("opencode").join("oh-my-opencode.json"))
}

/// 获取 opencode 配置文件路径
fn get_opencode_config_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".config").join("opencode").join("opencode.json"))
}

/// 检测 Bun 是否已安装（同时检查 PATH 和默认安装路径）
fn check_bun_installed() -> (bool, Option<String>) {
    // 首先尝试使用完整路径（处理刚安装但还没加入 PATH 的情况）
    if let Some(bun_path) = get_bun_path() {
        #[cfg(target_os = "windows")]
        let output = Command::new(&bun_path)
            .arg("--version")
            .creation_flags(CREATE_NO_WINDOW) // 隐藏终端窗口
            .output();
        
        #[cfg(not(target_os = "windows"))]
        let output = Command::new(&bun_path)
            .arg("--version")
            .output();
        
        if let Ok(out) = output {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return (true, Some(version));
            }
        }
    }
    
    // 然后尝试从 PATH 中查找
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", "bun", "--version"])
        .creation_flags(CREATE_NO_WINDOW) // 隐藏终端窗口
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bun")
        .arg("--version")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            (true, Some(version))
        }
        _ => (false, None),
    }
}

/// 检测 oh-my-opencode 是否已安装（通过检测 opencode.json 中的 plugins 配置）
fn check_ohmy_installed() -> bool {
    if let Ok(config_path) = get_opencode_config_path() {
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    // 检查 plugins 数组中是否包含 oh-my-opencode
                    if let Some(plugins) = json.get("plugins").and_then(|p| p.as_array()) {
                        return plugins.iter().any(|p| {
                            p.as_str().map(|s| s.contains("oh-my-opencode")).unwrap_or(false)
                        });
                    }
                }
            }
        }
    }
    
    // 备选：检测 oh-my-opencode.json 是否存在
    if let Ok(ohmy_path) = get_ohmy_config_path() {
        return ohmy_path.exists();
    }
    
    false
}

/// 读取 oh-my-opencode 配置
fn read_ohmy_config() -> Option<OhMyConfig> {
    let config_path = get_ohmy_config_path().ok()?;
    if !config_path.exists() {
        return None;
    }
    
    let content = fs::read_to_string(&config_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// 检测 oh-my-opencode 安装状态
#[tauri::command]
pub async fn check_ohmy_status() -> Result<OhMyStatus, String> {
    let (bun_installed, bun_version) = check_bun_installed();
    let npm_installed = check_npm_installed();
    let ohmy_installed = check_ohmy_installed();
    let config = if ohmy_installed { read_ohmy_config() } else { None };
    
    Ok(OhMyStatus {
        bun_installed,
        bun_version,
        npm_installed,
        ohmy_installed,
        config,
    })
}

/// 获取 OpenCode 内置的免费模型列表
fn get_builtin_free_models() -> Vec<AvailableModel> {
    vec![
        AvailableModel {
            provider_name: "OpenCode Zen".to_string(),
            model_id: "big-pickle".to_string(),
            display_name: "OpenCode Zen/Big Pickle".to_string(),
        },
        AvailableModel {
            provider_name: "OpenCode Zen".to_string(),
            model_id: "glm-4.7".to_string(),
            display_name: "OpenCode Zen/GLM-4.7".to_string(),
        },
        AvailableModel {
            provider_name: "OpenCode Zen".to_string(),
            model_id: "gpt-5-nano".to_string(),
            display_name: "OpenCode Zen/GPT-5 Nano".to_string(),
        },
        AvailableModel {
            provider_name: "OpenCode Zen".to_string(),
            model_id: "grok-code-fast-1".to_string(),
            display_name: "OpenCode Zen/Grok Code Fast 1".to_string(),
        },
        AvailableModel {
            provider_name: "OpenCode Zen".to_string(),
            model_id: "minimax-m2.1".to_string(),
            display_name: "OpenCode Zen/MiniMax M2.1".to_string(),
        },
    ]
}

/// 获取可用的模型列表（从 opencode.json 读取 + OpenCode 内置免费模型）
#[tauri::command]
pub async fn get_available_models() -> Result<Vec<AvailableModel>, String> {
    let mut models = Vec::new();
    
    // 1. 添加 OpenCode 内置的免费模型（放在最前面）
    models.extend(get_builtin_free_models());
    
    // 2. 从 opencode.json 读取用户配置的模型
    let config_path = get_opencode_config_path()?;
    
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // 遍历 provider 对象
                if let Some(providers) = json.get("provider").and_then(|p| p.as_object()) {
                    for (provider_name, provider_config) in providers {
                        // 遍历每个 provider 下的 models
                        if let Some(provider_models) = provider_config.get("models").and_then(|m| m.as_object()) {
                            for (model_id, _model_config) in provider_models {
                                models.push(AvailableModel {
                                    provider_name: provider_name.clone(),
                                    model_id: model_id.clone(),
                                    display_name: format!("{}/{}", provider_name, model_id),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 用户配置的模型按 display_name 排序（内置模型保持在前面）
    let builtin_count = get_builtin_free_models().len();
    if models.len() > builtin_count {
        models[builtin_count..].sort_by(|a, b| a.display_name.cmp(&b.display_name));
    }
    
    Ok(models)
}

/// 获取 7 个 Agent 的信息
#[tauri::command]
pub async fn get_agent_infos() -> Result<Vec<AgentInfo>, String> {
    Ok(vec![
        AgentInfo {
            key: "Sisyphus".to_string(),
            name: "Sisyphus".to_string(),
            description: "主要编排者".to_string(),
            usage: None,
        },
        AgentInfo {
            key: "oracle".to_string(),
            name: "Oracle".to_string(),
            description: "架构设计、代码审查和策略制定".to_string(),
            usage: Some("Ask @oracle to review this design and propose an architecture".to_string()),
        },
        AgentInfo {
            key: "librarian".to_string(),
            name: "Librarian".to_string(),
            description: "多仓库分析、文档查找和实现示例".to_string(),
            usage: Some("Ask @librarian how this is implemented—why does the behavior keep changing?".to_string()),
        },
        AgentInfo {
            key: "explore".to_string(),
            name: "Explore".to_string(),
            description: "快速代码库探索和模式匹配".to_string(),
            usage: Some("Ask @explore for the policy on this feature".to_string()),
        },
        AgentInfo {
            key: "frontend-ui-ux-engineer".to_string(),
            name: "Frontend".to_string(),
            description: "前端 UI/UX 开发".to_string(),
            usage: Some("委托构建精美的用户界面".to_string()),
        },
        AgentInfo {
            key: "document-writer".to_string(),
            name: "Document Writer".to_string(),
            description: "技术文档编写".to_string(),
            usage: None,
        },
        AgentInfo {
            key: "multimodal-looker".to_string(),
            name: "Multimodal Looker".to_string(),
            description: "多模态内容查看".to_string(),
            usage: None,
        },
    ])
}

/// 检测 npm/npx 是否已安装
fn check_npm_installed() -> bool {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", "npm", "--version"])
        .creation_flags(CREATE_NO_WINDOW) // 隐藏终端窗口
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("npm")
        .arg("--version")
        .output();
    
    matches!(output, Ok(out) if out.status.success())
}

/// 安装 Bun
#[tauri::command]
pub async fn install_bun() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-Command", 
            "irm bun.sh/install.ps1 | iex"
        ])
        .creation_flags(CREATE_NO_WINDOW) // 隐藏终端窗口
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bash")
        .args(["-c", "curl -fsSL https://bun.sh/install | bash"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            
            if out.status.success() {
                Ok(format!("Bun 安装成功\n{}", stdout))
            } else {
                // 提供更详细的错误信息
                Err(format!(
                    "Bun 安装失败\n退出码: {:?}\n标准输出: {}\n错误输出: {}\n\n提示: 您可以手动安装 Bun，然后再试。或者系统已有 npm/npx，将尝试使用 npx 安装。",
                    out.status.code(),
                    stdout,
                    stderr
                ))
            }
        }
        Err(e) => Err(format!("执行安装命令失败: {}\n\n提示: 请确保系统已安装 PowerShell。", e)),
    }
}

/// 获取 Bun 可执行文件的完整路径
fn get_bun_path() -> Option<PathBuf> {
    // Windows: 检查用户目录下的 .bun/bin/bun.exe
    if cfg!(target_os = "windows") {
        if let Some(home) = dirs::home_dir() {
            let bun_path = home.join(".bun").join("bin").join("bun.exe");
            if bun_path.exists() {
                return Some(bun_path);
            }
        }
    } else {
        // macOS/Linux: 检查 ~/.bun/bin/bun
        if let Some(home) = dirs::home_dir() {
            let bun_path = home.join(".bun").join("bin").join("bun");
            if bun_path.exists() {
                return Some(bun_path);
            }
        }
    }
    None
}

/// 安装 oh-my-opencode (需要 Bun 运行时)
#[tauri::command]
pub async fn install_ohmy() -> Result<String, String> {
    // 尝试获取 Bun 的完整路径（处理刚安装但还没加入 PATH 的情况）
    let bun_cmd = if let Some(bun_path) = get_bun_path() {
        bun_path.to_string_lossy().to_string()
    } else {
        // 回退到使用系统 PATH 中的 bun
        "bun".to_string()
    };
    
    // 使用 bun x 而不是 bunx（更可靠）
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", &bun_cmd, "x", "oh-my-opencode", "install"])
        .creation_flags(CREATE_NO_WINDOW) // 隐藏终端窗口
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let output = Command::new(&bun_cmd)
        .args(["x", "oh-my-opencode", "install"])
        .output();
    
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            
            if out.status.success() {
                Ok(format!("✓ oh-my-opencode 安装成功\n{}", stdout))
            } else {
                Err(format!(
                    "bun x oh-my-opencode install 失败\n退出码: {:?}\n输出: {}\n错误: {}\n\n使用的 Bun 路径: {}",
                    out.status.code(),
                    stdout,
                    stderr,
                    bun_cmd
                ))
            }
        }
        Err(e) => Err(format!(
            "执行 bun 命令失败: {}\n\n使用的路径: {}\n\n请重启应用后再试，或手动将 Bun 添加到 PATH 环境变量。",
            e,
            bun_cmd
        )),
    }
}

/// 保存 oh-my-opencode 配置
#[tauri::command]
pub async fn save_ohmy_config(agents: HashMap<String, String>) -> Result<(), String> {
    let config_path = get_ohmy_config_path()?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    // 构建配置
    let mut agent_configs = HashMap::new();
    for (key, model) in agents {
        agent_configs.insert(key, AgentConfig { model });
    }
    
    let config = OhMyConfig {
        schema: Some("https://raw.githubusercontent.com/code-yeongyu/oh-my-opencode/master/assets/oh-my-opencode.schema.json".to_string()),
        agents: agent_configs,
    };
    
    // 写入文件
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    Ok(())
}

/// 一键安装并配置
#[tauri::command]
pub async fn install_and_configure(agents: HashMap<String, String>) -> Result<String, String> {
    let mut log = String::new();
    
    // 1. 检测 Bun（oh-my-opencode 需要 Bun 运行时）
    let (bun_installed, bun_version) = check_bun_installed();
    
    if bun_installed {
        log.push_str(&format!("✓ Bun 已安装 ({})\n", bun_version.unwrap_or_default()));
    } else {
        // 必须安装 Bun，因为 oh-my-opencode 依赖 Bun 运行时
        log.push_str("⚠ Bun 未安装，oh-my-opencode 需要 Bun 运行时\n");
        log.push_str("正在安装 Bun...\n");
        
        match install_bun().await {
            Ok(msg) => {
                log.push_str(&format!("{}\n", msg));
            }
            Err(e) => {
                log.push_str(&format!("✗ Bun 安装失败: {}\n", e));
                return Err(format!(
                    "{}\n\n❌ 安装失败：oh-my-opencode 需要 Bun 运行时。\n\n\
                    请手动安装 Bun：\n\
                    方法 1: 在 PowerShell 中运行:\n\
                    powershell -ExecutionPolicy Bypass -c \"irm bun.sh/install.ps1|iex\"\n\n\
                    方法 2: 访问 https://bun.sh 下载安装\n\n\
                    安装完成后重启终端和本应用再试。",
                    log
                ));
            }
        }
    }
    
    // 2. 安装 oh-my-opencode
    log.push_str("正在安装 oh-my-opencode...\n");
    match install_ohmy().await {
        Ok(msg) => log.push_str(&format!("{}\n", msg)),
        Err(e) => return Err(format!("安装 oh-my-opencode 失败: {}\n\n{}", e, log)),
    }
    
    // 3. 保存配置
    log.push_str("正在保存配置...\n");
    save_ohmy_config(agents).await?;
    log.push_str("✓ 配置已保存！\n");
    
    log.push_str("\n🎉 oh-my-opencode 安装配置完成！\n");
    
    Ok(log)
}

/// 卸载 oh-my-opencode
#[tauri::command]
pub async fn uninstall_ohmy() -> Result<String, String> {
    let mut log = String::new();
    
    // 1. 删除 oh-my-opencode.json 配置文件
    let ohmy_config_path = get_ohmy_config_path()?;
    if ohmy_config_path.exists() {
        fs::remove_file(&ohmy_config_path)
            .map_err(|e| format!("删除 oh-my-opencode.json 失败: {}", e))?;
        log.push_str("已删除 oh-my-opencode.json 配置文件\n");
    }
    
    // 2. 从 opencode.json 中移除 plugins 数组中的 oh-my-opencode 项
    let opencode_config_path = get_opencode_config_path()?;
    if opencode_config_path.exists() {
        let content = fs::read_to_string(&opencode_config_path)
            .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;
        
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
            let mut modified = false;
            
            // 移除 plugins 数组中的 oh-my-opencode
            if let Some(plugins) = json.get_mut("plugins").and_then(|p| p.as_array_mut()) {
                let original_len = plugins.len();
                plugins.retain(|p| {
                    !p.as_str().map(|s| s.contains("oh-my-opencode")).unwrap_or(false)
                });
                if plugins.len() != original_len {
                    modified = true;
                    log.push_str("已从 opencode.json 中移除 oh-my-opencode 插件\n");
                }
                
                // 如果 plugins 数组为空，删除该字段
                if plugins.is_empty() {
                    if let Some(obj) = json.as_object_mut() {
                        obj.remove("plugins");
                    }
                }
            }
            
            if modified {
                let new_content = serde_json::to_string_pretty(&json)
                    .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;
                fs::write(&opencode_config_path, new_content)
                    .map_err(|e| format!("写入 opencode.json 失败: {}", e))?;
            }
        }
    }
    
    log.push_str("oh-my-opencode 卸载完成！\n");
    Ok(log)
}
