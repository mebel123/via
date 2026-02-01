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

pub struct PersonRelationAgent {
    pub openai_api_key: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    message: ChatMessage,
}
#[derive(Debug, Deserialize)]
struct Entity {
    #[serde(rename = "type")]
    entity_type: String,
    text: String,
}
#[derive(Debug, Deserialize)]
struct RelationCandidate {
    person: String,
    organization: String,
    confidence: f32,
}



#[async_trait]
impl Agent for PersonRelationAgent {
    fn name(&self) -> &'static str {
        "PERSON_RELATION_AGENT"
    }

    async fn run_document(&self, ctx: &RecordContext) -> Result<()> {
        println!("▶ PersonRelationAgent: run_document");

        if self.openai_api_key.is_empty() {
            println!("▶ PersonRelationAgent: OPENAI_API_KEY is missing");
            anyhow::bail!("OPENAI_API_KEY is missing");
        }

        let record_dir = record_dir_from_audio(&ctx.base_dir, &ctx.audio_file)
            .await
            .context("record_dir_from_audio failed")?;
        println!("▶ PersonRelationAgent: record_dir={}", record_dir.display());

        let entities_path = record_dir.join("entities.json");
        let text_path = record_dir.join("text.txt");

        if !entities_path.exists() {
            println!("▶ PersonRelationAgent: no entities.json, skipping");
            return Ok(());
        }
        if !text_path.exists() {
            println!("▶ PersonRelationAgent: no text.txt, skipping");
            return Ok(());
        }

        let document_id = load_document_id(&record_dir).await?;

        println!("▶ PersonRelationAgent: reading {}", entities_path.display());
        let raw = fs::read_to_string(&entities_path)
            .await
            .context("failed to read entities.json")?;
        println!("▶ PersonRelationAgent: entities.json bytes={}", raw.len());

        // entities.json ist bei dir jetzt immer { "entities": [...] }, trotzdem robust bleiben
        let entities: Vec<Entity> = if raw.trim_start().starts_with('{') {
            let v: Value = serde_json::from_str(&raw)
                .context("failed to parse entities.json as Value")?;

            let arr = v.get("entities")
                .context("entities field missing in entities.json")?
                .clone();

            serde_json::from_value(arr).context("failed to deserialize entities array")?
        } else {
            serde_json::from_str(&raw).context("failed to deserialize entities array")?
        };

        let persons: Vec<String> = entities
            .iter()
            .filter(|e| e.entity_type.eq_ignore_ascii_case("person"))
            .map(|e| e.text.clone())
            .collect();

        let orgs: Vec<String> = entities
            .iter()
            .filter(|e| e.entity_type.eq_ignore_ascii_case("organization"))
            .map(|e| e.text.clone())
            .collect();

        println!("▶ PersonRelationAgent: persons={}, orgs={}", persons.len(), orgs.len());
        if persons.is_empty() || orgs.is_empty() {
            println!("▶ PersonRelationAgent: no persons or organizations");
            return Ok(());
        }

        println!("▶ PersonRelationAgent: reading {}", text_path.display());
        let document_text = fs::read_to_string(&text_path)
            .await
            .context("failed to read text.txt")?;
        let preview: String = document_text.chars().take(300).collect();
        println!("▶ PersonRelationAgent: text preview: {}", preview.replace('\n', " "));

        // Wichtig: Text in den Prompt
        let prompt = format!(
            r#"You are analyzing a single document.

The document is a source, not the truth.
Do not assume roles.
Do not decide facts.
Only propose relations that are explicitly stated in the document text.
If there is no explicit statement linking a person to an organization, return an empty list.

Document text:
---
{}
---

Persons found (extracted):
{}

Organizations found (extracted):
{}

Task:
- Identify which persons are explicitly related to which organizations in THIS document
- Do not invent relations
- If unsure, return an empty list

Return JSON only (no markdown):
[
  {{
    "person": "<name>",
    "organization": "<name>",
    "confidence": 0.0-1.0
  }}
]
"#,
            document_text,
            serde_json::to_string(&persons)?,
            serde_json::to_string(&orgs)?,
        );

        let request = ChatRequest {
            model: "gpt-4.1-mini".into(),
            messages: vec![ChatMessage { role: "user".into(), content: prompt }],
            temperature: 0.0,
        };

        println!(
            "▶ PersonRelationAgent: sending OpenAI request persons={}, orgs={}",
            persons.len(),
            orgs.len()
        );

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("via/0.1 person-relation")
            .build()
            .context("failed to build reqwest client")?;

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.openai_api_key)
            .json(&request)
            .send()
            .await
            .context("failed to send OpenAI request")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI error: {}", body);
        }

        let response: ChatResponse = response.json().await.context("invalid OpenAI response")?;
        let raw_content = response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "[]".into());

        let cleaned = strip_json_fences(&raw_content);

        println!("▶ PersonRelationAgent: raw OpenAI content:\n{}", raw_content);
        println!("▶ PersonRelationAgent: cleaned OpenAI JSON:\n{}", cleaned);

        let candidates: Vec<RelationCandidate> = match serde_json::from_str(cleaned) {
            Ok(v) => v,
            Err(err) => {
                println!("▶ PersonRelationAgent: failed to parse candidates JSON: {}", err);
                Vec::new()
            }
        };

        if candidates.is_empty() {
            println!("▶ PersonRelationAgent: no relation candidates");
            return Ok(());
        }

        let mut store = EvidenceStore::load_or_create(&record_dir).await?;

        for c in candidates {
            if c.confidence <= 0.0 {
                continue;
            }

            let key = format!(
                "person:{}|associated_with|{}",
                c.person.to_lowercase(),
                c.organization.to_lowercase()
            );

            let mut record = EvidenceRecord::new(
                key,
                "person".into(),
                c.person.clone(),
                "associated_with".into(),
                c.organization.clone(),
            );

            record.extra.insert("role_candidate".into(), Value::Bool(false));

            store.add_or_update(record, &document_id, c.confidence, self.name());
        }

        store.save().await?;
        println!("▶ PersonRelationAgent finished");

        Ok(())
    }
}
