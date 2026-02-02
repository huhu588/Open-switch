// 深链接命令模块
// 处理 aiswitch:// 协议的深链接，用于一键配置 Provider

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// 深链接解析后的 Provider 配置数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkProviderData {
    /// 服务商名称
    pub name: String,
    /// API Key
    pub api_key: String,
    /// Base URL
    pub base_url: String,
    /// API 协议类型 (openai, anthropic, azure 等)
    pub model_type: String,
    /// 自定义模型列表 (可选)
    pub models: Option<Vec<DeepLinkModelData>>,
    /// 描述 (可选)
    pub description: Option<String>,
}

/// 深链接中的模型数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkModelData {
    /// 模型 ID
    pub id: String,
    /// 模型显示名称 (可选)
    pub name: Option<String>,
}

/// 解析深链接 URL 的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDeepLink {
    /// 操作类型
    pub action: String,
    /// Provider 配置数据
    pub provider: Option<DeepLinkProviderData>,
    /// 错误信息
    pub error: Option<String>,
}

/// 解析深链接 URL
/// 
/// 支持的 URL 格式：
/// aiswitch://add-provider?name=xxx&api_key=xxx&base_url=xxx&model_type=openai&models=model1,model2&description=xxx
/// 
/// 参数说明：
/// - name: 服务商名称 (必需)
/// - api_key: API Key (必需)
/// - base_url: Base URL (必需)
/// - model_type: API 协议类型 (必需，如 openai, anthropic, azure, gemini 等)
/// - models: 自定义模型列表，逗号分隔 (可选)
/// - description: 描述 (可选)
#[tauri::command]
pub fn parse_deep_link(url: String) -> ParsedDeepLink {
    // 解析 URL
    let parsed = match Url::parse(&url) {
        Ok(u) => u,
        Err(e) => {
            return ParsedDeepLink {
                action: "error".to_string(),
                provider: None,
                error: Some(format!("无效的 URL 格式: {}", e)),
            };
        }
    };

    // 检查 scheme
    if parsed.scheme() != "aiswitch" {
        return ParsedDeepLink {
            action: "error".to_string(),
            provider: None,
            error: Some("不支持的协议，请使用 aiswitch://".to_string()),
        };
    }

    // 获取 action (host 部分)
    let action = parsed.host_str().unwrap_or("").to_string();

    match action.as_str() {
        "add-provider" => parse_add_provider_url(&parsed),
        _ => ParsedDeepLink {
            action: "error".to_string(),
            provider: None,
            error: Some(format!("不支持的操作: {}", action)),
        },
    }
}

/// 解析添加 Provider 的深链接
fn parse_add_provider_url(url: &Url) -> ParsedDeepLink {
    // 收集查询参数
    let params: HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    // 必需参数检查
    let name = match params.get("name") {
        Some(n) if !n.is_empty() => n.clone(),
        _ => {
            return ParsedDeepLink {
                action: "add-provider".to_string(),
                provider: None,
                error: Some("缺少必需参数: name (服务商名称)".to_string()),
            };
        }
    };

    let api_key = match params.get("api_key") {
        Some(k) if !k.is_empty() => k.clone(),
        _ => {
            return ParsedDeepLink {
                action: "add-provider".to_string(),
                provider: None,
                error: Some("缺少必需参数: api_key".to_string()),
            };
        }
    };

    let base_url = match params.get("base_url") {
        Some(u) if !u.is_empty() => u.clone(),
        _ => {
            return ParsedDeepLink {
                action: "add-provider".to_string(),
                provider: None,
                error: Some("缺少必需参数: base_url".to_string()),
            };
        }
    };

    let model_type = match params.get("model_type") {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return ParsedDeepLink {
                action: "add-provider".to_string(),
                provider: None,
                error: Some("缺少必需参数: model_type (API 协议类型)".to_string()),
            };
        }
    };

    // 可选参数
    let description = params.get("description").cloned();

    // 解析模型列表
    let models = params.get("models").map(|m| {
        m.split(',')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let parts: Vec<&str> = s.split(':').collect();
                DeepLinkModelData {
                    id: parts.first().unwrap_or(&s).to_string(),
                    name: parts.get(1).map(|n| n.to_string()),
                }
            })
            .collect()
    });

    ParsedDeepLink {
        action: "add-provider".to_string(),
        provider: Some(DeepLinkProviderData {
            name,
            api_key,
            base_url,
            model_type,
            models,
            description,
        }),
        error: None,
    }
}

/// 生成深链接 URL（用于测试和文档）
#[tauri::command]
pub fn generate_deep_link(
    name: String,
    api_key: String,
    base_url: String,
    model_type: String,
    models: Option<Vec<String>>,
    description: Option<String>,
) -> String {
    let mut url = format!(
        "aiswitch://add-provider?name={}&api_key={}&base_url={}&model_type={}",
        urlencoding::encode(&name),
        urlencoding::encode(&api_key),
        urlencoding::encode(&base_url),
        urlencoding::encode(&model_type),
    );

    if let Some(models) = models {
        if !models.is_empty() {
            url.push_str(&format!("&models={}", urlencoding::encode(&models.join(","))));
        }
    }

    if let Some(desc) = description {
        url.push_str(&format!("&description={}", urlencoding::encode(&desc)));
    }

    url
}
