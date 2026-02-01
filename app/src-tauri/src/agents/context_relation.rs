use crate::pipeline::context::RecordContext;
use crate::paths::record_dir_from_audio;
use crate::store::evidence::{EvidenceRecord, EvidenceStore};

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use tokio::fs;
use crate::agents::agent::{strip_json_fences, Agent};
use crate::processing::utils::load_document_id;

const PROMPT_TEMPLATE: &str = r#"
You are analyzing a single document.

The document is a source, not the truth.
Do not infer intent, roles, or importance.
Do not generalize.
Only extract relationships that are explicitly stated in the text.

Document text:
---
{DOCUMENT_TEXT}
---

Extracted entities (with type):
{ENTITIES_JSON}

Task:
- Identify explicit relationships between any two entities mentioned in the document
- Only if the relationship is clearly stated in the text
- If no explicit relationship is stated, return an empty list

Return JSON only (no markdown):

[
  {{
    "from_type": "<entity_type>",
    "from": "<entity_text>",
    "to_type": "<entity_type>",
    "to": "<entity_text>",
    "confidence": 0.0-1.0
  }}
]
"#;

pub struct ContextRelationAgent {
    pub openai_api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entity {
    #[serde(rename = "type")]
    entity_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct RelationCandidate {
    from_type: String,
    from: String,
    to_type: String,
    to: String,
    confidence: f32,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<crate::agents::context_relation::ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: crate::agents::context_relation::ChatMessage,
}


#[async_trait]
impl Agent for ContextRelationAgent {
    fn name(&self) -> &'static str {
        "CONTEXT_RELATION_AGENT"
    }

    async fn run_document(&self, ctx: &RecordContext) -> Result<()> {
        println!("▶ ContextRelationAgent: run_document");

        if self.openai_api_key.is_empty() {
            anyhow::bail!("OPENAI_API_KEY is missing");
        }

        let record_dir = record_dir_from_audio(&ctx.base_dir, &ctx.audio_file).await?;
        let entities_path = record_dir.join("entities.json");
        let text_path = record_dir.join("text.txt");

        if !entities_path.exists() || !text_path.exists() {
            println!("▶ ContextRelationAgent: missing input files, skipping");
            return Ok(());
        }

        let document_id = load_document_id(&record_dir).await?;

        let raw_entities = fs::read_to_string(&entities_path).await?;
        let document_text = fs::read_to_string(&text_path).await?;

        let entities: Vec<Entity> = {
            let v: Value = serde_json::from_str(&raw_entities)?;
            let arr = v.get("entities").context("entities missing")?;
            serde_json::from_value(arr.clone())?
        };

        if entities.len() < 2 {
            println!("▶ ContextRelationAgent: not enough entities");
            return Ok(());
        }

        let prompt = PROMPT_TEMPLATE
            .replace("{DOCUMENT_TEXT}", &document_text)
            .replace("{ENTITIES_JSON}", &serde_json::to_string_pretty(&entities)?);

        let request = ChatRequest {
            model: "gpt-4o-mini".into(),
            messages: vec![ChatMessage {
                role: "user".into(),
                content: prompt,
            }],
            temperature: 0.0,
        };

        let client = Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.openai_api_key)
            .json(&request)
            .send()
            .await?;

        let response: ChatResponse = response.json().await?;
        let raw = response.choices.get(0).map(|c| c.message.content.clone()).unwrap_or("[]".into());
        let cleaned = strip_json_fences(&raw);

        let candidates: Vec<RelationCandidate> =
            serde_json::from_str(cleaned).unwrap_or_default();

        if candidates.is_empty() {
            println!("▶ ContextRelationAgent: no relations found");
            return Ok(());
        }

        let mut store = EvidenceStore::load_or_create(&record_dir).await?;

        for c in candidates {
            if c.confidence <= 0.0 {
                continue;
            }

            let key = format!(
                "{}:{}|associated_with|{}:{}",
                c.from_type.to_lowercase(),
                c.from.to_lowercase(),
                c.to_type.to_lowercase(),
                c.to.to_lowercase()
            );

            let record = EvidenceRecord::new(
                key,
                c.from_type.clone(),
                c.from.clone(),
                "associated_with".into(),
                c.to.clone(),
            );

            store.add_or_update(
                record,
                &document_id,
                c.confidence,
                self.name(),
            );
        }

        store.save().await?;
        println!("▶ ContextRelationAgent finished");

        Ok(())
    }
}
