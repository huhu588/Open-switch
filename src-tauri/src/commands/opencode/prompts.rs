// Prompts 统一管理命令
// 管理各 CLI 工具的系统提示文件：
// - CLAUDE.md (Claude Code)
// - AGENTS.md (Codex)
// - GEMINI.md (Gemini CLI)

use crate::modules::opencode_config::claude_code_manager::ClaudeCodeConfigManager;
use crate::modules::opencode_config::codex_manager::CodexConfigManager;
use crate::modules::opencode_config::gemini_manager::GeminiConfigManager;
use serde::{Deserialize, Serialize};

/// Prompt 类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromptType {
    Claude,
    Codex,
    Gemini,
}

/// Prompt 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub prompt_type: PromptType,
    pub name: String,
    pub file_name: String,
    pub exists: bool,
    pub content: Option<String>,
    pub char_count: usize,
}

/// 所有 Prompts 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsStatus {
    pub claude: PromptInfo,
    pub codex: PromptInfo,
    pub gemini: PromptInfo,
}

/// 获取所有 Prompts 状态
#[tauri::command]
pub async fn get_prompts_status() -> Result<PromptsStatus, String> {
    // Claude CLAUDE.md
    let claude_manager = ClaudeCodeConfigManager::new()?;
    let claude_content = claude_manager.read_claude_md()?;
    let claude_exists = claude_content.is_some();
    let claude_char_count = claude_content.as_ref().map(|c| c.len()).unwrap_or(0);
    
    // Codex AGENTS.md
    let codex_manager = CodexConfigManager::new()?;
    let codex_content = codex_manager.read_agents_md()?;
    let codex_exists = codex_content.is_some();
    let codex_char_count = codex_content.as_ref().map(|c| c.len()).unwrap_or(0);
    
    // Gemini GEMINI.md
    let gemini_manager = GeminiConfigManager::new()?;
    let gemini_content = gemini_manager.read_gemini_md()?;
    let gemini_exists = gemini_content.is_some();
    let gemini_char_count = gemini_content.as_ref().map(|c| c.len()).unwrap_or(0);
    
    Ok(PromptsStatus {
        claude: PromptInfo {
            prompt_type: PromptType::Claude,
            name: "Claude Code".to_string(),
            file_name: "CLAUDE.md".to_string(),
            exists: claude_exists,
            content: claude_content,
            char_count: claude_char_count,
        },
        codex: PromptInfo {
            prompt_type: PromptType::Codex,
            name: "Codex".to_string(),
            file_name: "AGENTS.md".to_string(),
            exists: codex_exists,
            content: codex_content,
            char_count: codex_char_count,
        },
        gemini: PromptInfo {
            prompt_type: PromptType::Gemini,
            name: "Gemini CLI".to_string(),
            file_name: "GEMINI.md".to_string(),
            exists: gemini_exists,
            content: gemini_content,
            char_count: gemini_char_count,
        },
    })
}

/// 获取单个 Prompt 内容
#[tauri::command]
pub async fn get_prompt(prompt_type: PromptType) -> Result<Option<String>, String> {
    match prompt_type {
        PromptType::Claude => {
            let manager = ClaudeCodeConfigManager::new()?;
            manager.read_claude_md()
        }
        PromptType::Codex => {
            let manager = CodexConfigManager::new()?;
            manager.read_agents_md()
        }
        PromptType::Gemini => {
            let manager = GeminiConfigManager::new()?;
            manager.read_gemini_md()
        }
    }
}

/// 保存单个 Prompt 内容
#[tauri::command]
pub async fn save_prompt(prompt_type: PromptType, content: String) -> Result<(), String> {
    match prompt_type {
        PromptType::Claude => {
            let manager = ClaudeCodeConfigManager::new()?;
            manager.write_claude_md(&content)
        }
        PromptType::Codex => {
            let manager = CodexConfigManager::new()?;
            manager.write_agents_md(&content)
        }
        PromptType::Gemini => {
            let manager = GeminiConfigManager::new()?;
            manager.write_gemini_md(&content)
        }
    }
}

/// 同步 Prompt 到多个目标
#[tauri::command]
pub async fn sync_prompt(
    content: String,
    targets: Vec<PromptType>,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    
    for target in targets {
        let result = save_prompt(target.clone(), content.clone()).await;
        match result {
            Ok(_) => {
                let name = match target {
                    PromptType::Claude => "Claude Code",
                    PromptType::Codex => "Codex",
                    PromptType::Gemini => "Gemini CLI",
                };
                results.push(format!("{}: 成功", name));
            }
            Err(e) => {
                let name = match target {
                    PromptType::Claude => "Claude Code",
                    PromptType::Codex => "Codex",
                    PromptType::Gemini => "Gemini CLI",
                };
                results.push(format!("{}: 失败 - {}", name, e));
            }
        }
    }
    
    Ok(results)
}

/// 删除 Prompt 文件
#[tauri::command]
pub async fn delete_prompt(prompt_type: PromptType) -> Result<(), String> {
    let path = match prompt_type {
        PromptType::Claude => {
            let home = dirs::home_dir()
                .ok_or_else(|| "无法获取用户主目录".to_string())?;
            home.join(".claude").join("CLAUDE.md")
        }
        PromptType::Codex => {
            let home = dirs::home_dir()
                .ok_or_else(|| "无法获取用户主目录".to_string())?;
            home.join(".codex").join("AGENTS.md")
        }
        PromptType::Gemini => {
            let home = dirs::home_dir()
                .ok_or_else(|| "无法获取用户主目录".to_string())?;
            home.join(".gemini").join("GEMINI.md")
        }
    };
    
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("删除文件失败: {}", e))?;
    }
    
    Ok(())
}

/// Prompt 预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

/// 获取推荐的 Prompt 预设
#[tauri::command]
pub async fn get_prompt_presets() -> Result<Vec<PromptPreset>, String> {
    Ok(vec![
        PromptPreset {
            id: "coding-assistant".to_string(),
            name: "编程助手".to_string(),
            description: "通用编程助手，专注于代码质量和最佳实践".to_string(),
            content: r#"# 编程助手

## 角色定义
你是一个专业的编程助手，专注于帮助开发者编写高质量的代码。

## 核心原则
1. 代码质量优先：始终追求清晰、可维护、高效的代码
2. 最佳实践：遵循行业标准和语言特定的最佳实践
3. 安全意识：在编写代码时始终考虑安全性
4. 文档完善：为重要的代码添加适当的注释和文档

## 响应风格
- 简洁明了，避免冗余
- 提供代码示例时，确保可以直接运行
- 解释关键决策的原因
- 在适当时提供替代方案
"#.to_string(),
        },
        PromptPreset {
            id: "code-reviewer".to_string(),
            name: "代码审查".to_string(),
            description: "专注于代码审查和改进建议".to_string(),
            content: r#"# 代码审查助手

## 角色定义
你是一个资深的代码审查专家，专注于发现代码问题并提供改进建议。

## 审查重点
1. 代码逻辑：检查逻辑错误和边界条件
2. 性能问题：识别可能的性能瓶颈
3. 安全漏洞：检测常见的安全问题
4. 代码风格：确保代码符合项目规范
5. 可维护性：评估代码的可读性和可维护性

## 反馈格式
- 使用清晰的标记区分问题严重程度（错误/警告/建议）
- 为每个问题提供具体的修复建议
- 解释问题可能导致的后果
"#.to_string(),
        },
        PromptPreset {
            id: "minimal".to_string(),
            name: "极简模式".to_string(),
            description: "最简化的提示，让 AI 自由发挥".to_string(),
            content: r#"# 指导原则

- 简洁回复
- 直接给出代码
- 最小化解释
"#.to_string(),
        },
    ])
}
