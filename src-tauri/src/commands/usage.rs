// 使用统计相关的 Tauri commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Utc, Timelike, Datelike};
use tauri::State;
use crate::database::Database;

/// 单条使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub timestamp: i64,  // Unix timestamp in milliseconds
    pub provider_name: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost: f64,  // in USD
    pub request_type: String,  // "chat", "completion", etc.
}

/// 使用统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_requests: u64,
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_cache_read_tokens: u64,
}

/// 按时间段的使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrend {
    pub label: String,  // 时间标签，如 "00:00", "01:00" 或 "Mon", "Tue"
    pub timestamp: i64,
    pub requests: u64,
    pub cost: f64,
    pub tokens: u64,
}

/// 使用统计数据存储
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageData {
    pub records: Vec<UsageRecord>,
}

impl UsageData {
    /// 获取存储路径
    fn get_storage_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("无法获取用户目录")?;
        let config_dir = home.join(".config").join("opencode");
        fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
        Ok(config_dir.join("usage_stats.json"))
    }

    /// 从文件加载
    pub fn load() -> Result<Self, String> {
        let path = Self::get_storage_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// 保存到文件
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_storage_path()?;
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// 添加记录
    pub fn add_record(&mut self, record: UsageRecord) {
        self.records.push(record);
    }

    /// 获取时间范围内的记录
    pub fn get_records_in_range(&self, start: i64, end: i64) -> Vec<&UsageRecord> {
        self.records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }

    /// 计算统计摘要
    pub fn calculate_summary(&self, start: i64, end: i64) -> UsageSummary {
        let records = self.get_records_in_range(start, end);
        
        UsageSummary {
            total_requests: records.len() as u64,
            total_cost: records.iter().map(|r| r.cost).sum(),
            total_input_tokens: records.iter().map(|r| r.input_tokens).sum(),
            total_output_tokens: records.iter().map(|r| r.output_tokens).sum(),
            total_cache_creation_tokens: records.iter().map(|r| r.cache_creation_tokens).sum(),
            total_cache_read_tokens: records.iter().map(|r| r.cache_read_tokens).sum(),
        }
    }

    /// 清理旧数据（保留最近 30 天）
    pub fn cleanup_old_records(&mut self) {
        let thirty_days_ago = Utc::now().timestamp_millis() - 30 * 24 * 60 * 60 * 1000;
        self.records.retain(|r| r.timestamp >= thirty_days_ago);
    }
}

/// 获取使用统计摘要
#[tauri::command]
pub async fn get_usage_summary(period: String) -> Result<UsageSummary, String> {
    let data = UsageData::load()?;
    let now = Utc::now().timestamp_millis();
    
    let start = match period.as_str() {
        "24h" => now - 24 * 60 * 60 * 1000,
        "7d" => now - 7 * 24 * 60 * 60 * 1000,
        "30d" => now - 30 * 24 * 60 * 60 * 1000,
        _ => now - 24 * 60 * 60 * 1000,
    };
    
    Ok(data.calculate_summary(start, now))
}

/// 获取使用趋势数据
#[tauri::command]
pub async fn get_usage_trend(period: String) -> Result<Vec<UsageTrend>, String> {
    let data = UsageData::load()?;
    let now = Utc::now();
    let now_millis = now.timestamp_millis();
    
    let (start_millis, interval_millis, format_fn): (i64, i64, Box<dyn Fn(DateTime<Utc>) -> String>) = match period.as_str() {
        "24h" => {
            // 按小时分组，过去24小时
            let start = now_millis - 24 * 60 * 60 * 1000;
            let interval = 60 * 60 * 1000; // 1 hour
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{:02}:00", dt.hour()));
            (start, interval, format_fn)
        },
        "7d" => {
            // 按天分组，过去7天
            let start = now_millis - 7 * 24 * 60 * 60 * 1000;
            let interval = 24 * 60 * 60 * 1000; // 1 day
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| {
                let weekday = dt.weekday();
                match weekday {
                    chrono::Weekday::Mon => "周一".to_string(),
                    chrono::Weekday::Tue => "周二".to_string(),
                    chrono::Weekday::Wed => "周三".to_string(),
                    chrono::Weekday::Thu => "周四".to_string(),
                    chrono::Weekday::Fri => "周五".to_string(),
                    chrono::Weekday::Sat => "周六".to_string(),
                    chrono::Weekday::Sun => "周日".to_string(),
                }
            });
            (start, interval, format_fn)
        },
        "30d" => {
            // 按天分组，过去30天
            let start = now_millis - 30 * 24 * 60 * 60 * 1000;
            let interval = 24 * 60 * 60 * 1000; // 1 day
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{}/{}", dt.format("%m"), dt.format("%d")));
            (start, interval, format_fn)
        },
        _ => {
            let start = now_millis - 24 * 60 * 60 * 1000;
            let interval = 60 * 60 * 1000;
            let format_fn: Box<dyn Fn(DateTime<Utc>) -> String> = Box::new(|dt: DateTime<Utc>| format!("{:02}:00", dt.hour()));
            (start, interval, format_fn)
        }
    };

    // 创建时间桶
    let mut buckets: Vec<UsageTrend> = Vec::new();
    let mut current = start_millis;
    
    while current < now_millis {
        let dt = DateTime::from_timestamp_millis(current).unwrap_or(now);
        buckets.push(UsageTrend {
            label: format_fn(dt),
            timestamp: current,
            requests: 0,
            cost: 0.0,
            tokens: 0,
        });
        current += interval_millis;
    }

    // 将记录分配到时间桶
    let records = data.get_records_in_range(start_millis, now_millis);
    for record in records {
        // 找到对应的时间桶
        for bucket in &mut buckets {
            if record.timestamp >= bucket.timestamp && record.timestamp < bucket.timestamp + interval_millis {
                bucket.requests += 1;
                bucket.cost += record.cost;
                bucket.tokens += record.input_tokens + record.output_tokens;
                break;
            }
        }
    }

    Ok(buckets)
}

/// 添加使用记录
#[tauri::command]
pub async fn add_usage_record(
    provider_name: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: Option<u64>,
    cache_read_tokens: Option<u64>,
    cost: Option<f64>,
    request_type: Option<String>,
) -> Result<(), String> {
    let mut data = UsageData::load()?;
    
    let record = UsageRecord {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: Utc::now().timestamp_millis(),
        provider_name,
        model,
        input_tokens,
        output_tokens,
        cache_creation_tokens: cache_creation_tokens.unwrap_or(0),
        cache_read_tokens: cache_read_tokens.unwrap_or(0),
        cost: cost.unwrap_or(0.0),
        request_type: request_type.unwrap_or_else(|| "chat".to_string()),
    };
    
    data.add_record(record);
    data.cleanup_old_records();  // 清理旧数据
    data.save()?;
    
    Ok(())
}

/// 清除所有使用统计
#[tauri::command]
pub async fn clear_usage_stats() -> Result<(), String> {
    let data = UsageData::default();
    data.save()
}

/// 获取按服务商分组的统计
#[tauri::command]
pub async fn get_usage_by_provider(period: String) -> Result<HashMap<String, UsageSummary>, String> {
    let data = UsageData::load()?;
    let now = Utc::now().timestamp_millis();
    
    let start = match period.as_str() {
        "24h" => now - 24 * 60 * 60 * 1000,
        "7d" => now - 7 * 24 * 60 * 60 * 1000,
        "30d" => now - 30 * 24 * 60 * 60 * 1000,
        _ => now - 24 * 60 * 60 * 1000,
    };
    
    let records = data.get_records_in_range(start, now);
    let mut by_provider: HashMap<String, Vec<&UsageRecord>> = HashMap::new();
    
    for record in records {
        by_provider
            .entry(record.provider_name.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }
    
    let mut result: HashMap<String, UsageSummary> = HashMap::new();
    for (provider, records) in by_provider {
        result.insert(provider, UsageSummary {
            total_requests: records.len() as u64,
            total_cost: records.iter().map(|r| r.cost).sum(),
            total_input_tokens: records.iter().map(|r| r.input_tokens).sum(),
            total_output_tokens: records.iter().map(|r| r.output_tokens).sum(),
            total_cache_creation_tokens: records.iter().map(|r| r.cache_creation_tokens).sum(),
            total_cache_read_tokens: records.iter().map(|r| r.cache_read_tokens).sum(),
        });
    }
    
    Ok(result)
}

// ============================================================================
// 模型定价管理
// ============================================================================

/// 模型定价信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub model_id: String,
    pub display_name: String,
    pub input_cost_per_million: String,
    pub output_cost_per_million: String,
    pub cache_read_cost_per_million: String,
    pub cache_creation_cost_per_million: String,
}

/// 获取所有模型定价
#[tauri::command]
pub async fn get_model_pricing_list(db: State<'_, Arc<Database>>) -> Result<Vec<ModelPricing>, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut stmt = conn.prepare(
        "SELECT model_id, display_name, input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM model_pricing ORDER BY display_name"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let rows = stmt.query_map([], |row| {
        Ok(ModelPricing {
            model_id: row.get(0)?,
            display_name: row.get(1)?,
            input_cost_per_million: row.get(2)?,
            output_cost_per_million: row.get(3)?,
            cache_read_cost_per_million: row.get(4)?,
            cache_creation_cost_per_million: row.get(5)?,
        })
    }).map_err(|e| format!("查询失败: {e}"))?;
    
    let mut pricing_list = Vec::new();
    for row in rows {
        pricing_list.push(row.map_err(|e| format!("读取行失败: {e}"))?);
    }
    
    Ok(pricing_list)
}

/// 更新模型定价
#[tauri::command]
pub async fn update_model_pricing(
    db: State<'_, Arc<Database>>,
    model_id: String,
    input_cost: String,
    output_cost: String,
    cache_read_cost: String,
    cache_creation_cost: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    conn.execute(
        "UPDATE model_pricing SET 
            input_cost_per_million = ?2,
            output_cost_per_million = ?3,
            cache_read_cost_per_million = ?4,
            cache_creation_cost_per_million = ?5
         WHERE model_id = ?1",
        rusqlite::params![model_id, input_cost, output_cost, cache_read_cost, cache_creation_cost],
    ).map_err(|e| format!("更新失败: {e}"))?;
    
    Ok(())
}

/// 添加新的模型定价
#[tauri::command]
pub async fn add_model_pricing(
    db: State<'_, Arc<Database>>,
    model_id: String,
    display_name: String,
    input_cost: String,
    output_cost: String,
    cache_read_cost: String,
    cache_creation_cost: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    conn.execute(
        "INSERT OR REPLACE INTO model_pricing (
            model_id, display_name, input_cost_per_million, output_cost_per_million,
            cache_read_cost_per_million, cache_creation_cost_per_million
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![model_id, display_name, input_cost, output_cost, cache_read_cost, cache_creation_cost],
    ).map_err(|e| format!("添加失败: {e}"))?;
    
    Ok(())
}

/// 删除模型定价
#[tauri::command]
pub async fn delete_model_pricing(
    db: State<'_, Arc<Database>>,
    model_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    conn.execute(
        "DELETE FROM model_pricing WHERE model_id = ?1",
        rusqlite::params![model_id],
    ).map_err(|e| format!("删除失败: {e}"))?;
    
    Ok(())
}

/// 重置模型定价为默认值
#[tauri::command]
pub async fn reset_model_pricing(db: State<'_, Arc<Database>>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    // 删除所有现有数据
    conn.execute("DELETE FROM model_pricing", [])
        .map_err(|e| format!("清除失败: {e}"))?;
    
    // 重新插入默认数据
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
            "INSERT INTO model_pricing (
                model_id, display_name, input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![model_id, display_name, input, output, cache_read, cache_creation],
        ).map_err(|e| format!("插入默认定价失败: {e}"))?;
    }
    
    Ok(())
}

// ============================================================================
// 服务商特定模型定价
// ============================================================================

/// 服务商模型定价信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelPricing {
    pub id: Option<i64>,
    pub provider_id: String,
    pub model_id: String,
    pub input_cost_per_million: String,
    pub output_cost_per_million: String,
    pub cache_read_cost_per_million: String,
    pub cache_creation_cost_per_million: String,
}

/// 获取指定服务商的所有模型定价
#[tauri::command]
pub async fn get_provider_model_pricing(
    db: State<'_, Arc<Database>>,
    provider_id: String,
) -> Result<Vec<ProviderModelPricing>, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, model_id, input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM provider_model_pricing WHERE provider_id = ?1 ORDER BY model_id"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let rows = stmt.query_map([&provider_id], |row| {
        Ok(ProviderModelPricing {
            id: Some(row.get(0)?),
            provider_id: row.get(1)?,
            model_id: row.get(2)?,
            input_cost_per_million: row.get(3)?,
            output_cost_per_million: row.get(4)?,
            cache_read_cost_per_million: row.get(5)?,
            cache_creation_cost_per_million: row.get(6)?,
        })
    }).map_err(|e| format!("查询失败: {e}"))?;
    
    let mut pricing_list = Vec::new();
    for row in rows {
        pricing_list.push(row.map_err(|e| format!("读取行失败: {e}"))?);
    }
    
    Ok(pricing_list)
}

/// 获取所有服务商的定价（按服务商分组）
#[tauri::command]
pub async fn get_all_provider_pricing(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<ProviderModelPricing>, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut stmt = conn.prepare(
        "SELECT id, provider_id, model_id, input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM provider_model_pricing ORDER BY provider_id, model_id"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let rows = stmt.query_map([], |row| {
        Ok(ProviderModelPricing {
            id: Some(row.get(0)?),
            provider_id: row.get(1)?,
            model_id: row.get(2)?,
            input_cost_per_million: row.get(3)?,
            output_cost_per_million: row.get(4)?,
            cache_read_cost_per_million: row.get(5)?,
            cache_creation_cost_per_million: row.get(6)?,
        })
    }).map_err(|e| format!("查询失败: {e}"))?;
    
    let mut pricing_list = Vec::new();
    for row in rows {
        pricing_list.push(row.map_err(|e| format!("读取行失败: {e}"))?);
    }
    
    Ok(pricing_list)
}

/// 设置服务商特定的模型定价
#[tauri::command]
pub async fn set_provider_model_pricing(
    db: State<'_, Arc<Database>>,
    provider_id: String,
    model_id: String,
    input_cost: String,
    output_cost: String,
    cache_read_cost: String,
    cache_creation_cost: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    conn.execute(
        "INSERT OR REPLACE INTO provider_model_pricing (
            provider_id, model_id, input_cost_per_million, output_cost_per_million,
            cache_read_cost_per_million, cache_creation_cost_per_million, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
        rusqlite::params![provider_id, model_id, input_cost, output_cost, cache_read_cost, cache_creation_cost],
    ).map_err(|e| format!("保存失败: {e}"))?;
    
    Ok(())
}

/// 删除服务商特定的模型定价
#[tauri::command]
pub async fn delete_provider_model_pricing(
    db: State<'_, Arc<Database>>,
    provider_id: String,
    model_id: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    conn.execute(
        "DELETE FROM provider_model_pricing WHERE provider_id = ?1 AND model_id = ?2",
        rusqlite::params![provider_id, model_id],
    ).map_err(|e| format!("删除失败: {e}"))?;
    
    Ok(())
}

/// 获取所有已配置定价的服务商列表
#[tauri::command]
pub async fn get_pricing_providers(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    let mut stmt = conn.prepare(
        "SELECT DISTINCT provider_id FROM provider_model_pricing ORDER BY provider_id"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let rows = stmt.query_map([], |row| row.get(0))
        .map_err(|e| format!("查询失败: {e}"))?;
    
    let mut providers = Vec::new();
    for row in rows {
        providers.push(row.map_err(|e| format!("读取行失败: {e}"))?);
    }
    
    // 添加默认的本地服务商
    let default_providers = vec!["claude_local", "codex_local", "gemini_local"];
    for p in default_providers {
        if !providers.contains(&p.to_string()) {
            providers.push(p.to_string());
        }
    }
    
    Ok(providers)
}

// ============================================================================
// 诊断命令
// ============================================================================

/// 数据诊断结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataDiagnostics {
    pub total_records: u32,
    pub min_created_at: Option<i64>,
    pub max_created_at: Option<i64>,
    pub providers: Vec<ProviderCount>,
    pub sample_records: Vec<SampleRecord>,
}

/// 示例记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleRecord {
    pub request_id: String,
    pub model: String,
    pub created_at: i64,
    pub total_cost: String,
}

/// 服务商记录数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCount {
    pub provider_id: String,
    pub provider_name: String,
    pub count: u32,
}

/// 诊断数据库数据
#[tauri::command]
pub async fn diagnose_usage_data(db: State<'_, Arc<Database>>) -> Result<DataDiagnostics, String> {
    let conn = db.conn.lock().map_err(|e| format!("获取数据库锁失败: {e}"))?;
    
    // 获取总记录数和时间范围
    let (total, min_ts, max_ts): (u32, Option<i64>, Option<i64>) = conn.query_row(
        "SELECT COUNT(*), MIN(created_at), MAX(created_at) FROM proxy_request_logs",
        [],
        |row| Ok((
            row.get::<_, i64>(0)? as u32,
            row.get(1)?,
            row.get(2)?,
        )),
    ).map_err(|e| format!("查询失败: {e}"))?;
    
    // 获取按服务商分组的记录数
    let mut stmt = conn.prepare(
        "SELECT provider_id, provider_name, COUNT(*) as cnt 
         FROM proxy_request_logs 
         GROUP BY provider_id 
         ORDER BY cnt DESC"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let providers = stmt.query_map([], |row| {
        Ok(ProviderCount {
            provider_id: row.get(0)?,
            provider_name: row.get(1)?,
            count: row.get::<_, i64>(2)? as u32,
        })
    }).map_err(|e| format!("查询失败: {e}"))?
    .filter_map(|r| r.ok())
    .collect();
    
    // 获取最近 5 条示例记录
    let mut stmt = conn.prepare(
        "SELECT request_id, model, created_at, total_cost_usd 
         FROM proxy_request_logs 
         ORDER BY created_at DESC 
         LIMIT 5"
    ).map_err(|e| format!("准备查询失败: {e}"))?;
    
    let sample_records = stmt.query_map([], |row| {
        Ok(SampleRecord {
            request_id: row.get(0)?,
            model: row.get(1)?,
            created_at: row.get(2)?,
            total_cost: row.get(3)?,
        })
    }).map_err(|e| format!("查询失败: {e}"))?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(DataDiagnostics {
        total_records: total,
        min_created_at: min_ts,
        max_created_at: max_ts,
        providers,
        sample_records,
    })
}

// ============================================================================
// 会话统计和工具调用统计
// ============================================================================

// 重新导出 schema 中的类型
pub use crate::database::schema::{SessionStatsSummary, ToolCallStats};

/// 获取会话统计汇总
#[tauri::command]
pub async fn get_session_stats_summary(
    db: State<'_, Arc<Database>>,
    period: String,
    provider_id: Option<String>,
) -> Result<SessionStatsSummary, String> {
    let now = chrono::Utc::now().timestamp();
    
    let start_ts = match period.as_str() {
        "24h" => Some(now - 24 * 60 * 60),
        "7d" => Some(now - 7 * 24 * 60 * 60),
        "30d" => Some(now - 30 * 24 * 60 * 60),
        "all" => None,
        _ => Some(now - 24 * 60 * 60),
    };
    
    db.get_session_stats_summary(start_ts, Some(now), provider_id.as_deref())
        .map_err(|e| format!("获取会话统计失败: {e}"))
}

/// 获取工具调用统计
#[tauri::command]
pub async fn get_tool_call_stats(
    db: State<'_, Arc<Database>>,
    period: String,
    provider_id: Option<String>,
) -> Result<Vec<ToolCallStats>, String> {
    let now = chrono::Utc::now().timestamp();
    
    let start_ts = match period.as_str() {
        "24h" => Some(now - 24 * 60 * 60),
        "7d" => Some(now - 7 * 24 * 60 * 60),
        "30d" => Some(now - 30 * 24 * 60 * 60),
        "all" => None,
        _ => Some(now - 24 * 60 * 60),
    };
    
    db.get_tool_call_stats(start_ts, Some(now), provider_id.as_deref())
        .map_err(|e| format!("获取工具调用统计失败: {e}"))
}
