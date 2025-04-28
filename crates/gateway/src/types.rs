// gateway/types.rs
use serde::{Deserialize, Serialize};

/// 100 % compatible with OpenAI & Anthropic chat body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpReq {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(flatten)]
    pub extra: serde_json::Value,   // stream, temperature, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct UpRes {
    /// raw bytes so we can stream straight back
    pub status: u16,
    pub headers: http::HeaderMap,
    pub body:  bytes::Bytes,
}
