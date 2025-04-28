// crates/core/src/rules.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallInput {
    pub prompt: String,
    pub model:  String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallVerdict {
    pub allowed: bool,
    pub reason:  Option<String>,
}

pub trait Rule: Send + Sync {
    fn check(&self, input: &FirewallInput) -> Result<FirewallVerdict>;
}

/// ---- Example rules ------------------------------------------------------

pub struct LengthRule {
    pub max_tokens: usize,
}

impl Rule for LengthRule {
    fn check(&self, input: &FirewallInput) -> Result<FirewallVerdict> {
        if input.prompt.split_whitespace().count() > self.max_tokens {
            return Ok(FirewallVerdict {
                allowed: false,
                reason: Some("prompt too long".into()),
            });
        }
        Ok(FirewallVerdict { allowed: true, reason: None })
    }
}

pub struct RegexBlockRule {
    pub pattern: regex::Regex,
}

impl Rule for RegexBlockRule {
    fn check(&self, input: &FirewallInput) -> Result<FirewallVerdict> {
        if self.pattern.is_match(&input.prompt) {
            return Ok(FirewallVerdict {
                allowed: false,
                reason: Some("matches blocked pattern".into()),
            });
        }
        Ok(FirewallVerdict { allowed: true, reason: None })
    }
}
