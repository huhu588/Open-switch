// Skill 管理相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::AppError;

/// 已安装的 Skill 信息
#[derive(Debug, Clone, Serialize)]
pub struct InstalledSkill {
    pub name: String,
    pub path: String,
    pub location: SkillLocation,
    pub content_preview: String,
}

/// Skill 安装位置类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillLocation {
    /// 项目 OpenCode: .opencode/skill/<name>/SKILL.md
    ProjectOpenCode,
    /// 全局 OpenCode: ~/.config/opencode/skill/<name>/SKILL.md
    GlobalOpenCode,
    /// 项目 Claude: .claude/skills/<name>/SKILL.md
    ProjectClaude,
    /// 全局 Claude: ~/.claude/skills/<name>/SKILL.md
    GlobalClaude,
}

impl std::fmt::Display for SkillLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLocation::ProjectOpenCode => write!(f, "项目 OpenCode"),
            SkillLocation::GlobalOpenCode => write!(f, "全局 OpenCode"),
            SkillLocation::ProjectClaude => write!(f, "项目 Claude"),
            SkillLocation::GlobalClaude => write!(f, "全局 Claude"),
        }
    }
}

/// 推荐的 Skill 信息
#[derive(Debug, Clone, Serialize)]
pub struct RecommendedSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub repo: String,
    pub repo_url: String,
    pub raw_url: String,
}

/// 安装 Skill 的输入参数
#[derive(Debug, Deserialize)]
pub struct InstallSkillInput {
    pub skill_id: String,
    pub raw_url: String,
    pub location: String, // "global_opencode" | "project_opencode" | "global_claude" | "project_claude"
}

/// 安装结果
#[derive(Debug, Serialize)]
pub struct InstallSkillResult {
    pub success: bool,
    pub message: String,
    pub installed_path: Option<String>,
}

/// 获取所有 Skill 扫描路径
fn get_skill_paths() -> Vec<(PathBuf, SkillLocation)> {
    let mut paths = Vec::new();
    
    // 全局 OpenCode: ~/.config/opencode/skill/
    if let Some(config_dir) = dirs::config_dir() {
        paths.push((
            config_dir.join("opencode").join("skill"),
            SkillLocation::GlobalOpenCode,
        ));
    }
    
    // 全局 Claude: ~/.claude/skills/
    if let Some(home_dir) = dirs::home_dir() {
        paths.push((
            home_dir.join(".claude").join("skills"),
            SkillLocation::GlobalClaude,
        ));
    }
    
    // 项目路径（当前工作目录）
    if let Ok(cwd) = std::env::current_dir() {
        // 项目 OpenCode: .opencode/skill/
        paths.push((
            cwd.join(".opencode").join("skill"),
            SkillLocation::ProjectOpenCode,
        ));
        
        // 项目 Claude: .claude/skills/
        paths.push((
            cwd.join(".claude").join("skills"),
            SkillLocation::ProjectClaude,
        ));
    }
    
    paths
}

/// 扫描已安装的 Skills
#[tauri::command]
pub fn get_installed_skills() -> Result<Vec<InstalledSkill>, AppError> {
    let mut skills = Vec::new();
    
    for (base_path, location) in get_skill_paths() {
        if !base_path.exists() {
            continue;
        }
        
        // 遍历目录下的所有子目录
        if let Ok(entries) = std::fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_file = path.join("SKILL.md");
                    if skill_file.exists() {
                        let name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        // 读取内容预览（前200字符）
                        let content_preview = std::fs::read_to_string(&skill_file)
                            .map(|c| {
                                let preview: String = c.chars().take(200).collect();
                                if c.len() > 200 {
                                    format!("{}...", preview)
                                } else {
                                    preview
                                }
                            })
                            .unwrap_or_default();
                        
                        skills.push(InstalledSkill {
                            name,
                            path: skill_file.to_string_lossy().to_string(),
                            location: location.clone(),
                            content_preview,
                        });
                    }
                }
            }
        }
    }
    
    // 按名称排序
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(skills)
}

/// 获取推荐的 Skills 列表
#[tauri::command]
pub fn get_recommended_skills() -> Vec<RecommendedSkill> {
    vec![
        // ============ Anthropic 官方 Skills (anthropics/skills) ============
        // 路径格式: skills/<name>/SKILL.md
        
        // --- 开发工具 ---
        RecommendedSkill {
            id: "skill-creator".to_string(),
            name: "Skill Creator".to_string(),
            description: "Skill 创建器 - 交互式工具，指导您通过问答创建新 Skill".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/skill-creator/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "mcp-builder".to_string(),
            name: "MCP Builder".to_string(),
            description: "MCP 服务器构建器 - 帮助创建 MCP 服务器".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/mcp-builder/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "frontend-design".to_string(),
            name: "Frontend Design".to_string(),
            description: "前端设计 - 创建精美、生产级的 Web 界面".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/frontend-design/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "web-artifacts-builder".to_string(),
            name: "Web Artifacts Builder".to_string(),
            description: "Web 工件构建器 - 创建交互式 Web 工件".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/web-artifacts-builder/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "theme-factory".to_string(),
            name: "Theme Factory".to_string(),
            description: "主题工厂 - 创建和定制应用主题".to_string(),
            category: "development".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/theme-factory/SKILL.md".to_string(),
        },
        
        // --- 文档处理 ---
        RecommendedSkill {
            id: "pdf".to_string(),
            name: "PDF".to_string(),
            description: "PDF 处理 - 提取文本、合并拆分、表单填写等".to_string(),
            category: "document".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/pdf/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "docx".to_string(),
            name: "DOCX".to_string(),
            description: "Word 文档 - 创建、编辑、追踪修订、注释等".to_string(),
            category: "document".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/docx/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "pptx".to_string(),
            name: "PPTX".to_string(),
            description: "PowerPoint - 创建、编辑演示文稿，支持布局和图表".to_string(),
            category: "document".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/pptx/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "xlsx".to_string(),
            name: "XLSX".to_string(),
            description: "Excel 表格 - 创建、编辑电子表格和数据分析".to_string(),
            category: "document".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/xlsx/SKILL.md".to_string(),
        },
        
        // --- 企业工作流 ---
        RecommendedSkill {
            id: "brand-guidelines".to_string(),
            name: "Brand Guidelines".to_string(),
            description: "品牌指南 - 应用品牌颜色和排版到工件".to_string(),
            category: "enterprise".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/brand-guidelines/SKILL.md".to_string(),
        },
        RecommendedSkill {
            id: "internal-comms".to_string(),
            name: "Internal Comms".to_string(),
            description: "内部沟通 - 撰写状态报告、通讯、FAQ 等".to_string(),
            category: "enterprise".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/internal-comms/SKILL.md".to_string(),
        },
        
        // --- 创意工具 ---
        RecommendedSkill {
            id: "algorithmic-art".to_string(),
            name: "Algorithmic Art".to_string(),
            description: "算法艺术 - 生成算法艺术和可视化作品".to_string(),
            category: "creative".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/skills/algorithmic-art/SKILL.md".to_string(),
        },
        
        // --- 模板 ---
        RecommendedSkill {
            id: "template-skill".to_string(),
            name: "Template Skill".to_string(),
            description: "Skill 模板 - 创建自定义 Skill 的起点".to_string(),
            category: "template".to_string(),
            repo: "anthropics/skills".to_string(),
            repo_url: "https://github.com/anthropics/skills".to_string(),
            raw_url: "https://raw.githubusercontent.com/anthropics/skills/main/template-skill/SKILL.md".to_string(),
        },
    ]
}

/// 安装 Skill
#[tauri::command]
pub async fn install_skill(input: InstallSkillInput) -> Result<InstallSkillResult, AppError> {
    // 确定安装路径
    let base_path = match input.location.as_str() {
        "global_opencode" => {
            dirs::config_dir()
                .ok_or_else(|| AppError::Custom("无法获取配置目录".to_string()))?
                .join("opencode")
                .join("skill")
        }
        "project_opencode" => {
            std::env::current_dir()
                .map_err(|e| AppError::Custom(format!("无法获取当前目录: {}", e)))?
                .join(".opencode")
                .join("skill")
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
    
    // 创建 skill 目录
    let skill_dir = base_path.join(&input.skill_id);
    std::fs::create_dir_all(&skill_dir)
        .map_err(|e| AppError::Custom(format!("创建目录失败: {}", e)))?;
    
    // 下载 SKILL.md 文件
    let client = reqwest::Client::new();
    let response = client.get(&input.raw_url)
        .header("User-Agent", "Open-Switch/1.0")
        .send()
        .await
        .map_err(|e| AppError::Custom(format!("下载失败: {}", e)))?;
    
    if !response.status().is_success() {
        return Ok(InstallSkillResult {
            success: false,
            message: format!("下载失败: HTTP {}", response.status()),
            installed_path: None,
        });
    }
    
    let content = response.text()
        .await
        .map_err(|e| AppError::Custom(format!("读取内容失败: {}", e)))?;
    
    // 写入文件
    let skill_file = skill_dir.join("SKILL.md");
    std::fs::write(&skill_file, content)
        .map_err(|e| AppError::Custom(format!("写入文件失败: {}", e)))?;
    
    Ok(InstallSkillResult {
        success: true,
        message: "安装成功".to_string(),
        installed_path: Some(skill_file.to_string_lossy().to_string()),
    })
}

/// 删除 Skill
#[tauri::command]
pub fn delete_skill(skill_path: String) -> Result<(), AppError> {
    let path = PathBuf::from(&skill_path);
    
    // 获取 skill 目录（SKILL.md 的父目录）
    let skill_dir = path.parent()
        .ok_or_else(|| AppError::Custom("无效的路径".to_string()))?;
    
    // 删除整个目录
    std::fs::remove_dir_all(skill_dir)
        .map_err(|e| AppError::Custom(format!("删除失败: {}", e)))?;
    
    Ok(())
}

/// 读取 Skill 内容
#[tauri::command]
pub fn read_skill_content(skill_path: String) -> Result<String, AppError> {
    std::fs::read_to_string(&skill_path)
        .map_err(|e| AppError::Custom(format!("读取文件失败: {}", e)))
}
