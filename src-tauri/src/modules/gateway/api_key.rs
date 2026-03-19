use super::db;
use super::types::{ApiKeyCreatePayload, GatewayApiKey};
use sha2::{Digest, Sha256};

const KEY_PREFIX: &str = "sk-gw-";

pub fn generate_api_key(payload: &ApiKeyCreatePayload) -> Result<(String, GatewayApiKey), String> {
    let raw_key = format!(
        "{}{}",
        KEY_PREFIX,
        uuid::Uuid::new_v4().to_string().replace("-", "")
    );

    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    let key_prefix = &raw_key[..12];
    let id = uuid::Uuid::new_v4().to_string();

    let allowed_models = payload
        .allowed_models
        .as_ref()
        .map(|models| serde_json::to_string(models).unwrap_or_default());

    db::insert_api_key(
        &id,
        &payload.name,
        &key_hash,
        key_prefix,
        allowed_models.as_deref(),
    )?;

    let api_key = GatewayApiKey {
        id,
        name: payload.name.clone(),
        key_hash,
        key_prefix: key_prefix.to_string(),
        allowed_models,
        enabled: true,
        created_at: chrono::Utc::now().timestamp(),
        last_used_at: None,
        usage_count: 0,
    };

    Ok((raw_key, api_key))
}

pub fn verify_api_key(raw_key: &str) -> Result<Option<GatewayApiKey>, String> {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());

    db::validate_api_key(&key_hash)
}

pub fn is_model_allowed(api_key: &GatewayApiKey, model: &str) -> bool {
    match &api_key.allowed_models {
        Some(models_json) => {
            if models_json.is_empty() || models_json == "[]" {
                return true;
            }
            match serde_json::from_str::<Vec<String>>(models_json) {
                Ok(models) => {
                    if models.is_empty() {
                        return true;
                    }
                    models.iter().any(|m| m == model || m == "*")
                }
                Err(_) => true,
            }
        }
        None => true,
    }
}

pub fn increment_usage(key_hash: &str) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    db::with_db(|conn| {
        conn.execute(
            "UPDATE gateway_api_keys SET usage_count = usage_count + 1, last_used_at = ?1 WHERE key_hash = ?2",
            rusqlite::params![now, key_hash],
        )?;
        Ok(())
    })
}
