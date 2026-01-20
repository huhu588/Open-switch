// Skills 管理相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::AppError;

// ==================== skills仓库管理 ====================

/// 技能仓库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsRepository {
    /// 仓库 ID（唯一标识）
    pub id: String,
    /// 仓库显示名称
    pub name: String,
    /// 仓库 URL（GitHub 仓库地址）
    pub url: String,
    /// 技能列表索引文件的 raw URL
    pub index_url: String,
    /// 是否为内置仓库
    #[serde(default)]
    pub builtin: bool,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// 从 raw.githubusercontent.com URL 中提取分支名
/// 格式: https://raw.githubusercontent.com/owner/repo/branch/path/to/file
fn extract_branch_from_url(url: &str) -> Option<String> {
    let url = url.trim_start_matches("https://raw.githubusercontent.com/");
    let parts: Vec<&str> = url.split('/').collect();
    // parts[0] = owner, parts[1] = repo, parts[2] = branch
    if parts.len() >= 3 {
        Some(parts[2].to_string())
    } else {
        None
    }
}

/// 从仓库索引获取的技能信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    #[serde(default)]
    pub id: String,
    #[serde(alias = "title")]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    /// SKILL.md 文件的相对路径
    #[serde(default, alias = "file")]
    pub path: String,
}

/// 仓库索引文件格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsIndex {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub skills: Vec<RemoteSkill>,
}

/// 获取技能仓库配置文件路径
fn get_repos_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("opencode").join("skills-repos.json"))
}

/// 获取内置仓库列表
fn get_builtin_repos() -> Vec<SkillsRepository> {
    vec![
        // Anthropic 官方 Skills 仓库
        SkillsRepository {
            id: "anthropics-skills".to_string(),
            name: "Anthropic Official".to_string(),
            url: "https://github.com/anthropics/skills".to_string(),
            index_url: "https://raw.githubusercontent.com/anthropics/skills/main/index.json".to_string(),
            builtin: true,
            enabled: true,
        },
        // obra/superpowers - 软件开发超能力
        SkillsRepository {
            id: "obra-superpowers".to_string(),
            name: "Superpowers".to_string(),
            url: "https://github.com/obra/superpowers".to_string(),
            index_url: "https://raw.githubusercontent.com/obra/superpowers/main/index.json".to_string(),
            builtin: true,
            enabled: true,
        },
        // cexll/myclaude 社区技能仓库
        SkillsRepository {
            id: "cexll-myclaude".to_string(),
            name: "MyClaude Skills".to_string(),
            url: "https://github.com/cexll/myclaude".to_string(),
            index_url: "https://raw.githubusercontent.com/cexll/myclaude/master/skills/index.json".to_string(),
            builtin: true,
            enabled: true,
        },
    ]
}

/// 加载技能仓库列表
fn load_skills_repos() -> Vec<SkillsRepository> {
    let mut repos = get_builtin_repos();
    
    // 加载用户自定义仓库
    if let Some(config_path) = get_repos_config_path() {
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(user_repos) = serde_json::from_str::<Vec<SkillsRepository>>(&content) {
                    repos.extend(user_repos);
                }
            }
        }
    }
    
    // 应用用户保存的启用状态
    let enabled_states = load_repo_enabled_states();
    for repo in &mut repos {
        if let Some(&enabled) = enabled_states.get(&repo.id) {
            repo.enabled = enabled;
        }
    }
    
    repos
}

/// 保存用户自定义仓库
fn save_user_repos(repos: &[SkillsRepository]) -> Result<(), AppError> {
    let config_path = get_repos_config_path()
        .ok_or_else(|| AppError::Custom("无法获取配置路径".to_string()))?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Custom(format!("创建目录失败: {}", e)))?;
    }
    
    // 只保存非内置仓库
    let user_repos: Vec<_> = repos.iter().filter(|r| !r.builtin).cloned().collect();
    let content = serde_json::to_string_pretty(&user_repos)
        .map_err(|e| AppError::Custom(format!("序列化失败: {}", e)))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| AppError::Custom(format!("写入文件失败: {}", e)))?;
    
    Ok(())
}

/// 获取所有技能仓库
#[tauri::command]
pub fn get_skills_repos() -> Vec<SkillsRepository> {
    load_skills_repos()
}

/// 添加技能仓库
#[tauri::command]
pub fn add_skills_repo(repo: SkillsRepository) -> Result<Vec<SkillsRepository>, AppError> {
    let mut repos = load_skills_repos();
    
    // 检查是否已存在
    if repos.iter().any(|r| r.id == repo.id || r.url == repo.url) {
        return Err(AppError::Custom("仓库已存在".to_string()));
    }
    
    repos.push(repo);
    save_user_repos(&repos)?;
    
    Ok(repos)
}

/// 删除技能仓库
#[tauri::command]
pub fn delete_skills_repo(repo_id: String) -> Result<Vec<SkillsRepository>, AppError> {
    let repos = load_skills_repos();
    
    // 检查是否为内置仓库
    if let Some(repo) = repos.iter().find(|r| r.id == repo_id) {
        if repo.builtin {
            return Err(AppError::Custom("无法删除内置仓库".to_string()));
        }
    }
    
    let remaining: Vec<_> = repos.into_iter().filter(|r| r.id != repo_id).collect();
    save_user_repos(&remaining)?;
    
    Ok(remaining)
}

/// 获取仓库启用状态配置路径
fn get_repo_enabled_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("opencode").join("skills-repos-enabled.json"))
}

/// 加载仓库启用状态
fn load_repo_enabled_states() -> std::collections::HashMap<String, bool> {
    if let Some(config_path) = get_repo_enabled_config_path() {
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(states) = serde_json::from_str(&content) {
                    return states;
                }
            }
        }
    }
    std::collections::HashMap::new()
}

/// 保存仓库启用状态
fn save_repo_enabled_states(states: &std::collections::HashMap<String, bool>) -> Result<(), AppError> {
    let config_path = get_repo_enabled_config_path()
        .ok_or_else(|| AppError::Custom("无法获取配置路径".to_string()))?;
    
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Custom(format!("创建目录失败: {}", e)))?;
    }
    
    let content = serde_json::to_string_pretty(states)
        .map_err(|e| AppError::Custom(format!("序列化失败: {}", e)))?;
    
    std::fs::write(&config_path, content)
        .map_err(|e| AppError::Custom(format!("写入文件失败: {}", e)))?;
    
    Ok(())
}

/// 切换仓库启用状态
#[tauri::command]
pub fn toggle_skills_repo_enabled(repo_id: String, enabled: bool) -> Result<Vec<SkillsRepository>, AppError> {
    // 加载当前状态
    let mut states = load_repo_enabled_states();
    states.insert(repo_id.clone(), enabled);
    
    // 保存状态
    save_repo_enabled_states(&states)?;
    
    // 返回更新后的仓库列表
    Ok(load_skills_repos())
}

/// 切换仓库启用状态（旧版本，保留兼容性）
#[tauri::command]
pub fn toggle_skills_repo(repo_id: String) -> Result<Vec<SkillsRepository>, AppError> {
    let repos = load_skills_repos();
    
    // 找到当前状态并取反
    let current_enabled = repos.iter()
        .find(|r| r.id == repo_id)
        .map(|r| r.enabled)
        .unwrap_or(true);
    
    let new_enabled = !current_enabled;
    
    // 使用新的状态保存系统
    let mut states = load_repo_enabled_states();
    states.insert(repo_id, new_enabled);
    save_repo_enabled_states(&states)?;
    
    Ok(load_skills_repos())
}

/// 从远程仓库获取技能列表
#[tauri::command]
pub async fn fetch_skills_from_repo(repo_id: String) -> Result<Vec<RecommendedSkills>, AppError> {
    let repos = load_skills_repos();
    let repo = repos.iter()
        .find(|r| r.id == repo_id)
        .ok_or_else(|| AppError::Custom("仓库不存在".to_string()))?;
    
    if !repo.enabled {
        return Err(AppError::Custom("仓库已禁用".to_string()));
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Custom(format!("创建 HTTP 客户端失败: {}", e)))?;
    
    // 尝试获取 index.json
    let response = client.get(&repo.index_url)
        .header("User-Agent", "Open-Switch/1.0")
        .send()
        .await
        .map_err(|e| AppError::Custom(format!("请求失败: {}", e)))?;
    
    if !response.status().is_success() {
        // 如果没有 index.json，尝试扫描仓库目录
        return fetch_skills_by_scanning(&client, repo).await;
    }
    
    let body = response.text().await
        .map_err(|e| AppError::Custom(format!("读取响应失败: {}", e)))?;
    
    let index: SkillsIndex = serde_json::from_str(&body)
        .map_err(|e| AppError::Custom(format!("解析索引文件失败: {}", e)))?;
    
    // 从 index_url 中提取分支名
    let branch = extract_branch_from_url(&repo.index_url).unwrap_or("main".to_string());
    
    // 转换为 RecommendedSkills
    let base_raw_url = repo.url
        .replace("github.com", "raw.githubusercontent.com")
        .trim_end_matches('/')
        .to_string() + "/" + &branch + "/";
    
    let skills = index.skills.into_iter()
        .filter(|s| !s.name.is_empty())
        .map(|s| {
            let id = if s.id.is_empty() {
                s.name.to_lowercase().replace(' ', "-")
            } else {
                s.id.clone()
            };
            let raw_url = format!("{}{}", base_raw_url, s.path);
            RecommendedSkills {
                id,
                name: s.name,
                description: s.description,
                category: s.category,
                repo: repo.name.clone(),
                repo_url: repo.url.clone(),
                raw_url,
            }
        }).collect();
    
    Ok(skills)
}

/// 检测仓库的默认分支
async fn detect_default_branch(client: &reqwest::Client, owner: &str, repo_name: &str) -> String {
    // 尝试通过 GitHub API 获取默认分支
    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo_name);
    
    if let Ok(response) = client.get(&api_url)
        .header("User-Agent", "Open-Switch/1.0")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
    {
        if response.status().is_success() {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(branch) = json.get("default_branch").and_then(|b| b.as_str()) {
                    return branch.to_string();
                }
            }
        }
    }
    
    // 默认使用 main
    "main".to_string()
}

/// 通过 GitHub API 扫描仓库目录获取技能
async fn fetch_skills_by_scanning(client: &reqwest::Client, repo: &SkillsRepository) -> Result<Vec<RecommendedSkills>, AppError> {
    let parts: Vec<&str> = repo.url.trim_end_matches('/').split('/').collect();
    if parts.len() < 2 {
        return Err(AppError::Custom("无效的仓库 URL".to_string()));
    }
    
    let owner = parts[parts.len() - 2];
    let repo_name = parts[parts.len() - 1];
    
    let mut all_skills = Vec::new();
    
    let possible_paths = vec!["skills", "skill", "prompts", "claude-skills"];
    
    for path in &possible_paths {
        let api_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo_name, path
        );
        
        let response = match client.get(&api_url)
            .header("User-Agent", "Open-Switch/1.0")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await {
                Ok(r) => r,
                Err(_) => continue,
            };
        
        let status = response.status();
        
        // 处理 GitHub API 速率限制
        if status.as_u16() == 403 {
            if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
                if remaining.to_str().unwrap_or("1") == "0" {
                    return Err(AppError::Custom("GitHub API 速率限制已达上限，请稍后再试".to_string()));
                }
            }
            continue;
        }
        
        if !status.is_success() {
            continue;
        }
        
        #[derive(Deserialize)]
        struct GitHubContent {
            name: String,
            #[serde(rename = "type")]
            content_type: String,
            path: String,
        }
        
        let contents: Vec<GitHubContent> = match response.json().await {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        // 尝试从 index_url 提取分支，如果失败则检测默认分支
        let branch = match extract_branch_from_url(&repo.index_url) {
            Some(b) => b,
            None => detect_default_branch(client, owner, repo_name).await,
        };
        let base_raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/",
            owner, repo_name, branch
        );
        
        // GitHub 仓库路径，如 "anthropics/skills"
        let github_path = format!("{}/{}", owner, repo_name);
        
        for content in contents {
            if content.content_type == "dir" {
                let formatted_name = content.name
                    .split('-')
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                
                all_skills.push(RecommendedSkills {
                    id: content.name.clone(),
                    name: formatted_name,
                    description: github_path.clone(), // 使用 GitHub 路径作为来源标识
                    category: "community".to_string(),
                    repo: repo.name.clone(),
                    repo_url: repo.url.clone(),
                    raw_url: format!("{}{}/SKILL.md", base_raw_url, content.path),
                });
            } else if content.name.ends_with(".md") && content.name != "README.md" {
                let skills_name = content.name.trim_end_matches(".md");
                let formatted_name = skills_name
                    .split('-')
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                
                all_skills.push(RecommendedSkills {
                    id: skills_name.to_string(),
                    name: formatted_name,
                    description: github_path.clone(), // 使用 GitHub 路径作为来源标识
                    category: "community".to_string(),
                    repo: repo.name.clone(),
                    repo_url: repo.url.clone(),
                    raw_url: format!("{}{}", base_raw_url, content.path),
                });
            }
        }
        
        if !all_skills.is_empty() {
            break;
        }
    }
    
    Ok(all_skills)
}

/// 从所有启用的仓库获取技能列表
#[tauri::command]
pub async fn discover_skills() -> Result<Vec<RecommendedSkills>, AppError> {
    let repos = load_skills_repos();
    let enabled_repos: Vec<_> = repos.into_iter().filter(|r| r.enabled).collect();
    
    if enabled_repos.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut all_skills = Vec::new();
    let mut failed_repos = Vec::new();
    
    for repo in &enabled_repos {
        match fetch_skills_from_repo(repo.id.clone()).await {
            Ok(skills) => {
                all_skills.extend(skills);
            }
            Err(e) => {
                // 记录失败的仓库，但继续处理其他仓库
                failed_repos.push(format!("{}: {}", repo.name, e));
            }
        }
    }
    
    // 只有当所有仓库都失败时才返回错误
    // 如果有仓库成功（即使返回空列表），则正常返回结果
    if failed_repos.len() == enabled_repos.len() && !enabled_repos.is_empty() {
        return Err(AppError::Custom(format!(
            "获取技能失败: {}",
            failed_repos.join("; ")
        )));
    }
    
    Ok(all_skills)
}

/// 已安装的 Skills 信息
#[derive(Debug, Clone, Serialize)]
pub struct InstalledSkills {
    pub name: String,
    pub path: String,
    pub location: SkillsLocation,
    pub content_preview: String,
}

/// skills 安装位置类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillsLocation {
    /// 项目 OpenCode: .opencode/skills/<name>/SKILL.md
    ProjectOpenCode,
    /// 全局 OpenCode: ~/.config/opencode/skills/<name>/SKILL.md
    GlobalOpenCode,
    /// 项目 Claude: .claude/skills/<name>/SKILL.md
    ProjectClaude,
    /// 全局 Claude: ~/.claude/skills/<name>/SKILL.md
    GlobalClaude,
}

impl std::fmt::Display for SkillsLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillsLocation::ProjectOpenCode => write!(f, "项目 OpenCode"),
            SkillsLocation::GlobalOpenCode => write!(f, "全局 OpenCode"),
            SkillsLocation::ProjectClaude => write!(f, "项目 Claude"),
            SkillsLocation::GlobalClaude => write!(f, "全局 Claude"),
        }
    }
}

/// 推荐的 Skills 信息
#[derive(Debug, Clone, Serialize)]
pub struct RecommendedSkills {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub repo: String,
    pub repo_url: String,
    pub raw_url: String,
}

/// 安装 Skills 的输入参数
#[derive(Debug, Deserialize)]
pub struct InstallSkillsInput {
    pub skill_id: String,
    pub raw_url: String,
    pub location: String,
}

/// 安装结果
#[derive(Debug, Serialize)]
pub struct InstallSkillsResult {
    pub success: bool,
    pub message: String,
    pub installed_path: Option<String>,
}

/// 获取所有 Skills 扫描路径
fn get_skills_paths() -> Vec<(PathBuf, SkillsLocation)> {
    let mut paths = Vec::new();
    
    if let Some(home_dir) = dirs::home_dir() {
        let global_opencode_path = home_dir.join(".config").join("opencode").join("skills");
        let global_claude_path = home_dir.join(".claude").join("skills");

        paths.push((global_opencode_path, SkillsLocation::GlobalOpenCode));
        paths.push((global_claude_path, SkillsLocation::GlobalClaude));
    }
    
    if let Ok(cwd) = std::env::current_dir() {
        paths.push((cwd.join(".opencode").join("skills"), SkillsLocation::ProjectOpenCode));
        paths.push((cwd.join(".claude").join("skills"), SkillsLocation::ProjectClaude));
    }
    
    paths
}

/// 扫描已安装的 Skills
#[tauri::command]
pub fn get_installed_skills() -> Result<Vec<InstalledSkills>, AppError> {
    let mut skills = Vec::new();
    
    for (base_path, location) in get_skills_paths() {
        if !base_path.exists() {
            continue;
        }
        
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skills_file = path.join("SKILL.md");
                    if skills_file.exists() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        let content_preview = std::fs::read_to_string(&skills_file)
                            .map(|c| {
                                let preview: String = c.chars().take(200).collect();
                                if c.len() > 200 {
                                    format!("{}...", preview)
                                } else {
                                    preview
                                }
                            })
                            .unwrap_or_default();
                        
                        skills.push(InstalledSkills {
                            name,
                            path: skills_file.to_string_lossy().to_string(),
                            location: location.clone(),
                            content_preview,
                        });
                    }
                }
            }
        }
    }
    
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(skills)
}

/// 获取推荐的 Skills 列表
#[tauri::command]
pub fn get_recommended_skills() -> Vec<RecommendedSkills> {
    vec![
        RecommendedSkills {
            id: "skill-creator".to_string(),
            name: "Skill Creator".to_string(),
            description: "Skill 创建器 - 交互式工具，指导您通过问答创建新 Skill".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/skill-creator/SKILL.md".to_string(),
        },
        RecommendedSkills {
            id: "mcp-builder".to_string(),
            name: "MCP Builder".to_string(),
            description: "MCP 服务器构建器 - 帮助创建 MCP 服务器".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/mcp-builder/SKILL.md".to_string(),
        },
        RecommendedSkills {
            id: "frontend-design".to_string(),
            name: "Frontend Design".to_string(),
            description: "前端设计 - 创建精美、生产级的 Web 界面".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/frontend-design/SKILL.md".to_string(),
        },
    ]
}

/// 安装 Skills
#[tauri::command]
pub async fn install_skills(input: InstallSkillsInput) -> Result<InstallSkillsResult, AppError> {
    let base_path = match input.location.as_str() {
        "global_opencode" => {
            dirs::home_dir()
                .ok_or_else(|| AppError::Custom("无法获取主目录".to_string()))?
                .join(".config")
                .join("opencode")
                .join("skills")
        }
        "project_opencode" => {
            std::env::current_dir()
                .map_err(|e| AppError::Custom(format!("无法获取当前目录: {}", e)))?
                .join(".opencode")
                .join("skills")
        }
        "global_claude" => {
            dirs::home_dir()
                .ok_or_else(|| AppError::Custom("无法获取主目录".to_string()))?
                .join(".claude")
                .join("skills")
        }
        "project_claude" => {
            std::env::current_dir()
                .map_err(|e| AppError::Custom(format!("无法获取当前目录: {}", e)))?
                .join(".claude")
                .join("skills")
        }
        _ => return Err(AppError::Custom("无效的安装位置".to_string())),
    };
    
    let skills_dir = base_path.join(&input.skill_id);
    std::fs::create_dir_all(&skills_dir)
        .map_err(|e| AppError::Custom(format!("创建目录失败: {}", e)))?;
    
    // 定义清理函数，在失败时删除空目录
    let cleanup_on_failure = |dir: &std::path::Path| {
        // 只有目录为空时才删除
        if let Ok(entries) = std::fs::read_dir(dir) {
            if entries.count() == 0 {
                let _ = std::fs::remove_dir(dir);
            }
        }
    };
    
    let client = reqwest::Client::new();
    let response = match client.get(&input.raw_url)
        .header("User-Agent", "Open-Switch/1.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            cleanup_on_failure(&skills_dir);
            return Err(AppError::Custom(format!("下载失败: {}", e)));
        }
    };
    
    if !response.status().is_success() {
        cleanup_on_failure(&skills_dir);
        return Ok(InstallSkillsResult {
            success: false,
            message: format!("下载失败: HTTP {}", response.status()),
            installed_path: None,
        });
    }
    
    let content = match response.text().await {
        Ok(c) => c,
        Err(e) => {
            cleanup_on_failure(&skills_dir);
            return Err(AppError::Custom(format!("读取内容失败: {}", e)));
        }
    };
    
    let skills_file = skills_dir.join("SKILL.md");
    if let Err(e) = std::fs::write(&skills_file, &content) {
        cleanup_on_failure(&skills_dir);
        return Err(AppError::Custom(format!("写入文件失败: {}", e)));
    }
    
    Ok(InstallSkillsResult {
        success: true,
        message: "安装成功".to_string(),
        installed_path: Some(skills_file.to_string_lossy().to_string()),
    })
}

/// 删除 Skills
#[tauri::command]
pub fn delete_skills(skills_path: String) -> Result<(), AppError> {
    let path = PathBuf::from(&skills_path);
    
    let skills_dir = path.parent()
        .ok_or_else(|| AppError::Custom("无效的路径".to_string()))?;
    
    std::fs::remove_dir_all(skills_dir)
        .map_err(|e| AppError::Custom(format!("删除失败: {}", e)))?;
    
    Ok(())
}

/// 读取 Skills 内容
#[tauri::command]
pub fn read_skills_content(skills_path: String) -> Result<String, AppError> {
    std::fs::read_to_string(&skills_path)
        .map_err(|e| AppError::Custom(format!("读取文件失败: {}", e)))
}
