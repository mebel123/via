use async_trait::async_trait;
use serde::{Deserialize, Serialize};
// src/agents/agent.rs
use crate::pipeline::context::RecordContext;

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &'static str;
    async fn run_document(&self, ctx: &RecordContext) -> anyhow::Result<()>;
}

pub fn strip_json_fences(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with("```") {
        s.trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        s
    }
}
