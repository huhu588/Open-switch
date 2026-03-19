//! Usage Logger - 记录 API 请求使用情况

use super::parser::TokenUsage;
use crate::modules::opencode_db::{lock_conn, Database};
use crate::opencode_error::AppError;
use crate::modules::proxy::types::AppType;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::SystemTime;

/// 模型定价信息
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_cost_per_million: Decimal,
    pub output_cost_per_million: Decimal,
    pub cache_read_cost_per_million: Decimal,
    pub cache_creation_cost_per_million: Decimal,
}

/// 成本明细
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub cache_read_cost: Decimal,
    pub cache_creation_cost: Decimal,
    pub total_cost: Decimal,
}

/// 记录使用量到数据库
pub fn log_usage(
    db: &Database,
    provider_id: &str,
    provider_name: Option<&str>,
    app_type: AppType,
    model: &str,
    usage: TokenUsage,
    latency_ms: u64,
    status_code: u16,
) -> Result<(), AppError> {
    let conn = lock_conn!(db.conn);

    // 获取模型定价
    let pricing = get_model_pricing(&conn, model)?;
    
    // 计算成本
    let cost = calculate_cost(&usage, pricing.as_ref());

    let request_id = uuid::Uuid::new_v4().to_string();
    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO proxy_request_logs (
            request_id, provider_id, provider_name, app_type, model,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens,
            input_cost_usd, output_cost_usd, cache_read_cost_usd, cache_creation_cost_usd, total_cost_usd,
            latency_ms, status_code, is_streaming, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
        rusqlite::params![
            request_id,
            provider_id,
            provider_name,
            app_type.as_str(),
            model,
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_read_tokens,
            usage.cache_creation_tokens,
            cost.input_cost.to_string(),
            cost.output_cost.to_string(),
            cost.cache_read_cost.to_string(),
            cost.cache_creation_cost.to_string(),
            cost.total_cost.to_string(),
            latency_ms as i64,
            status_code as i64,
            0, // is_streaming
            created_at,
        ],
    )
    .map_err(|e| AppError::Database(format!("记录使用量失败: {e}")))?;

    Ok(())
}

/// 获取模型定价
fn get_model_pricing(conn: &rusqlite::Connection, model_id: &str) -> Result<Option<ModelPricing>, AppError> {
    // 清洗模型名称
    let cleaned = clean_model_id(model_id);

    let result = conn.query_row(
        "SELECT input_cost_per_million, output_cost_per_million,
                cache_read_cost_per_million, cache_creation_cost_per_million
         FROM model_pricing WHERE model_id = ?1",
        [&cleaned],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        },
    );

    match result {
        Ok((input, output, cache_read, cache_creation)) => {
            Ok(Some(ModelPricing {
                input_cost_per_million: Decimal::from_str(&input).unwrap_or(Decimal::ZERO),
                output_cost_per_million: Decimal::from_str(&output).unwrap_or(Decimal::ZERO),
                cache_read_cost_per_million: Decimal::from_str(&cache_read).unwrap_or(Decimal::ZERO),
                cache_creation_cost_per_million: Decimal::from_str(&cache_creation).unwrap_or(Decimal::ZERO),
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(format!("查询模型定价失败: {e}"))),
    }
}

/// 清洗模型 ID
fn clean_model_id(model_id: &str) -> String {
    // 去前缀（如 "anthropic/"）
    let without_prefix = model_id
        .rsplit_once('/')
        .map_or(model_id, |(_, r)| r);
    
    // 去后缀（如 ":v2"）
    let without_suffix = without_prefix
        .split(':')
        .next()
        .unwrap_or(without_prefix);
    
    // @ 替换为 -
    without_suffix.trim().replace('@', "-")
}

/// 计算成本
fn calculate_cost(usage: &TokenUsage, pricing: Option<&ModelPricing>) -> CostBreakdown {
    let pricing = match pricing {
        Some(p) => p,
        None => {
            return CostBreakdown {
                input_cost: Decimal::ZERO,
                output_cost: Decimal::ZERO,
                cache_read_cost: Decimal::ZERO,
                cache_creation_cost: Decimal::ZERO,
                total_cost: Decimal::ZERO,
            };
        }
    };

    let million = Decimal::from(1_000_000u64);

    // 计算各项成本
    // 注意：input_tokens 需要扣除 cache_read_tokens（避免缓存部分被重复计费）
    let billable_input_tokens = (usage.input_tokens as u64).saturating_sub(usage.cache_read_tokens as u64);
    
    let input_cost = Decimal::from(billable_input_tokens) * pricing.input_cost_per_million / million;
    let output_cost = Decimal::from(usage.output_tokens as u64) * pricing.output_cost_per_million / million;
    let cache_read_cost = Decimal::from(usage.cache_read_tokens as u64) * pricing.cache_read_cost_per_million / million;
    let cache_creation_cost = Decimal::from(usage.cache_creation_tokens as u64) * pricing.cache_creation_cost_per_million / million;

    let total_cost = input_cost + output_cost + cache_read_cost + cache_creation_cost;

    CostBreakdown {
        input_cost,
        output_cost,
        cache_read_cost,
        cache_creation_cost,
        total_cost,
    }
}
