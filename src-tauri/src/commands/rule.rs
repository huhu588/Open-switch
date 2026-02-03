// 规则管理命令模块
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 已安装的规则信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledRule {
    pub name: String,
    pub location: String,       // global_opencode, project_opencode, etc.
    pub path: String,           // 实际文件路径
    pub description: String,    // 从文件内容解析
    pub rule_type: String,      // agents_md, rules_dir, instructions
    pub enabled: bool,          // 是否启用
}

/// 推荐规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub content: String,        // 规则内容
    pub file_type: String,      // agents_md, rules_md
}

/// 获取目录路径
fn get_paths() -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();
    
    // 获取用户主目录
    if let Some(home) = dirs::home_dir() {
        // 全局 OpenCode 配置
        paths.insert("global_opencode".to_string(), home.join(".config").join("opencode"));
        // 全局 Claude 配置
        paths.insert("global_claude".to_string(), home.join(".claude"));
        // 全局 Cursor 配置
        paths.insert("global_cursor".to_string(), home.join(".cursor"));
    }
    
    // 项目级配置（当前目录）
    if let Ok(cwd) = std::env::current_dir() {
        paths.insert("project_opencode".to_string(), cwd.join(".opencode"));
        paths.insert("project_claude".to_string(), cwd.join(".claude"));
        paths.insert("project_root".to_string(), cwd.clone());
    }
    
    paths
}

/// 解析规则描述（从文件内容提取）
fn parse_rule_description(content: &str) -> String {
    // 尝试从 YAML frontmatter 提取 description
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..end + 3];
            for line in frontmatter.lines() {
                let line = line.trim();
                if line.starts_with("description:") {
                    return line[12..].trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
        }
    }
    
    // 从第一个非空行或 # 标题提取
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line == "---" {
            continue;
        }
        if line.starts_with('#') {
            return line.trim_start_matches('#').trim().to_string();
        }
        return line.chars().take(100).collect::<String>() + "...";
    }
    
    "无描述".to_string()
}

/// 获取已安装的规则列表
#[tauri::command]
pub fn get_installed_rules() -> Vec<InstalledRule> {
    let mut rules = Vec::new();
    let paths = get_paths();
    
    // 检查 AGENTS.md 文件（包括禁用的）
    let agents_locations = [
        ("global_opencode", "AGENTS.md"),
        ("project_root", "AGENTS.md"),
    ];
    
    for (location_key, filename) in agents_locations.iter() {
        if let Some(base_path) = paths.get(*location_key) {
            // 提取不带扩展名的名称（AGENTS.md -> AGENTS）
            let name = filename.trim_end_matches(".md").to_string();
            
            // 检查启用的文件
            let file_path = base_path.join(filename);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    rules.push(InstalledRule {
                        name: name.clone(),
                        location: location_key.to_string(),
                        path: file_path.to_string_lossy().to_string(),
                        description: parse_rule_description(&content),
                        rule_type: "agents_md".to_string(),
                        enabled: true,
                    });
                }
            }
            
            // 检查禁用的文件
            let disabled_path = base_path.join(format!("{}.disabled", filename));
            if disabled_path.exists() {
                if let Ok(content) = fs::read_to_string(&disabled_path) {
                    rules.push(InstalledRule {
                        name: name.clone(),
                        location: location_key.to_string(),
                        path: disabled_path.to_string_lossy().to_string(),
                        description: parse_rule_description(&content),
                        rule_type: "agents_md".to_string(),
                        enabled: false,
                    });
                }
            }
        }
    }
    
    // 检查 .opencode/rules/ 目录
    let rules_dirs = [
        ("global_opencode", "rules"),
        ("project_opencode", "rules"),
    ];
    
    for (location_key, subdir) in rules_dirs.iter() {
        if let Some(base_path) = paths.get(*location_key) {
            let rules_path = base_path.join(subdir);
            if rules_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&rules_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            
                            // 检查是否为禁用文件 (.md.disabled 或 .mdc.disabled)
                            let is_disabled = file_name.ends_with(".md.disabled") || file_name.ends_with(".mdc.disabled");
                            let is_enabled_md = (ext == "md" || ext == "mdc") && !file_name.ends_with(".disabled");
                            
                            if is_enabled_md || is_disabled {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    // 提取真实名称（去除 .disabled 后缀）
                                    let name = if is_disabled {
                                        // 从 xxx.md.disabled 提取 xxx
                                        let base_name = file_name.trim_end_matches(".disabled");
                                        base_name.trim_end_matches(".md").trim_end_matches(".mdc").to_string()
                                    } else {
                                        path.file_stem()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("unknown")
                                            .to_string()
                                    };
                                    
                                    rules.push(InstalledRule {
                                        name,
                                        location: location_key.to_string(),
                                        path: path.to_string_lossy().to_string(),
                                        description: parse_rule_description(&content),
                                        rule_type: "rules_dir".to_string(),
                                        enabled: !is_disabled,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 检查 Claude skills 目录中的规则
    let claude_skills = [
        ("global_claude", "skills"),
        ("project_claude", "skills"),
    ];
    
    for (location_key, subdir) in claude_skills.iter() {
        if let Some(base_path) = paths.get(*location_key) {
            let skills_path = base_path.join(subdir);
            if skills_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&skills_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            // 检查 SKILL.md 文件 - 这是 skills，不是规则，跳过
                            let skill_md = path.join("SKILL.md");
                            if skill_md.exists() {
                                continue; // 这是 skills，不是规则
                            }
                            
                            // 检查其他 .md 文件作为规则
                            if let Ok(sub_entries) = fs::read_dir(&path) {
                                for sub_entry in sub_entries.flatten() {
                                    let sub_path = sub_entry.path();
                                    if sub_path.is_file() {
                                        let file_name = sub_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                        let ext = sub_path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                        
                                        let is_disabled = file_name.ends_with(".md.disabled");
                                        let is_enabled_md = ext == "md" && !file_name.ends_with(".disabled");
                                        
                                        if is_enabled_md || is_disabled {
                                            if let Ok(content) = fs::read_to_string(&sub_path) {
                                                let name = if is_disabled {
                                                    file_name.trim_end_matches(".md.disabled").to_string()
                                                } else {
                                                    sub_path.file_stem()
                                                        .and_then(|n| n.to_str())
                                                        .unwrap_or("unknown")
                                                        .to_string()
                                                };
                                                
                                                rules.push(InstalledRule {
                                                    name,
                                                    location: location_key.to_string(),
                                                    path: sub_path.to_string_lossy().to_string(),
                                                    description: parse_rule_description(&content),
                                                    rule_type: "claude_skills".to_string(),
                                                    enabled: !is_disabled,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 检查 Cursor rules 目录
    if let Some(base_path) = paths.get("global_cursor") {
        let rules_path = base_path.join("rules");
        if rules_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&rules_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        
                        // 检查是否为禁用文件
                        let is_disabled = file_name.ends_with(".md.disabled") || file_name.ends_with(".mdc.disabled");
                        let is_enabled_md = (ext == "md" || ext == "mdc") && !file_name.ends_with(".disabled");
                        
                        if is_enabled_md || is_disabled {
                            if let Ok(content) = fs::read_to_string(&path) {
                                let name = if is_disabled {
                                    let base_name = file_name.trim_end_matches(".disabled");
                                    base_name.trim_end_matches(".md").trim_end_matches(".mdc").to_string()
                                } else {
                                    path.file_stem()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("unknown")
                                        .to_string()
                                };
                                
                                rules.push(InstalledRule {
                                    name,
                                    location: "global_cursor".to_string(),
                                    path: path.to_string_lossy().to_string(),
                                    description: parse_rule_description(&content),
                                    rule_type: "cursor_rules".to_string(),
                                    enabled: !is_disabled,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    
    rules
}

/// 获取推荐规则列表 - 来源: https://github.com/NeekChaw/RIPER-5
#[tauri::command]
pub fn get_recommended_rules() -> Vec<RecommendedRule> {
    vec![
        // --- RIPER-5 核心规则 ---
        RecommendedRule {
            id: "riper-5-cn".to_string(),
            name: "RIPER-5 严格协议 (中文)".to_string(),
            description: "AI 严格行为协议和工作流框架，防止 AI 随意修改代码".to_string(),
            category: "workflow".to_string(),
            file_type: "agents_md".to_string(),
            content: include_str!("../assets/rules/riper-5-cn.md").to_string(),
        },
        
        // --- Claude Code 工作流 ---
        RecommendedRule {
            id: "deep-thinking".to_string(),
            name: "超深度思考".to_string(),
            description: "让 AI 进行深度思考，提供更全面的分析".to_string(),
            category: "thinking".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# 超深度思考模式

## 核心原则
在回答任何问题之前，先进行深入思考：

### 系统思维
- 从整体架构到具体实现进行分析
- 识别组件间的依赖关系和相互影响
- 考虑解决方案对整个系统的长远影响

### 辩证思维
- 评估多种解决方案的优缺点
- 寻找看似对立观点间的平衡点
- 从不同角度审视问题的本质

### 创新思维
- 打破常规模式，寻求创新解决方案
- 探索非传统的问题解决路径
- 结合不同领域的知识和方法

### 批判性思维
- 从多个角度验证和优化解决方案
- 识别潜在的问题和风险
- 确保逻辑的严密性和结论的可靠性

## 思维平衡
- 分析 ↔ 直觉
- 细节检查 ↔ 全局视角
- 理论理解 ↔ 实际应用
- 深度思考 ↔ 执行效率
- 复杂性 ↔ 清晰度
"#.to_string(),
        },
        RecommendedRule {
            id: "linus-code-review".to_string(),
            name: "Linus 代码审查".to_string(),
            description: "模拟 Linus Torvalds 的思维模式，以犯利、深刻的视角审查代码".to_string(),
            category: "review".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# Linus Torvalds 代码审查模式

你现在是 Linus Torvalds，以严谨、直接、不容忍低质量代码的风格著称。

## 审查原则

### 代码简洁性
- 代码应该尽可能简单
- 复杂性是数据，不是代码
- 每个函数只做一件事

### 性能意识
- 思考每一行代码的性能影响
- 避免不必要的内存分配
- 考虑缓存和 CPU 使用

### 错误处理
- 错误处理不是事后考虑
- 每个可能失败的操作都必须检查
- 失败路径应该和成功路径一样清晰

### 命名
- 名称应该清晰地表明用途
- 不要缩写，除非它是广泛使用的
- 不要过度工程化命名

## 审查风格
- 直接指出问题，不要婉转
- 提供具体的改进建议
- 解释为什么某个方法更好
- 对优雅的代码给予认可
"#.to_string(),
        },
        RecommendedRule {
            id: "ai-dev-workflow".to_string(),
            name: "AI 协同开发规范".to_string(),
            description: "三阶段工作流，将模糊想法转化为完整开发文档".to_string(),
            category: "workflow".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# AI 协同开发规范

## 三阶段工作流

### 第一阶段：需求分析
1. 理解用户的真实需求
2. 识别关键约束和要求
3. 定义清晰的成功标准

### 第二阶段：方案设计
1. 探索多种可能的解决方案
2. 评估每种方案的优劣
3. 选择最佳方案并详细规划

### 第三阶段：实施执行
1. 按计划逐步实现
2. 每步验证结果
3. 记录变更和进度

## 工作原则
- 不假设，只确认
- 先计划，后执行
- 小步快跑，频繁验证
- 保持与原始需求的联系
"#.to_string(),
        },
        RecommendedRule {
            id: "git-commit-pro".to_string(),
            name: "专业 Git 提交".to_string(),
            description: "规范化的 Git 提交信息和分支管理".to_string(),
            category: "workflow".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# 专业 Git 提交规范

## 提交信息格式
```
<type>(<scope>): <subject>

<body>

<footer>
```

## Type 类型
- `feat`: 新功能
- `fix`: 修复 Bug
- `docs`: 文档更新
- `style`: 代码格式（不影响代码逻辑）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建流程/工具

## Subject 规则
- 使用祈使句
- 首字母小写
- 不加句号
- 不超过 50 字符

## 分支命名
- `feature/xxx`: 新功能
- `fix/xxx`: 修复
- `hotfix/xxx`: 紧急修复
- `release/x.x.x`: 发布

## 提交原则
- 原子化提交：每次提交只做一件事
- 有意义的提交信息
- 关联 Issue 或任务编号
"#.to_string(),
        },
        RecommendedRule {
            id: "kiro-requirements".to_string(),
            name: "Kiro 需求收集".to_string(),
            description: "系统化的需求收集和分析流程".to_string(),
            category: "requirements".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# Kiro 需求收集流程

## 需求收集步骤

### 1. 背景理解
- 项目的整体目标是什么？
- 目标用户是谁？
- 现有系统是什么状态？

### 2. 功能需求
- 用户需要完成什么任务？
- 每个任务的输入和输出是什么？
- 有哪些业务规则需要遵守？

### 3. 非功能需求
- 性能要求（响应时间、并发数）
- 安全要求
- 兼容性要求
- 可维护性要求

### 4. 约束条件
- 技术约束（现有技术栈、集成要求）
- 时间约束
- 资源约束

## 需求文档模板
```markdown
# 需求文档

## 项目背景
[..]

## 功能需求
[..]

## 非功能需求
[..]

## 约束和依赖
[..]

## 成功标准
[..]
```
"#.to_string(),
        },
        
        // --- 其他实用规则 ---
        RecommendedRule {
            id: "code-quality".to_string(),
            name: "代码质量标准".to_string(),
            description: "代码质量和可维护性标准".to_string(),
            category: "quality".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# 代码质量标准

## 代码完整性
- 始终显示完整代码上下文
- 在代码块中指定语言和路径
- 适当的错误处理
- 标准化命名约定
- 清晰简洁的注释

## 编辑指南
- 只显示必要的修改
- 包括文件路径和语言标识符
- 提供上下文注释
- 考虑对代码库的影响
- 验证与请求的相关性
- 保持范围合规性
- 避免不必要的更改

## 禁止行为
- 使用未经验证的依赖项
- 留下不完整的功能
- 包含未测试的代码
- 使用过时的解决方案
- 跳过或缩略代码部分
- 修改不相关的代码
- 使用代码占位符
"#.to_string(),
        },
        RecommendedRule {
            id: "security-rules".to_string(),
            name: "安全编码规范".to_string(),
            description: "Web 应用安全最佳实践".to_string(),
            category: "security".to_string(),
            file_type: "agents_md".to_string(),
            content: r#"# 安全编码规范

## 输入验证
- 永远不要信任用户输入
- 使用白名单验证
- 对特殊字符进行转义
- 限制输入长度

## 认证与授权
- 使用安全的密码哈希
- 实施会话管理
- 使用 HTTPS
- 实施 CORS 策略

## 敏感数据
- 不在代码中硬编码密钥
- 使用环境变量存储配置
- 加密存储敏感数据
- 日志中脱敏敏感信息

## 常见漏洞防护
- XSS: 转义输出
- CSRF: 使用 token
- SQL 注入: 使用参数化查询
- 路径遍历: 验证文件路径
"#.to_string(),
        },
    ]
}

/// 安装规则
#[tauri::command]
pub fn install_rule(rule_id: String, content: String, location: String) -> Result<String, String> {
    let paths = get_paths();
    
    // 获取推荐规则信息
    let recommended = get_recommended_rules();
    let rule = recommended.iter().find(|r| r.id == rule_id);
    
    let (file_name, file_content) = if let Some(r) = rule {
        let name = if r.file_type == "agents_md" {
            "AGENTS.md".to_string()
        } else {
            format!("{}.md", r.id)
        };
        (name, r.content.clone())
    } else {
        // 自定义规则
        (format!("{}.md", rule_id), content)
    };
    
    // 确定安装路径
    let target_path = match location.as_str() {
        "global_opencode" => {
            let base = paths.get("global_opencode")
                .ok_or("无法获取全局 OpenCode 配置路径")?;
            if file_name == "AGENTS.md" {
                base.join(&file_name)
            } else {
                base.join("rules").join(&file_name)
            }
        }
        "project_opencode" => {
            let base = paths.get("project_opencode")
                .ok_or("无法获取项目 OpenCode 配置路径")?;
            if file_name == "AGENTS.md" {
                paths.get("project_root")
                    .ok_or("无法获取项目根目录")?
                    .join(&file_name)
            } else {
                base.join("rules").join(&file_name)
            }
        }
        "global_claude" => {
            let base = paths.get("global_claude")
                .ok_or("无法获取全局 Claude 配置路径")?;
            base.join("rules").join(&file_name)
        }
        "project_claude" => {
            let base = paths.get("project_claude")
                .ok_or("无法获取项目 Claude 配置路径")?;
            base.join("rules").join(&file_name)
        }
        "global_cursor" => {
            let base = paths.get("global_cursor")
                .ok_or("无法获取全局 Cursor 配置路径")?;
            base.join("rules").join(&file_name)
        }
        _ => return Err(format!("不支持的安装位置: {}", location)),
    };
    
    // 创建目录
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    // 写入文件
    fs::write(&target_path, &file_content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(target_path.to_string_lossy().to_string())
}

/// 删除规则
#[tauri::command]
pub fn delete_rule(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err("规则文件不存在".to_string());
    }
    
    fs::remove_file(&path)
        .map_err(|e| format!("删除规则失败: {}", e))?;
    
    Ok(())
}

/// 读取规则内容
#[tauri::command]
pub fn read_rule_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path)
        .map_err(|e| format!("读取规则内容失败: {}", e))
}

/// 保存规则内容
#[tauri::command]
pub fn save_rule_content(path: String, content: String) -> Result<(), String> {
    fs::write(&path, &content)
        .map_err(|e| format!("保存规则失败: {}", e))
}

/// 切换规则启用状态
#[tauri::command]
pub fn toggle_rule_enabled(path: String, enabled: bool) -> Result<String, String> {
    let current_path = PathBuf::from(&path);
    
    if !current_path.exists() {
        return Err("规则文件不存在".to_string());
    }
    
    let file_name = current_path.file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取文件名")?;
    
    let parent = current_path.parent()
        .ok_or("无法获取父目录")?;
    
    let new_path = if enabled {
        // 启用：移除 .disabled 后缀
        if file_name.ends_with(".disabled") {
            parent.join(file_name.trim_end_matches(".disabled"))
        } else {
            return Ok(path); // 已经启用
        }
    } else {
        // 禁用：添加 .disabled 后缀
        if !file_name.ends_with(".disabled") {
            parent.join(format!("{}.disabled", file_name))
        } else {
            return Ok(path); // 已经禁用
        }
    };
    
    fs::rename(&current_path, &new_path)
        .map_err(|e| format!("切换规则状态失败: {}", e))?;
    
    Ok(new_path.to_string_lossy().to_string())
}

// ============================================================================
// 规则多应用统一管理
// ============================================================================

/// 聚合的规则管理信息
#[derive(Debug, Clone, Serialize)]
pub struct ManagedRule {
    pub name: String,
    pub description: String,
    pub content: String,
    pub rule_type: String,
    // 各应用部署状态
    pub opencode_enabled: bool,
    pub claude_enabled: bool,
    pub codex_enabled: bool,
    pub gemini_enabled: bool,
    pub cursor_enabled: bool,
    // 源路径（用于读取内容）
    pub source_path: Option<String>,
}

/// 规则统计信息
#[derive(Debug, Clone, Serialize, Default)]
pub struct RuleStats {
    pub opencode_count: usize,
    pub claude_count: usize,
    pub codex_count: usize,
    pub gemini_count: usize,
    pub cursor_count: usize,
}

/// 获取各应用的规则目录
fn get_app_rules_paths() -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();
    
    if let Some(home) = dirs::home_dir() {
        // OpenCode
        paths.insert("opencode".to_string(), home.join(".config").join("opencode").join("rules"));
        // Claude Code
        paths.insert("claude".to_string(), home.join(".claude").join("rules"));
        // Codex
        paths.insert("codex".to_string(), home.join(".codex").join("rules"));
        // Gemini
        paths.insert("gemini".to_string(), home.join(".gemini").join("rules"));
        // Cursor
        paths.insert("cursor".to_string(), home.join(".cursor").join("rules"));
    }
    
    paths
}

/// 扫描目录中的规则文件
fn scan_rules_in_dir(dir: &PathBuf) -> Vec<String> {
    let mut rules = Vec::new();
    
    if !dir.is_dir() {
        return rules;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                
                // 只统计启用的规则
                if (ext == "md" || ext == "mdc") && !file_name.ends_with(".disabled") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        rules.push(stem.to_string());
                    }
                }
            }
        }
    }
    
    rules
}

/// 获取所有管理的规则（聚合各应用的状态）
#[tauri::command]
pub fn get_managed_rules() -> Vec<ManagedRule> {
    use std::collections::HashSet;
    
    let app_paths = get_app_rules_paths();
    let mut all_rule_names: HashSet<String> = HashSet::new();
    let mut managed_rules: HashMap<String, ManagedRule> = HashMap::new();
    
    // 收集所有规则名称和内容
    for (app, dir) in &app_paths {
        if !dir.is_dir() {
            continue;
        }
        
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    
                    if (ext == "md" || ext == "mdc") && !file_name.ends_with(".disabled") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let name = stem.to_string();
                            all_rule_names.insert(name.clone());
                            
                            // 如果还没有这个规则的记录，创建一个
                            if !managed_rules.contains_key(&name) {
                                let content = fs::read_to_string(&path).unwrap_or_default();
                                let description = parse_rule_description(&content);
                                
                                managed_rules.insert(name.clone(), ManagedRule {
                                    name: name.clone(),
                                    description,
                                    content: content.clone(),
                                    rule_type: "rules_dir".to_string(),
                                    opencode_enabled: false,
                                    claude_enabled: false,
                                    codex_enabled: false,
                                    gemini_enabled: false,
                                    cursor_enabled: false,
                                    source_path: Some(path.to_string_lossy().to_string()),
                                });
                            }
                            
                            // 标记在此应用启用
                            if let Some(rule) = managed_rules.get_mut(&name) {
                                match app.as_str() {
                                    "opencode" => rule.opencode_enabled = true,
                                    "claude" => rule.claude_enabled = true,
                                    "codex" => rule.codex_enabled = true,
                                    "gemini" => rule.gemini_enabled = true,
                                    "cursor" => rule.cursor_enabled = true,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    let mut result: Vec<ManagedRule> = managed_rules.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    
    result
}

/// 获取规则统计信息
#[tauri::command]
pub fn get_rule_stats() -> RuleStats {
    let app_paths = get_app_rules_paths();
    let mut stats = RuleStats::default();
    
    for (app, dir) in &app_paths {
        let count = scan_rules_in_dir(dir).len();
        match app.as_str() {
            "opencode" => stats.opencode_count = count,
            "claude" => stats.claude_count = count,
            "codex" => stats.codex_count = count,
            "gemini" => stats.gemini_count = count,
            "cursor" => stats.cursor_count = count,
            _ => {}
        }
    }
    
    stats
}

/// 切换规则在某个应用上的启用状态
#[tauri::command]
pub fn toggle_rule_app(rule_name: String, app: String, enabled: bool, content: String) -> Result<(), String> {
    let app_paths = get_app_rules_paths();
    
    let target_dir = app_paths.get(&app)
        .ok_or_else(|| format!("不支持的应用: {}", app))?;
    
    let target_path = target_dir.join(format!("{}.md", rule_name));
    
    if enabled {
        // 创建目录
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
        // 写入规则文件
        fs::write(&target_path, &content)
            .map_err(|e| format!("写入规则失败: {}", e))?;
    } else {
        // 删除规则文件
        if target_path.exists() {
            fs::remove_file(&target_path)
                .map_err(|e| format!("删除规则失败: {}", e))?;
        }
    }
    
    Ok(())
}

/// 从所有应用中删除规则
#[tauri::command]
pub fn delete_rule_from_all(rule_name: String) -> Result<(), String> {
    let app_paths = get_app_rules_paths();
    
    for (_app, dir) in &app_paths {
        let target_path = dir.join(format!("{}.md", rule_name));
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }
        // 也删除禁用的版本
        let disabled_path = dir.join(format!("{}.md.disabled", rule_name));
        if disabled_path.exists() {
            let _ = fs::remove_file(&disabled_path);
        }
    }
    
    Ok(())
}
