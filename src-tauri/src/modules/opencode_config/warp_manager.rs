// Warp 配置管理器
// 管理 Warp 的本地配置：
// - 安装检测（按平台区分路径）
// - warp.sqlite 本地数据库用量读取
// - Rules 目录管理（~/.warp/ 或 %APPDATA%\warp\）

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Warp 对话用量统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WarpUsageStats {
    /// 总积分消耗
    pub total_credits_spent: f64,
    /// 按模型分组的 token 用量
    pub token_usage_by_model: HashMap<String, WarpTokenUsage>,
    /// 工具使用统计
    pub tool_usage: WarpToolUsage,
    /// 代码变更统计
    pub code_changes: WarpCodeChanges,
    /// 对话总数
    pub conversation_count: usize,
}

/// 单个模型的 token 用量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WarpTokenUsage {
    pub total_tokens: u64,
    pub byok_tokens: u64,
}

/// 工具使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WarpToolUsage {
    pub run_command_count: u64,
    pub read_files_count: u64,
    pub search_codebase_count: u64,
    pub grep_count: u64,
    pub file_glob_count: u64,
    pub call_mcp_tool_count: u64,
    pub apply_file_diff_count: u64,
}

/// 代码变更统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WarpCodeChanges {
    pub lines_added: u64,
    pub lines_removed: u64,
    pub files_changed: u64,
}

pub struct WarpConfigManager {
    /// Warp 配置目录
    warp_dir: PathBuf,
    /// warp.sqlite 路径
    db_path: PathBuf,
    /// rules 目录
    rules_dir: PathBuf,
}

impl WarpConfigManager {
    pub fn new() -> Result<Self, String> {
        let user_home = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        // 按平台确定 Warp 配置目录和数据库路径
        let (warp_dir, db_path) = Self::resolve_paths(&user_home)?;
        let rules_dir = warp_dir.join("rules");

        Ok(Self {
            warp_dir,
            db_path,
            rules_dir,
        })
    }

    /// 按平台解析 Warp 路径
    fn resolve_paths(user_home: &PathBuf) -> Result<(PathBuf, PathBuf), String> {
        #[cfg(target_os = "macos")]
        {
            let warp_dir = user_home.join(".warp");
            let db_path = user_home
                .join("Library")
                .join("Group Containers")
                .join("2BBY89MBSN.dev.warp")
                .join("Library")
                .join("Application Support")
                .join("dev.warp.Warp-Stable")
                .join("warp.sqlite");
            Ok((warp_dir, db_path))
        }

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .unwrap_or_else(|_| user_home.join("AppData").join("Local").to_string_lossy().to_string());

            // Windows 上 Warp 数据全部在 %LOCALAPPDATA%\Warp\Warp\
            let warp_dir = PathBuf::from(&local_app_data).join("Warp").join("Warp");
            let db_path = warp_dir.join("data").join("warp.sqlite");
            Ok((warp_dir, db_path))
        }

        #[cfg(target_os = "linux")]
        {
            let warp_dir = user_home.join(".warp");
            let db_path = user_home.join(".local").join("share").join("warp").join("warp.sqlite");
            Ok((warp_dir, db_path))
        }
    }

    // ==================== 路径获取 ====================

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.warp_dir
    }

    pub fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }

    pub fn get_rules_dir(&self) -> &PathBuf {
        &self.rules_dir
    }

    // ==================== 状态检测 ====================

    /// 检查 Warp 是否已安装
    pub fn is_installed(&self) -> bool {
        // 检查配置目录或数据库文件是否存在
        self.warp_dir.exists() || self.db_path.exists()
    }

    /// 检查数据库文件是否可访问
    pub fn is_db_accessible(&self) -> bool {
        self.db_path.exists() && fs::metadata(&self.db_path).is_ok()
    }

    /// 确保 rules 目录存在
    pub fn ensure_rules_dir(&self) -> Result<(), String> {
        if !self.rules_dir.exists() {
            fs::create_dir_all(&self.rules_dir)
                .map_err(|e| format!("创建 Warp rules 目录失败: {}", e))?;
        }
        Ok(())
    }

    // ==================== 数据库用量读取 ====================

    /// 从 warp.sqlite 读取用量统计
    pub fn get_usage_stats(&self) -> Result<WarpUsageStats, String> {
        if !self.db_path.exists() {
            return Err("Warp 数据库文件不存在，请确认 Warp 已安装且使用过".to_string());
        }

        // 以只读模式打开数据库
        let conn = Connection::open_with_flags(
            &self.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ).map_err(|e| {
            if e.to_string().contains("permission") || e.to_string().contains("Permission") {
                #[cfg(target_os = "macos")]
                return "无法访问 Warp 数据库，请在 系统设置 > 隐私与安全 > 完全磁盘访问权限 中授权本应用".to_string();
                #[cfg(not(target_os = "macos"))]
                return format!("无法访问 Warp 数据库: {}", e);
            }
            format!("打开 Warp 数据库失败: {}", e)
        })?;

        self.query_usage_stats(&conn)
    }

    /// 查询用量统计数据
    fn query_usage_stats(&self, conn: &Connection) -> Result<WarpUsageStats, String> {
        let mut stats = WarpUsageStats::default();

        // Warp 对话数据存储在 agent_conversations.conversation_data JSON 中
        // 其中包含 conversation_usage_metadata 字段
        let table_exists = self.check_table_exists(conn, "agent_conversations");

        if !table_exists {
            // 回退：尝试从其他表查找 usage 数据
            return self.try_query_from_any_table(conn);
        }

        // 从 agent_conversations 表查询 conversation_data
        let mut stmt = conn.prepare(
            "SELECT conversation_data FROM agent_conversations WHERE conversation_data IS NOT NULL"
        ).map_err(|e| format!("准备查询失败: {}", e))?;

        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        }).map_err(|e| format!("查询失败: {}", e))?;

        for row in rows {
            if let Ok(data_str) = row {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&data_str) {
                    // conversation_data 是外层 JSON，需要取其中的 conversation_usage_metadata
                    if let Some(metadata) = data.get("conversation_usage_metadata") {
                        self.accumulate_usage(&mut stats, metadata);
                    }
                }
            }
        }

        Ok(stats)
    }

    /// 检查表是否存在
    fn check_table_exists(&self, conn: &Connection, table_name: &str) -> bool {
        conn.prepare(&format!(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}'", table_name
        ))
        .and_then(|mut stmt| stmt.query_row([], |_| Ok(())))
        .is_ok()
    }

    /// 尝试从任何可能的表读取 usage 数据
    fn try_query_from_any_table(&self, conn: &Connection) -> Result<WarpUsageStats, String> {
        let mut stats = WarpUsageStats::default();

        // 获取所有表名
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table'"
        ).map_err(|e| format!("查询表列表失败: {}", e))?;

        let tables: Vec<String> = stmt.query_map([], |row| {
            row.get(0)
        }).map_err(|e| format!("读取表列表失败: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        // 在每个表中查找包含 usage metadata 的列
        for table in &tables {
            if let Ok(mut col_stmt) = conn.prepare(&format!("PRAGMA table_info('{}')", table)) {
                let columns: Vec<String> = match col_stmt.query_map([], |row| {
                    let name: String = row.get(1)?;
                    Ok(name)
                }) {
                    Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
                    Err(_) => continue,
                };

                for col in &columns {
                    if col.contains("usage") || col.contains("metadata") {
                        if let Ok(mut data_stmt) = conn.prepare(
                            &format!("SELECT \"{}\" FROM \"{}\" WHERE \"{}\" IS NOT NULL", col, table, col)
                        ) {
                            if let Ok(rows) = data_stmt.query_map([], |row| {
                                let val: String = row.get(0)?;
                                Ok(val)
                            }) {
                                for row in rows.flatten() {
                                    if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&row) {
                                        if metadata.get("token_usage").is_some() || metadata.get("credits_spent").is_some() {
                                            self.accumulate_usage(&mut stats, &metadata);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// 累加单条对话的用量数据
    fn accumulate_usage(&self, stats: &mut WarpUsageStats, metadata: &serde_json::Value) {
        stats.conversation_count += 1;

        // 累加 credits_spent
        if let Some(credits) = metadata.get("credits_spent").and_then(|v| v.as_f64()) {
            stats.total_credits_spent += credits;
        }

        // 累加 token_usage（按模型分组）
        if let Some(token_usage) = metadata.get("token_usage").and_then(|v| v.as_array()) {
            for usage in token_usage {
                if let Some(model_id) = usage.get("model_id").and_then(|v| v.as_str()) {
                    let entry = stats.token_usage_by_model
                        .entry(model_id.to_string())
                        .or_insert_with(WarpTokenUsage::default);

                    if let Some(total_tokens) = usage.get("total_tokens").and_then(|v| v.as_u64()) {
                        entry.total_tokens += total_tokens;
                    }
                    if let Some(byok_tokens) = usage.get("byok_tokens").and_then(|v| v.as_u64()) {
                        entry.byok_tokens += byok_tokens;
                    }
                }
            }
        }

        // 累加工具使用统计
        if let Some(tool_usage) = metadata.get("tool_usage_metadata") {
            Self::add_tool_count(&mut stats.tool_usage.run_command_count, tool_usage, "run_command_stats");
            Self::add_tool_count(&mut stats.tool_usage.read_files_count, tool_usage, "read_files_stats");
            Self::add_tool_count(&mut stats.tool_usage.search_codebase_count, tool_usage, "search_codebase_stats");
            Self::add_tool_count(&mut stats.tool_usage.grep_count, tool_usage, "grep_stats");
            Self::add_tool_count(&mut stats.tool_usage.file_glob_count, tool_usage, "file_glob_stats");
            Self::add_tool_count(&mut stats.tool_usage.call_mcp_tool_count, tool_usage, "call_mcp_tool_stats");
            Self::add_tool_count(&mut stats.tool_usage.apply_file_diff_count, tool_usage, "apply_file_diff_stats");

            // 代码变更统计
            if let Some(diff_stats) = tool_usage.get("apply_file_diff_stats") {
                if let Some(added) = diff_stats.get("lines_added").and_then(|v| v.as_u64()) {
                    stats.code_changes.lines_added += added;
                }
                if let Some(removed) = diff_stats.get("lines_removed").and_then(|v| v.as_u64()) {
                    stats.code_changes.lines_removed += removed;
                }
                if let Some(changed) = diff_stats.get("files_changed").and_then(|v| v.as_u64()) {
                    stats.code_changes.files_changed += changed;
                }
            }
        }
    }

    /// 累加单个工具的 count
    fn add_tool_count(target: &mut u64, tool_usage: &serde_json::Value, key: &str) {
        if let Some(count) = tool_usage.get(key)
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
        {
            *target += count;
        }
    }
}
