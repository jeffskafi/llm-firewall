// gateway/provider.rs
use anyhow::Result;
use reqwest::Client;
use super::types::{UpReq, UpRes};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    async fn chat(&self, req: UpReq) -> Result<UpRes>;
}

// ---------- OpenAI ----------
pub struct OpenAI { pub base: String, pub key: String }

#[async_trait::async_trait]
impl Provider for OpenAI {
    async fn chat(&self, req: UpReq) -> Result<UpRes> {
        let url = format!("{}/v1/chat/completions", self.base);
        let client = Client::new();
        let res = client
            .post(url)
            .bearer_auth(&self.key)
            .json(&req)
            .send()
            .await?;
        let status = res.status().as_u16();
        let headers = res.headers().clone();
        let body = res.bytes().await?;
        Ok(UpRes { status, headers, body })
    }
}

// ---------- Anthropic ----------
pub struct Anthropic { pub api_key: String }

#[async_trait::async_trait]
impl Provider for Anthropic {
    async fn chat(&self, req: UpReq) -> Result<UpRes> {
        let client = Client::new();
        let res = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .json(&req)
            .send()
            .await?;
        Ok(UpRes {
            status: res.status().as_u16(),
            headers: res.headers().clone(),
            body: res.bytes().await?,
        })
    }
}

// ---------- Ollama (local) ----------
pub struct Ollama { pub base: String }

#[async_trait::async_trait]
impl Provider for Ollama {
    async fn chat(&self, req: UpReq) -> Result<UpRes> {
        let url = format!("{}/api/chat", self.base);        // Ollama’s REST path
        let res = Client::new().post(url).json(&req).send().await?;
        Ok(UpRes {
            status: res.status().as_u16(),
            headers: res.headers().clone(),
            body: res.bytes().await?,
        })
    }
}

use std::collections::HashMap;

// ---------- Router ----------
#[derive(Clone)]
pub struct Router {
    map: HashMap<String, Arc<dyn Provider>>,
}

impl Router {
    pub fn new() -> Self { Self { map: HashMap::new() } }

    pub fn add_prefix<P: Provider + 'static>(&mut self, prefix: &str, p: P) {
        self.map.insert(prefix.into(), Arc::new(p));
    }

    pub fn route(&self, model: &str) -> Option<Arc<dyn Provider>> {
        self.map
            .iter()
            .find(|(pref, _)| model.starts_with(pref.as_str()))
            .map(|(_, prov)| prov.clone())
    }
}