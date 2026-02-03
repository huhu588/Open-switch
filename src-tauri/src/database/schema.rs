//! 数据库 Schema 定义和迁移
//!
//! 负责数据库表结构的创建和版本迁移

use super::{lock_conn, Database, SCHEMA_VERSION};
use crate::error::AppError;
use rusqlite::Connection;

impl Database {
    /// 创建所有数据库表
    pub(crate) fn create_tables(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        Self::create_tables_on_conn(&conn)
    }

    /// 在指定连接上创建表
    pub(crate) fn create_tables_on_conn(conn: &Connection) -> Result<(), AppError> {
        // 1. 代理请求日志表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS proxy_request_logs (
                request_id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                provider_name TEXT,
                app_type TEXT NOT NULL,
                model TEXT NOT NULL,
                request_model TEXT,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                cache_read_tokens INTEGER NOT NULL DEFAULT 0,
                cache_creation_tokens INTEGER NOT NULL DEFAULT 0,
                input_cost_usd TEXT NOT NULL DEFAULT '0',
                output_cost_usd TEXT NOT NULL DEFAULT '0',
                cache_read_cost_usd TEXT NOT NULL DEFAULT '0',
                cache_creation_cost_usd TEXT NOT NULL DEFAULT '0',
                total_cost_usd TEXT NOT NULL DEFAULT '0',
                latency_ms INTEGER NOT NULL,
                first_token_ms INTEGER,
                status_code INTEGER NOT NULL,
                error_message TEXT,
                is_streaming INTEGER NOT NULL DEFAULT 0,
                cost_multiplier TEXT NOT NULL DEFAULT '1.0',
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 proxy_request_logs 表失败: {e}")))?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_logs_provider 
             ON proxy_request_logs(provider_id, app_type)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 provider 索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_logs_created_at 
             ON proxy_request_logs(created_at)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 created_at 索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_request_logs_model 
             ON proxy_request_logs(model)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 model 索引失败: {e}")))?;

        // 2. 模型定价表（默认全局定价）
        conn.execute(
            "CREATE TABLE IF NOT EXISTS model_pricing (
                model_id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                input_cost_per_million TEXT NOT NULL,
                output_cost_per_million TEXT NOT NULL,
                cache_read_cost_per_million TEXT NOT NULL DEFAULT '0',
                cache_creation_cost_per_million TEXT NOT NULL DEFAULT '0'
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 model_pricing 表失败: {e}")))?;

        // 2.1 服务商特定模型定价表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_model_pricing (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                input_cost_per_million TEXT NOT NULL,
                output_cost_per_million TEXT NOT NULL,
                cache_read_cost_per_million TEXT NOT NULL DEFAULT '0',
                cache_creation_cost_per_million TEXT NOT NULL DEFAULT '0',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(provider_id, model_id)
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 provider_model_pricing 表失败: {e}")))?;

        // 3. 代理配置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS proxy_config (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                proxy_enabled INTEGER NOT NULL DEFAULT 0,
                listen_address TEXT NOT NULL DEFAULT '127.0.0.1',
                listen_port INTEGER NOT NULL DEFAULT 15721,
                takeover_claude INTEGER NOT NULL DEFAULT 0,
                takeover_codex INTEGER NOT NULL DEFAULT 0,
                takeover_gemini INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 proxy_config 表失败: {e}")))?;

        // 初始化代理配置
        conn.execute(
            "INSERT OR IGNORE INTO proxy_config (id) VALUES (1)",
            [],
        )
        .map_err(|e| AppError::Database(format!("初始化代理配置失败: {e}")))?;

        // 4. 配置备份表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS proxy_live_backup (
                app_type TEXT PRIMARY KEY,
                original_config TEXT NOT NULL,
                backed_up_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 proxy_live_backup 表失败: {e}")))?;

        // 5. 会话统计表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_stats (
                session_id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                provider_id TEXT,
                conversation_count INTEGER NOT NULL DEFAULT 0,
                tool_call_count INTEGER NOT NULL DEFAULT 0,
                files_changed INTEGER NOT NULL DEFAULT 0,
                lines_added INTEGER NOT NULL DEFAULT 0,
                lines_deleted INTEGER NOT NULL DEFAULT 0,
                response_time_ms INTEGER NOT NULL DEFAULT 0,
                thinking_time_ms INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 session_stats 表失败: {e}")))?;

        // 创建 session_stats 索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_stats_source 
             ON session_stats(source)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 session_stats source 索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_session_stats_created_at 
             ON session_stats(created_at)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 session_stats created_at 索引失败: {e}")))?;

        // 6. 工具调用统计表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tool_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                call_count INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES session_stats(session_id)
            )",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 tool_calls 表失败: {e}")))?;

        // 创建 tool_calls 索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_calls_session 
             ON tool_calls(session_id)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 tool_calls session 索引失败: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_calls_tool_name 
             ON tool_calls(tool_name)",
            [],
        )
        .map_err(|e| AppError::Database(format!("创建 tool_calls tool_name 索引失败: {e}")))?;

        Ok(())
    }

    /// 应用数据库迁移
    pub(crate) fn apply_migrations(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        
        let version = Self::get_user_version(&conn)?;
        
        if version > SCHEMA_VERSION {
            return Err(AppError::Database(format!(
                "数据库版本过新（{version}），当前应用仅支持 {SCHEMA_VERSION}"
            )));
        }

        if version < SCHEMA_VERSION {
            // 执行迁移
            Self::set_user_version(&conn, SCHEMA_VERSION)?;
        }

        Ok(())
    }

    /// 确保模型定价数据已初始化
    pub(crate) fn ensure_model_pricing_seeded(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM model_pricing", [], |row| row.get(0))
            .map_err(|e| AppError::Database(format!("统计模型定价数据失败: {e}")))?;

        if count == 0 {
            Self::seed_model_pricing(&conn)?;
        }

        Ok(())
    }

    /// 插入默认模型定价数据
    fn seed_model_pricing(conn: &Connection) -> Result<(), AppError> {
        let pricing_data = [
            // Claude 4.5 系列
            ("claude-opus-4-5-20251101", "Claude Opus 4.5", "5", "25", "0.50", "6.25"),
            ("claude-sonnet-4-5-20250929", "Claude Sonnet 4.5", "3", "15", "0.30", "3.75"),
            ("claude-haiku-4-5-20251001", "Claude Haiku 4.5", "1", "5", "0.10", "1.25"),
            // Claude 4 系列
            ("claude-opus-4-20250514", "Claude Opus 4", "15", "75", "1.50", "18.75"),
            ("claude-sonnet-4-20250514", "Claude Sonnet 4", "3", "15", "0.30", "3.75"),
            // Claude 3.5 系列
            ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", "0.80", "4", "0.08", "1"),
            ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", "3", "15", "0.30", "3.75"),
            // GPT-5.2 系列
            ("gpt-5.2", "GPT-5.2", "1.75", "14", "0.175", "0"),
            ("gpt-5.2-codex", "GPT-5.2 Codex", "1.75", "14", "0.175", "0"),
            // GPT-5.1 系列
            ("gpt-5.1", "GPT-5.1", "1.25", "10", "0.125", "0"),
            ("gpt-5.1-codex", "GPT-5.1 Codex", "1.25", "10", "0.125", "0"),
            // GPT-5 系列
            ("gpt-5", "GPT-5", "1.25", "10", "0.125", "0"),
            ("gpt-5-codex", "GPT-5 Codex", "1.25", "10", "0.125", "0"),
            // Gemini 3 系列
            ("gemini-3-pro-preview", "Gemini 3 Pro Preview", "2", "12", "0.2", "0"),
            ("gemini-3-flash-preview", "Gemini 3 Flash Preview", "0.5", "3", "0.05", "0"),
            // Gemini 2.5 系列
            ("gemini-2.5-pro", "Gemini 2.5 Pro", "1.25", "10", "0.125", "0"),
            ("gemini-2.5-flash", "Gemini 2.5 Flash", "0.3", "2.5", "0.03", "0"),
            // DeepSeek 系列
            ("deepseek-v3.2", "DeepSeek V3.2", "2.00", "3.00", "0.40", "0"),
            ("deepseek-v3", "DeepSeek V3", "2.00", "8.00", "0.40", "0"),
            // Kimi 系列
            ("kimi-k2-thinking", "Kimi K2 Thinking", "4.00", "16.00", "1.00", "0"),
            ("kimi-k2-0905", "Kimi K2", "4.00", "16.00", "1.00", "0"),
        ];

        for (model_id, display_name, input, output, cache_read, cache_creation) in pricing_data {
            conn.execute(
                "INSERT OR REPLACE INTO model_pricing (
                    model_id, display_name, input_cost_per_million, output_cost_per_million,
                    cache_read_cost_per_million, cache_creation_cost_per_million
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![model_id, display_name, input, output, cache_read, cache_creation],
            )
            .map_err(|e| AppError::Database(format!("插入模型定价失败: {e}")))?;
        }

        Ok(())
    }

    // --- 辅助方法 ---

    fn get_user_version(conn: &Connection) -> Result<i32, AppError> {
        conn.query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(|e| AppError::Database(format!("读取 user_version 失败: {e}")))
    }

    fn set_user_version(conn: &Connection, version: i32) -> Result<(), AppError> {
        let sql = format!("PRAGMA user_version = {version};");
        conn.execute(&sql, [])
            .map_err(|e| AppError::Database(format!("写入 user_version 失败: {e}")))?;
        Ok(())
    }
}

// ============================================================================
// 数据库查询方法
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 使用量汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub total_requests: u64,
    pub total_cost: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub success_rate: f32,
}

/// 每日统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStats {
    pub date: String,
    pub request_count: u64,
    pub total_cost: String,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

/// 使用趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTrend {
    pub period: String,
    pub request_count: u64,
    pub total_cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub top_model: Option<String>,
}

/// 模型使用量（用于堆叠图）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub request_count: u64,
}

/// 按模型分组的趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTrendData {
    pub period: String,
    pub models: Vec<ModelUsage>,
    pub total_tokens: u64,
    pub total_cost: f64,
}

/// Provider 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStats {
    pub provider_id: String,
    pub provider_name: String,
    pub request_count: u64,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost: String,
    pub success_rate: f32,
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfigDb {
    pub proxy_enabled: bool,
    pub listen_address: String,
    pub listen_port: u16,
    pub takeover_claude: bool,
    pub takeover_codex: bool,
    pub takeover_gemini: bool,
}

/// 会话统计汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatsSummary {
    pub total_conversations: u64,
    pub total_tool_calls: u64,
    pub total_files_changed: u64,
    pub total_lines_added: u64,
    pub total_lines_deleted: u64,
    pub total_response_time_ms: u64,
    pub total_thinking_time_ms: u64,
    pub avg_response_time_ms: f64,
    pub avg_thinking_time_ms: f64,
    pub session_count: u64,
}

/// 工具调用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallStats {
    pub tool_name: String,
    pub call_count: u64,
    pub percentage: f64,
}

impl Default for ProxyConfigDb {
    fn default() -> Self {
        Self {
            proxy_enabled: false,
            listen_address: "127.0.0.1".to_string(),
            listen_port: 15721,
            takeover_claude: false,
            takeover_codex: false,
            takeover_gemini: false,
        }
    }
}

impl Database {
    /// 获取使用量汇总
    pub fn get_usage_summary(&self, start_ts: Option<i64>, end_ts: Option<i64>) -> Result<UsageSummary, AppError> {
        let conn = lock_conn!(self.conn);

        let (where_clause, params): (String, Vec<i64>) = match (start_ts, end_ts) {
            (Some(start), Some(end)) => {
                ("WHERE created_at >= ?1 AND created_at <= ?2".to_string(), vec![start, end])
            }
            (Some(start), None) => {
                ("WHERE created_at >= ?1".to_string(), vec![start])
            }
            (None, Some(end)) => {
                ("WHERE created_at <= ?1".to_string(), vec![end])
            }
            (None, None) => (String::new(), vec![]),
        };

        let sql = format!(
            "SELECT
                COUNT(*) as total_requests,
                COALESCE(SUM(CAST(total_cost_usd AS REAL)), 0) as total_cost,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COALESCE(SUM(cache_creation_tokens), 0) as total_cache_creation_tokens,
                COALESCE(SUM(cache_read_tokens), 0) as total_cache_read_tokens,
                COALESCE(SUM(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 ELSE 0 END), 0) as success_count
            FROM proxy_request_logs
            {where_clause}"
        );

        let result = if params.is_empty() {
            conn.query_row(&sql, [], |row| {
                let total_requests: i64 = row.get(0)?;
                let total_cost: f64 = row.get(1)?;
                let total_input_tokens: i64 = row.get(2)?;
                let total_output_tokens: i64 = row.get(3)?;
                let total_cache_creation_tokens: i64 = row.get(4)?;
                let total_cache_read_tokens: i64 = row.get(5)?;
                let success_count: i64 = row.get(6)?;

                let success_rate = if total_requests > 0 {
                    (success_count as f32 / total_requests as f32) * 100.0
                } else {
                    0.0
                };

                Ok(UsageSummary {
                    total_requests: total_requests as u64,
                    total_cost: format!("{total_cost:.6}"),
                    total_input_tokens: total_input_tokens as u64,
                    total_output_tokens: total_output_tokens as u64,
                    total_cache_creation_tokens: total_cache_creation_tokens as u64,
                    total_cache_read_tokens: total_cache_read_tokens as u64,
                    success_rate,
                })
            })
        } else if params.len() == 1 {
            conn.query_row(&sql, [params[0]], |row| {
                let total_requests: i64 = row.get(0)?;
                let total_cost: f64 = row.get(1)?;
                let total_input_tokens: i64 = row.get(2)?;
                let total_output_tokens: i64 = row.get(3)?;
                let total_cache_creation_tokens: i64 = row.get(4)?;
                let total_cache_read_tokens: i64 = row.get(5)?;
                let success_count: i64 = row.get(6)?;

                let success_rate = if total_requests > 0 {
                    (success_count as f32 / total_requests as f32) * 100.0
                } else {
                    0.0
                };

                Ok(UsageSummary {
                    total_requests: total_requests as u64,
                    total_cost: format!("{total_cost:.6}"),
                    total_input_tokens: total_input_tokens as u64,
                    total_output_tokens: total_output_tokens as u64,
                    total_cache_creation_tokens: total_cache_creation_tokens as u64,
                    total_cache_read_tokens: total_cache_read_tokens as u64,
                    success_rate,
                })
            })
        } else {
            conn.query_row(&sql, [params[0], params[1]], |row| {
                let total_requests: i64 = row.get(0)?;
                let total_cost: f64 = row.get(1)?;
                let total_input_tokens: i64 = row.get(2)?;
                let total_output_tokens: i64 = row.get(3)?;
                let total_cache_creation_tokens: i64 = row.get(4)?;
                let total_cache_read_tokens: i64 = row.get(5)?;
                let success_count: i64 = row.get(6)?;

                let success_rate = if total_requests > 0 {
                    (success_count as f32 / total_requests as f32) * 100.0
                } else {
                    0.0
                };

                Ok(UsageSummary {
                    total_requests: total_requests as u64,
                    total_cost: format!("{total_cost:.6}"),
                    total_input_tokens: total_input_tokens as u64,
                    total_output_tokens: total_output_tokens as u64,
                    total_cache_creation_tokens: total_cache_creation_tokens as u64,
                    total_cache_read_tokens: total_cache_read_tokens as u64,
                    success_rate,
                })
            })
        };

        result.map_err(|e| AppError::Database(format!("查询使用量汇总失败: {e}")))
    }

    /// 获取使用趋势
    pub fn get_usage_trend(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        period: &str,
        provider_id: Option<&str>,
    ) -> Result<Vec<UsageTrend>, AppError> {
        let conn = lock_conn!(self.conn);

        // 根据时间段决定分组粒度
        let (group_format, _date_format) = match period {
            "24h" => ("%Y-%m-%d %H:00", "hour"),
            "7d" => ("%Y-%m-%d", "day"),
            "30d" => ("%Y-%m-%d", "day"),
            "all" => ("%Y-%m-%d", "day"),
            _ => ("%Y-%m-%d", "day"),
        };

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(start) = start_ts {
            conditions.push("created_at >= ?".to_string());
            params.push(start.into());
        }
        if let Some(end) = end_ts {
            conditions.push("created_at <= ?".to_string());
            params.push(end.into());
        }
        if let Some(pid) = provider_id {
            conditions.push("provider_id = ?".to_string());
            params.push(pid.to_string().into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT
                strftime('{group_format}', created_at, 'unixepoch', 'localtime') as period,
                COUNT(*) as request_count,
                COALESCE(SUM(CAST(total_cost_usd AS REAL)), 0) as total_cost,
                COALESCE(SUM(input_tokens), 0) as input_tokens,
                COALESCE(SUM(output_tokens), 0) as output_tokens
            FROM proxy_request_logs
            {where_clause}
            GROUP BY period
            ORDER BY period ASC"
        );

        // 计算每个时间段使用最多的模型
        let model_sql = format!(
            "SELECT
                strftime('{group_format}', created_at, 'unixepoch', 'localtime') as period,
                model,
                COUNT(*) as cnt
            FROM proxy_request_logs
            {where_clause}
            GROUP BY period, model
            ORDER BY period ASC, cnt DESC"
        );

        let mut top_models: HashMap<String, String> = HashMap::new();
        let mut model_stmt = conn.prepare(&model_sql)
            .map_err(|e| AppError::Database(format!("准备模型查询失败: {e}")))?;
        let mut model_rows = model_stmt.query(rusqlite::params_from_iter(params.clone()))
            .map_err(|e| AppError::Database(format!("查询模型统计失败: {e}")))?;

        while let Some(row) = model_rows.next().map_err(|e| AppError::Database(format!("读取模型行失败: {e}")))? {
            let period: String = row.get(0).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let model: Option<String> = row.get(1).ok();
            if !top_models.contains_key(&period) {
                if let Some(m) = model {
                    top_models.insert(period, m);
                }
            }
        }

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let mut rows = stmt.query(rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("查询使用趋势失败: {e}")))?;

        let mut trends = Vec::new();
        while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("读取行失败: {e}")))? {
            let period: String = row.get(0).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let top_model = top_models.get(&period).cloned();
            trends.push(UsageTrend {
                period,
                request_count: row.get::<_, i64>(1).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_cost: row.get(2).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?,
                input_tokens: row.get::<_, i64>(3).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                output_tokens: row.get::<_, i64>(4).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                top_model,
            });
        }

        Ok(trends)
    }

    /// 获取按模型分组的使用趋势（用于堆叠柱形图）
    pub fn get_usage_trend_by_model(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        period: &str,
        provider_id: Option<&str>,
    ) -> Result<Vec<ModelTrendData>, AppError> {
        let conn = lock_conn!(self.conn);

        // 根据时间段决定分组粒度
        let group_format = match period {
            "24h" => "%Y-%m-%d %H:00",
            "7d" => "%Y-%m-%d",
            "30d" => "%Y-%m-%d",
            "all" => "%Y-%m-%d",
            _ => "%Y-%m-%d",
        };

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(start) = start_ts {
            conditions.push("created_at >= ?".to_string());
            params.push(start.into());
        }
        if let Some(end) = end_ts {
            conditions.push("created_at <= ?".to_string());
            params.push(end.into());
        }
        if let Some(pid) = provider_id {
            conditions.push("provider_id = ?".to_string());
            params.push(pid.to_string().into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询每个时间段、每个模型的使用量
        let sql = format!(
            "SELECT
                strftime('{group_format}', created_at, 'unixepoch', 'localtime') as period,
                model,
                COALESCE(SUM(input_tokens), 0) as input_tokens,
                COALESCE(SUM(output_tokens), 0) as output_tokens,
                COALESCE(SUM(input_tokens + output_tokens), 0) as total_tokens,
                COALESCE(SUM(CAST(total_cost_usd AS REAL)), 0) as total_cost,
                COUNT(*) as request_count
            FROM proxy_request_logs
            {where_clause}
            GROUP BY period, model
            ORDER BY period ASC, total_tokens DESC"
        );

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let mut rows = stmt.query(rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("查询模型趋势失败: {e}")))?;

        // 按时间段分组
        let mut period_map: std::collections::BTreeMap<String, Vec<ModelUsage>> = std::collections::BTreeMap::new();
        
        while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("读取行失败: {e}")))? {
            let period: String = row.get(0).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let model: String = row.get::<_, Option<String>>(1)
                .map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?
                .unwrap_or_else(|| "unknown".to_string());
            
            let model_usage = ModelUsage {
                model,
                input_tokens: row.get::<_, i64>(2).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                output_tokens: row.get::<_, i64>(3).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_tokens: row.get::<_, i64>(4).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_cost: row.get(5).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?,
                request_count: row.get::<_, i64>(6).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
            };

            period_map.entry(period).or_default().push(model_usage);
        }

        // 转换为结果格式
        let result: Vec<ModelTrendData> = period_map
            .into_iter()
            .map(|(period, models)| {
                let total_tokens: u64 = models.iter().map(|m| m.total_tokens).sum();
                let total_cost: f64 = models.iter().map(|m| m.total_cost).sum();
                ModelTrendData {
                    period,
                    models,
                    total_tokens,
                    total_cost,
                }
            })
            .collect();

        Ok(result)
    }

    /// 获取每日趋势
    pub fn get_daily_trends(&self, start_ts: i64, end_ts: i64, bucket_seconds: i64) -> Result<Vec<DailyStats>, AppError> {
        let conn = lock_conn!(self.conn);

        let sql = "
            SELECT
                CAST((created_at - ?1) / ?3 AS INTEGER) as bucket_idx,
                COUNT(*) as request_count,
                COALESCE(SUM(CAST(total_cost_usd AS REAL)), 0) as total_cost,
                COALESCE(SUM(input_tokens + output_tokens), 0) as total_tokens,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens
            FROM proxy_request_logs
            WHERE created_at >= ?1 AND created_at <= ?2
            GROUP BY bucket_idx
            ORDER BY bucket_idx ASC";

        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let rows = stmt.query_map(rusqlite::params![start_ts, end_ts, bucket_seconds], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                DailyStats {
                    date: String::new(),
                    request_count: row.get::<_, i64>(1)? as u64,
                    total_cost: format!("{:.6}", row.get::<_, f64>(2)?),
                    total_tokens: row.get::<_, i64>(3)? as u64,
                    total_input_tokens: row.get::<_, i64>(4)? as u64,
                    total_output_tokens: row.get::<_, i64>(5)? as u64,
                },
            ))
        }).map_err(|e| AppError::Database(format!("查询每日趋势失败: {e}")))?;

        let mut map: HashMap<i64, DailyStats> = HashMap::new();
        for row in rows {
            let (bucket_idx, stat) = row.map_err(|e| AppError::Database(format!("读取行失败: {e}")))?;
            if bucket_idx >= 0 {
                map.insert(bucket_idx, stat);
            }
        }

        // 计算桶数
        let bucket_count = ((end_ts - start_ts) as f64 / bucket_seconds as f64).ceil() as i64;

        let mut stats = Vec::with_capacity(bucket_count as usize);
        for i in 0..bucket_count {
            let bucket_start_ts = start_ts + i * bucket_seconds;
            let date = chrono::DateTime::from_timestamp(bucket_start_ts, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string())
                .unwrap_or_default();

            if let Some(mut stat) = map.remove(&i) {
                stat.date = date;
                stats.push(stat);
            } else {
                stats.push(DailyStats {
                    date,
                    request_count: 0,
                    total_cost: "0.000000".to_string(),
                    total_tokens: 0,
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                });
            }
        }

        Ok(stats)
    }

    /// 获取 Provider 统计
    pub fn get_provider_stats(&self, start_ts: Option<i64>, end_ts: Option<i64>) -> Result<Vec<ProviderStats>, AppError> {
        let conn = lock_conn!(self.conn);

        let (where_clause, params): (String, Vec<i64>) = match (start_ts, end_ts) {
            (Some(start), Some(end)) => {
                ("WHERE created_at >= ?1 AND created_at <= ?2".to_string(), vec![start, end])
            }
            (Some(start), None) => {
                ("WHERE created_at >= ?1".to_string(), vec![start])
            }
            (None, Some(end)) => {
                ("WHERE created_at <= ?1".to_string(), vec![end])
            }
            (None, None) => (String::new(), vec![]),
        };

        let sql = format!(
            "SELECT
                provider_id,
                COALESCE(provider_name, provider_id) as provider_name,
                COUNT(*) as request_count,
                COALESCE(SUM(input_tokens + output_tokens), 0) as total_tokens,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COALESCE(SUM(cache_creation_tokens), 0) as total_cache_creation_tokens,
                COALESCE(SUM(cache_read_tokens), 0) as total_cache_read_tokens,
                COALESCE(SUM(CAST(total_cost_usd AS REAL)), 0) as total_cost,
                COALESCE(SUM(CASE WHEN status_code >= 200 AND status_code < 300 THEN 1 ELSE 0 END), 0) as success_count
            FROM proxy_request_logs
            {where_clause}
            GROUP BY provider_id
            ORDER BY total_cost DESC"
        );

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let mut rows = stmt.query(rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("查询 Provider 统计失败: {e}")))?;

        let mut stats = Vec::new();
        while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("读取行失败: {e}")))? {
            let request_count: i64 = row.get(2).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let success_count: i64 = row.get(9).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let success_rate = if request_count > 0 {
                (success_count as f32 / request_count as f32) * 100.0
            } else {
                0.0
            };

            stats.push(ProviderStats {
                provider_id: row.get(0).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?,
                provider_name: row.get(1).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?,
                request_count: request_count as u64,
                total_tokens: row.get::<_, i64>(3).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_input_tokens: row.get::<_, i64>(4).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_output_tokens: row.get::<_, i64>(5).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_cache_creation_tokens: row.get::<_, i64>(6).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_cache_read_tokens: row.get::<_, i64>(7).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))? as u64,
                total_cost: format!("{:.6}", row.get::<_, f64>(8).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?),
                success_rate,
            });
        }

        Ok(stats)
    }

    /// 获取代理配置
    pub fn get_proxy_config(&self) -> Result<ProxyConfigDb, AppError> {
        let conn = lock_conn!(self.conn);

        conn.query_row(
            "SELECT proxy_enabled, listen_address, listen_port, takeover_claude, takeover_codex, takeover_gemini
             FROM proxy_config WHERE id = 1",
            [],
            |row| {
                Ok(ProxyConfigDb {
                    proxy_enabled: row.get::<_, i32>(0)? != 0,
                    listen_address: row.get(1)?,
                    listen_port: row.get::<_, i32>(2)? as u16,
                    takeover_claude: row.get::<_, i32>(3)? != 0,
                    takeover_codex: row.get::<_, i32>(4)? != 0,
                    takeover_gemini: row.get::<_, i32>(5)? != 0,
                })
            },
        )
        .map_err(|e| AppError::Database(format!("获取代理配置失败: {e}")))
    }

    /// 更新代理配置
    pub fn update_proxy_config(&self, config: &ProxyConfigDb) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        conn.execute(
            "UPDATE proxy_config SET 
                proxy_enabled = ?1,
                listen_address = ?2,
                listen_port = ?3,
                takeover_claude = ?4,
                takeover_codex = ?5,
                takeover_gemini = ?6,
                updated_at = datetime('now')
             WHERE id = 1",
            rusqlite::params![
                if config.proxy_enabled { 1 } else { 0 },
                config.listen_address,
                config.listen_port as i32,
                if config.takeover_claude { 1 } else { 0 },
                if config.takeover_codex { 1 } else { 0 },
                if config.takeover_gemini { 1 } else { 0 },
            ],
        )
        .map_err(|e| AppError::Database(format!("更新代理配置失败: {e}")))?;

        Ok(())
    }

    /// 保存配置备份
    pub fn save_live_backup(&self, app_type: &str, original_config: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        conn.execute(
            "INSERT OR REPLACE INTO proxy_live_backup (app_type, original_config, backed_up_at)
             VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![app_type, original_config],
        )
        .map_err(|e| AppError::Database(format!("保存配置备份失败: {e}")))?;

        Ok(())
    }

    /// 获取配置备份
    pub fn get_live_backup(&self, app_type: &str) -> Result<Option<String>, AppError> {
        let conn = lock_conn!(self.conn);

        let result = conn.query_row(
            "SELECT original_config FROM proxy_live_backup WHERE app_type = ?1",
            [app_type],
            |row| row.get(0),
        );

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(format!("获取配置备份失败: {e}"))),
        }
    }

    /// 删除配置备份
    pub fn delete_live_backup(&self, app_type: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        conn.execute(
            "DELETE FROM proxy_live_backup WHERE app_type = ?1",
            [app_type],
        )
        .map_err(|e| AppError::Database(format!("删除配置备份失败: {e}")))?;

        Ok(())
    }

    /// 清除所有使用统计
    pub fn clear_usage_stats(&self) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        // 清除代理请求日志
        conn.execute("DELETE FROM proxy_request_logs", [])
            .map_err(|e| AppError::Database(format!("清除代理请求日志失败: {e}")))?;

        // 清除工具调用统计（需要在 session_stats 之前删除，因为外键约束）
        conn.execute("DELETE FROM tool_calls", [])
            .map_err(|e| AppError::Database(format!("清除工具调用统计失败: {e}")))?;

        // 清除会话统计
        conn.execute("DELETE FROM session_stats", [])
            .map_err(|e| AppError::Database(format!("清除会话统计失败: {e}")))?;

        Ok(())
    }

    /// 清理旧的请求日志（保留最近 N 天）
    pub fn cleanup_old_logs(&self, days: i64) -> Result<u64, AppError> {
        let conn = lock_conn!(self.conn);

        let cutoff = chrono::Utc::now().timestamp() - days * 24 * 60 * 60;

        let deleted = conn.execute(
            "DELETE FROM proxy_request_logs WHERE created_at < ?1",
            [cutoff],
        )
        .map_err(|e| AppError::Database(format!("清理旧日志失败: {e}")))?;

        Ok(deleted as u64)
    }

    // ============================================================================
    // 会话统计相关方法
    // ============================================================================

    /// 插入或更新会话统计
    pub fn upsert_session_stats(
        &self,
        session_id: &str,
        source: &str,
        provider_id: Option<&str>,
        conversation_count: u32,
        tool_call_count: u32,
        files_changed: u32,
        lines_added: u32,
        lines_deleted: u32,
        response_time_ms: u64,
        thinking_time_ms: u64,
        created_at: i64,
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO session_stats (
                session_id, source, provider_id, conversation_count, tool_call_count,
                files_changed, lines_added, lines_deleted, response_time_ms, thinking_time_ms,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(session_id) DO UPDATE SET
                conversation_count = conversation_count + excluded.conversation_count,
                tool_call_count = tool_call_count + excluded.tool_call_count,
                files_changed = files_changed + excluded.files_changed,
                lines_added = lines_added + excluded.lines_added,
                lines_deleted = lines_deleted + excluded.lines_deleted,
                response_time_ms = response_time_ms + excluded.response_time_ms,
                thinking_time_ms = thinking_time_ms + excluded.thinking_time_ms,
                updated_at = excluded.updated_at",
            rusqlite::params![
                session_id,
                source,
                provider_id,
                conversation_count,
                tool_call_count,
                files_changed,
                lines_added,
                lines_deleted,
                response_time_ms,
                thinking_time_ms,
                created_at,
                now,
            ],
        )
        .map_err(|e| AppError::Database(format!("插入会话统计失败: {e}")))?;

        Ok(())
    }

    /// 插入工具调用记录
    pub fn insert_tool_call(
        &self,
        session_id: &str,
        tool_name: &str,
        call_count: u32,
        created_at: i64,
    ) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);

        conn.execute(
            "INSERT INTO tool_calls (session_id, tool_name, call_count, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![session_id, tool_name, call_count, created_at],
        )
        .map_err(|e| AppError::Database(format!("插入工具调用记录失败: {e}")))?;

        Ok(())
    }

    /// 获取会话统计汇总
    pub fn get_session_stats_summary(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        provider_id: Option<&str>,
    ) -> Result<SessionStatsSummary, AppError> {
        let conn = lock_conn!(self.conn);

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(start) = start_ts {
            conditions.push("created_at >= ?".to_string());
            params.push(start.into());
        }
        if let Some(end) = end_ts {
            conditions.push("created_at <= ?".to_string());
            params.push(end.into());
        }
        if let Some(pid) = provider_id {
            conditions.push("provider_id = ?".to_string());
            params.push(pid.to_string().into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT
                COALESCE(SUM(conversation_count), 0) as total_conversations,
                COALESCE(SUM(tool_call_count), 0) as total_tool_calls,
                COALESCE(SUM(files_changed), 0) as total_files_changed,
                COALESCE(SUM(lines_added), 0) as total_lines_added,
                COALESCE(SUM(lines_deleted), 0) as total_lines_deleted,
                COALESCE(SUM(response_time_ms), 0) as total_response_time_ms,
                COALESCE(SUM(thinking_time_ms), 0) as total_thinking_time_ms,
                COUNT(*) as session_count
            FROM session_stats
            {where_clause}"
        );

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let result = stmt.query_row(rusqlite::params_from_iter(params), |row| {
            let total_conversations: i64 = row.get(0)?;
            let total_tool_calls: i64 = row.get(1)?;
            let total_files_changed: i64 = row.get(2)?;
            let total_lines_added: i64 = row.get(3)?;
            let total_lines_deleted: i64 = row.get(4)?;
            let total_response_time_ms: i64 = row.get(5)?;
            let total_thinking_time_ms: i64 = row.get(6)?;
            let session_count: i64 = row.get(7)?;

            let avg_response_time_ms = if total_conversations > 0 {
                total_response_time_ms as f64 / total_conversations as f64
            } else {
                0.0
            };
            let avg_thinking_time_ms = if total_conversations > 0 {
                total_thinking_time_ms as f64 / total_conversations as f64
            } else {
                0.0
            };

            Ok(SessionStatsSummary {
                total_conversations: total_conversations as u64,
                total_tool_calls: total_tool_calls as u64,
                total_files_changed: total_files_changed as u64,
                total_lines_added: total_lines_added as u64,
                total_lines_deleted: total_lines_deleted as u64,
                total_response_time_ms: total_response_time_ms as u64,
                total_thinking_time_ms: total_thinking_time_ms as u64,
                avg_response_time_ms,
                avg_thinking_time_ms,
                session_count: session_count as u64,
            })
        });

        result.map_err(|e| AppError::Database(format!("查询会话统计汇总失败: {e}")))
    }

    /// 获取工具调用统计
    pub fn get_tool_call_stats(
        &self,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        provider_id: Option<&str>,
    ) -> Result<Vec<ToolCallStats>, AppError> {
        let conn = lock_conn!(self.conn);

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        // 需要 JOIN session_stats 来过滤 provider_id
        let join_clause = if provider_id.is_some() {
            "JOIN session_stats s ON t.session_id = s.session_id"
        } else {
            ""
        };

        if let Some(start) = start_ts {
            conditions.push("t.created_at >= ?".to_string());
            params.push(start.into());
        }
        if let Some(end) = end_ts {
            conditions.push("t.created_at <= ?".to_string());
            params.push(end.into());
        }
        if let Some(pid) = provider_id {
            conditions.push("s.provider_id = ?".to_string());
            params.push(pid.to_string().into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT
                t.tool_name,
                COALESCE(SUM(t.call_count), 0) as total_calls
            FROM tool_calls t
            {join_clause}
            {where_clause}
            GROUP BY t.tool_name
            ORDER BY total_calls DESC"
        );

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| AppError::Database(format!("准备查询失败: {e}")))?;

        let mut rows = stmt.query(rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("查询工具调用统计失败: {e}")))?;

        let mut stats: Vec<ToolCallStats> = Vec::new();
        let mut total_calls: u64 = 0;

        while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("读取行失败: {e}")))? {
            let tool_name: String = row.get(0).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            let call_count: i64 = row.get(1).map_err(|e| AppError::Database(format!("读取字段失败: {e}")))?;
            total_calls += call_count as u64;
            stats.push(ToolCallStats {
                tool_name,
                call_count: call_count as u64,
                percentage: 0.0, // 稍后计算
            });
        }

        // 计算百分比
        for stat in &mut stats {
            stat.percentage = if total_calls > 0 {
                (stat.call_count as f64 / total_calls as f64) * 100.0
            } else {
                0.0
            };
        }

        Ok(stats)
    }

    /// 检查会话统计是否存在
    pub fn session_stats_exists(&self, session_id: &str) -> bool {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return false,
        };

        conn.query_row(
            "SELECT 1 FROM session_stats WHERE session_id = ?1",
            [session_id],
            |_| Ok(()),
        )
        .is_ok()
    }
}
