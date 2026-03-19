// Augment Code 配置管理器
// 管理 Augment Code 的配置：
// - .augment/rules/ (项目级规则目录)
// - VS Code 扩展安装检测

use std::fs;
use std::path::PathBuf;

pub struct AugmentConfigManager {
    /// 项目级 rules 目录 (.augment/rules/)
    rules_dir: PathBuf,
    /// VS Code 扩展目录
    vscode_extensions_dir: PathBuf,
}

impl AugmentConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        // 项目级目录
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| user_home.clone());
        let rules_dir = cwd.join(".augment").join("rules");

        // VS Code 扩展目录
        let vscode_extensions_dir = user_home.join(".vscode").join("extensions");

        Ok(Self {
            rules_dir,
            vscode_extensions_dir,
        })
    }

    /// 确保 rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Augment rules 目录失败: {}", e))?;
        }
        Ok(())
    }

    // ==================== 路径获取 ====================

    /// 获取 rules 目录路径
    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    // ==================== 状态检测 ====================

    /// 检查 Augment Code 扩展是否已安装
    /// 扫描 ~/.vscode/extensions/ 目录查找 augmentcode-* 前缀的目录
    pub fn is_installed(&self) -> bool {
        if !self.vscode_extensions_dir.exists() {
            return false;
        }

        if let Ok(entries) = fs::read_dir(&self.vscode_extensions_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("augmentcode-") || name.starts_with("augment-") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取 Augment Code 扩展版本（如有）
    pub fn get_extension_version(&self) -> Option<String> {
        if !self.vscode_extensions_dir.exists() {
            return None;
        }

        if let Ok(entries) = fs::read_dir(&self.vscode_extensions_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("augmentcode-") || name.starts_with("augment-") {
                        // 从目录名提取版本号，格式通常为 augmentcode-augment-0.x.x
                        if let Some(version_part) = name.rsplit('-').next() {
                            return Some(version_part.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// 获取 rules 目录中的规则数量
    pub fn get_rules_count(&self) -> usize {
        if !self.rules_dir.is_dir() {
            return 0;
        }

        fs::read_dir(&self.rules_dir)
            .map(|entries| {
                entries.flatten().filter(|entry| {
                    let path = entry.path();
                    path.is_file() && path.extension().map_or(false, |ext| ext == "md")
                }).count()
            })
            .unwrap_or(0)
    }
}
