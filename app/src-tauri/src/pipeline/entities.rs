use super::context::RecordContext;
use super::pipeline::PipelineStep;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;
use anyhow::{Context, Result};
use reqwest::Client;
use crate::paths::record_dir_from_audio;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

pub struct EntityExtractionStep {
    pub openai_api_key: String,
}
fn strip_json_fences(s: &str) -> &str {
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
fn extract_entities_from_value(mut v: Value) -> Value {
    // Case 1: already { "entities": [...] }
    if let Value::Object(map) = &v {
        if map.contains_key("entities") {
            return v;
        }
    }

    // Case 2: raw array -> wrap
    if let Value::Array(arr) = v {
        return serde_json::json!({ "entities": arr });
    }

    // Case 3: structured object like { persons: [], organizations: [], ... }
    if let Value::Object(map) = v {
        let mut entities = Vec::new();

        for (key, value) in map {
            if let Value::Array(items) = value {
                for item in items {
                    if let Value::String(text) = item {
                        entities.push(serde_json::json!({
                            "type": key.trim_end_matches('s'),
                            "text": text
                        }));
                    }
                }
            }
        }

        return serde_json::json!({ "entities": entities });
    }

    // Fallback
    serde_json::json!({ "entities": [] })
}


#[async_trait::async_trait]
impl PipelineStep for EntityExtractionStep {
    fn name(&self) -> &'static str {
        "entity-extraction"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl EntityExtractionStep {
    async fn run_inner(&self, ctx: &RecordContext) -> Result<()> {
        println!("▶ base_dir        = {}", ctx.base_dir.display());
        println!("▶ audio_file      = {}", ctx.audio_file.display());
        if self.openai_api_key.is_empty() {
            println!("▶ EntityExtractionStep OPENAI_API_KEY is missing");
            anyhow::bail!("OPENAI_API_KEY is missing");
        }
        let record_dir = record_dir_from_audio(&ctx.base_dir, &ctx.audio_file).await?;
        let text_path = record_dir.join("text.txt");
        let entities_path = record_dir.join("entities.json");
        println!("▶ record_dir      = {}", record_dir.display());
        println!("▶ text_path       = {}", text_path.display());
        println!("▶ entities_path   = {}", entities_path.display());

        let document = fs::read_to_string(&text_path)
            .await
            .context("failed to read text.txt")?;

        let system_prompt =
            "You extract named entities from text.\n\
             The document is a source, not the truth.\n\
             Do not infer or assume.\n\
             Only extract what is explicitly present.\n\
             Return JSON only.";

        let user_prompt = format!(
            "Extract entities from the following document:\n\n{}",
            document
        );

        println!("▶ EntityExtractionStep {} {} export tp file {}", system_prompt, user_prompt, entities_path.display());

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: user_prompt,
            },
        ];

        let request = ChatRequest {
            model: "gpt-4.1-mini".into(),
            messages,
            temperature: 0.0,
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("via/0.1 entity-extraction")
            .build()
            .context("failed to build reqwest client")?;

        println!(
            "▶ EntityExtractionStep sending request: model={}, chars={}",
            request.model,
            document.len()
        );
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.openai_api_key)
            .json(&request)
            .send()
            .await
            .context("failed to send OpenAI request")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            println!("▶ EntityExtractionStep OpenAI request failed {}", body);
            anyhow::bail!("OpenAI error: {}", body);
        }

        let response: ChatResponse = response
            .json()
            .await
            .context("invalid OpenAI response")?;

        let raw_content = response
            .choices
            .get(0)
            .map(|c| c.message.content.clone())
            .unwrap_or_else(|| "[]".into());

        println!("▶ raw OpenAI content:\n{}", raw_content);

        let cleaned = strip_json_fences(&raw_content);

        println!("▶ cleaned OpenAI JSON:\n{}", cleaned);

        let raw_value: Value = serde_json::from_str(cleaned)
            .context("OpenAI did not return valid JSON")?;
        let mut entities = extract_entities_from_value(raw_value);

        if let Value::Object(map) = &mut entities {
            if let Some(Value::Array(items)) = map.get_mut("entities") {
                for item in items {
                    if let Value::Object(obj) = item {
                        obj.insert("source".into(), Value::String("openai".into()));
                        obj.insert("status".into(), Value::String("suggested".into()));
                    }
                }
            }
        }

        let pretty = serde_json::to_string_pretty(&entities)?;

        fs::write(&entities_path, pretty)
            .await
            .context("failed to write entities.json")?;

        let pretty = serde_json::to_string_pretty(&entities)?;

        fs::write(&entities_path, pretty)
            .await
            .context("failed to write entities.json")?;

        Ok(())
    }
}
