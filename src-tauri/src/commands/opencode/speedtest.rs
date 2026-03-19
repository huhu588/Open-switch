// Speed Test 延迟测试命令
// 测试 API 端点的延迟和可用性

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 速度测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub status_code: Option<u16>,
    pub error_message: Option<String>,
    pub quality: SpeedQuality,
}

/// 速度质量评级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpeedQuality {
    Excellent, // < 200ms
    Good,      // 200-500ms
    Fair,      // 500-1000ms
    Poor,      // > 1000ms
    Failed,    // 连接失败
}

impl SpeedQuality {
    fn from_latency(latency_ms: u64) -> Self {
        if latency_ms < 200 {
            SpeedQuality::Excellent
        } else if latency_ms < 500 {
            SpeedQuality::Good
        } else if latency_ms < 1000 {
            SpeedQuality::Fair
        } else {
            SpeedQuality::Poor
        }
    }
}

/// 批量测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSpeedTestResult {
    pub provider_name: String,
    pub base_url: String,
    pub results: Vec<SpeedTestResult>,
    pub average_latency_ms: Option<u64>,
    pub success_rate: f64,
    pub overall_quality: SpeedQuality,
}

/// 测试单个端点延迟
#[tauri::command]
pub async fn test_endpoint_latency(
    base_url: String,
    api_key: Option<String>,
    model_type: String,
) -> Result<SpeedTestResult, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;
    
    // 根据不同的模型类型构建测试 URL
    let test_url = match model_type.as_str() {
        "claude" => {
            // Anthropic API: /v1/messages
            format!("{}/v1/messages", base_url.trim_end_matches('/'))
        }
        "codex" | "openai" => {
            // OpenAI API: /v1/models
            format!("{}/v1/models", base_url.trim_end_matches('/'))
        }
        "gemini" => {
            // Gemini API: 使用简单的 GET 请求测试
            format!("{}/v1/models", base_url.trim_end_matches('/'))
        }
        _ => {
            // 默认使用 OpenAI 格式
            format!("{}/v1/models", base_url.trim_end_matches('/'))
        }
    };
    
    let start = Instant::now();
    
    // 构建请求
    let mut request = client.get(&test_url);
    
    // 添加认证头
    if let Some(ref key) = api_key {
        match model_type.as_str() {
            "claude" => {
                request = request
                    .header("x-api-key", key)
                    .header("anthropic-version", "2023-06-01");
            }
            _ => {
                request = request.header("Authorization", format!("Bearer {}", key));
            }
        }
    }
    
    // 发送请求
    match request.send().await {
        Ok(response) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            let status_code = response.status().as_u16();
            
            // 2xx 或 401/403 (未授权但端点可达) 都算连接成功
            let success = status_code < 500;
            let quality = if success {
                SpeedQuality::from_latency(latency_ms)
            } else {
                SpeedQuality::Failed
            };
            
            Ok(SpeedTestResult {
                success,
                latency_ms: Some(latency_ms),
                status_code: Some(status_code),
                error_message: if !success {
                    Some(format!("HTTP {}", status_code))
                } else {
                    None
                },
                quality,
            })
        }
        Err(e) => {
            let latency_ms = start.elapsed().as_millis() as u64;
            
            Ok(SpeedTestResult {
                success: false,
                latency_ms: Some(latency_ms),
                status_code: None,
                error_message: Some(e.to_string()),
                quality: SpeedQuality::Failed,
            })
        }
    }
}

/// 批量测试端点（多次测试取平均）
#[tauri::command]
pub async fn batch_test_endpoint(
    provider_name: String,
    base_url: String,
    api_key: Option<String>,
    model_type: String,
    test_count: Option<u32>,
) -> Result<BatchSpeedTestResult, String> {
    let count = test_count.unwrap_or(3).min(10); // 最多测试 10 次
    let mut results = Vec::new();
    
    for _ in 0..count {
        let result = test_endpoint_latency(
            base_url.clone(),
            api_key.clone(),
            model_type.clone(),
        ).await?;
        
        results.push(result);
        
        // 测试间隔 100ms
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 计算统计数据
    let successful_results: Vec<&SpeedTestResult> = results
        .iter()
        .filter(|r| r.success)
        .collect();
    
    let success_rate = successful_results.len() as f64 / results.len() as f64;
    
    let average_latency_ms = if !successful_results.is_empty() {
        let total: u64 = successful_results
            .iter()
            .filter_map(|r| r.latency_ms)
            .sum();
        Some(total / successful_results.len() as u64)
    } else {
        None
    };
    
    let overall_quality = if success_rate < 0.5 {
        SpeedQuality::Failed
    } else if let Some(avg) = average_latency_ms {
        SpeedQuality::from_latency(avg)
    } else {
        SpeedQuality::Failed
    };
    
    Ok(BatchSpeedTestResult {
        provider_name,
        base_url,
        results,
        average_latency_ms,
        success_rate,
        overall_quality,
    })
}

/// 测试多个提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTestConfig {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model_type: String,
}

#[tauri::command]
pub async fn test_multiple_providers(
    providers: Vec<ProviderTestConfig>,
) -> Result<Vec<BatchSpeedTestResult>, String> {
    let mut results = Vec::new();
    
    for provider in providers {
        let result = batch_test_endpoint(
            provider.name,
            provider.base_url,
            provider.api_key,
            provider.model_type,
            Some(3),
        ).await?;
        
        results.push(result);
    }
    
    // 按延迟排序
    results.sort_by(|a, b| {
        match (&a.average_latency_ms, &b.average_latency_ms) {
            (Some(a_lat), Some(b_lat)) => a_lat.cmp(b_lat),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    Ok(results)
}

// ============================================================================
// Provider URL 延迟测试命令
// ============================================================================

/// 单个 URL 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlTestResult {
    pub url: String,
    pub latency_ms: Option<u64>,
    pub success: bool,
    pub quality: String,
    pub error_message: Option<String>,
}

/// Provider URLs 测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUrlsTestResult {
    pub provider_name: String,
    pub results: Vec<UrlTestResult>,
    pub fastest_url: Option<String>,
    pub fastest_latency_ms: Option<u64>,
}

/// 测试 Provider 的所有 URL
#[tauri::command]
pub async fn test_provider_urls(
    provider_name: String,
    urls: Vec<String>,
    api_key: Option<String>,
    model_type: String,
    test_count: Option<u32>,
) -> Result<ProviderUrlsTestResult, String> {
    let count = test_count.unwrap_or(3).min(5); // 每个 URL 测试 3-5 次
    let mut results: Vec<UrlTestResult> = Vec::new();
    
    for url in urls {
        let mut latencies: Vec<u64> = Vec::new();
        let mut last_error: Option<String> = None;
        let mut success_count = 0;
        
        for _ in 0..count {
            let result = test_endpoint_latency(
                url.clone(),
                api_key.clone(),
                model_type.clone(),
            ).await?;
            
            if result.success {
                success_count += 1;
                if let Some(latency) = result.latency_ms {
                    latencies.push(latency);
                }
            } else {
                last_error = result.error_message;
            }
            
            // 测试间隔 100ms
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // 计算平均延迟
        let average_latency = if !latencies.is_empty() {
            Some(latencies.iter().sum::<u64>() / latencies.len() as u64)
        } else {
            None
        };
        
        let success = success_count > count / 2; // 超过一半成功才算成功
        let quality = match average_latency {
            Some(ms) if ms < 200 => "excellent",
            Some(ms) if ms < 500 => "good",
            Some(ms) if ms < 1000 => "fair",
            Some(_) => "poor",
            None => "failed",
        };
        
        results.push(UrlTestResult {
            url,
            latency_ms: average_latency,
            success,
            quality: quality.to_string(),
            error_message: if !success { last_error } else { None },
        });
    }
    
    // 按延迟排序找出最快的
    results.sort_by(|a, b| {
        match (&a.latency_ms, &b.latency_ms) {
            (Some(a_lat), Some(b_lat)) => a_lat.cmp(b_lat),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    let fastest = results.iter().find(|r| r.success && r.latency_ms.is_some());
    let fastest_url = fastest.map(|r| r.url.clone());
    let fastest_latency_ms = fastest.and_then(|r| r.latency_ms);
    
    Ok(ProviderUrlsTestResult {
        provider_name,
        results,
        fastest_url,
        fastest_latency_ms,
    })
}

/// 测试 Provider 所有 URL 并自动选择最快的（整合测试和选择）
#[tauri::command]
pub async fn test_and_auto_select_fastest(
    provider_name: String,
    urls: Vec<String>,
    api_key: Option<String>,
    model_type: String,
    config_manager: tauri::State<'_, std::sync::Mutex<crate::modules::opencode_config::ConfigManager>>,
) -> Result<ProviderUrlsTestResult, String> {
    // 先测试所有 URL
    let test_result = test_provider_urls(
        provider_name.clone(),
        urls,
        api_key,
        model_type,
        Some(3),
    ).await?;
    
    // 如果有最快的 URL，更新配置
    if let Some(ref fastest_url) = test_result.fastest_url {
        let mut manager = config_manager.lock()
            .map_err(|e| format!("获取配置管理器失败: {}", e))?;
        
        // 读取当前配置
        let mut config = manager.opencode().read_config()
            .map_err(|e| format!("读取配置失败: {}", e))?;
        
        // 获取 provider 并更新
        if let Some(provider) = config.get_provider_mut(&provider_name) {
            // 更新所有 URL 的延迟测试结果
            for result in &test_result.results {
                provider.update_url_latency(&result.url, result.latency_ms);
            }
            
            // 设置最快的 URL 为激活 URL
            provider.set_active_base_url(fastest_url);
            
            // 写回配置
            manager.opencode_mut().write_config(&config)
                .map_err(|e| format!("保存配置失败: {}", e))?;
        }
    }
    
    Ok(test_result)
}
