use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpencodeAppError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("配置错误: {0}")]
    Config(#[from] crate::modules::opencode_config::ConfigError),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("代理错误: {0}")]
    Proxy(String),

    #[error("{0}")]
    Custom(String),
}

impl From<String> for OpencodeAppError {
    fn from(s: String) -> Self {
        OpencodeAppError::Custom(s)
    }
}

impl From<&str> for OpencodeAppError {
    fn from(s: &str) -> Self {
        OpencodeAppError::Custom(s.to_string())
    }
}

impl Serialize for OpencodeAppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppError = OpencodeAppError;
pub type Result<T> = std::result::Result<T, OpencodeAppError>;
