// crates/core/src/lib.rs  (append)
mod rules;
pub use rules::*;

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct AppConfig;

#[derive(Clone)]
pub struct Firewall {
    rules: Arc<Vec<Box<dyn Rule>>>,
}

impl Firewall {
    pub async fn new(_cfg: AppConfig) -> Result<Self> {
        // hard-code two starter rules; later load from YAML / DB
        let r: Vec<Box<dyn Rule>> = vec![
            Box::new(LengthRule { max_tokens: 4096 }),
            Box::new(RegexBlockRule { pattern: regex::Regex::new("(?i)\\bkill\\b").unwrap() }),
        ];
        Ok(Self { rules: Arc::new(r) })
    }

    pub fn evaluate(&self, input: &FirewallInput) -> Result<FirewallVerdict> {
        for rule in self.rules.iter() {
            let verdict = rule.check(input)?;
            if !verdict.allowed {
                return Ok(verdict);
            }
        }
        Ok(FirewallVerdict { allowed: true, reason: None })
    }
}
