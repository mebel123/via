use super::context::RecordContext;
use super::pipeline::PipelineStep;
use serde::Deserialize;
use reqwest::{Client, multipart::{Form, Part}};
use anyhow::Context;
use crate::paths::record_dir_from_audio;
#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

pub struct TranscriptionStep {
    pub openai_api_key: String,
}

#[async_trait::async_trait]
impl PipelineStep for TranscriptionStep {
    fn name(&self) -> &'static str {
        "transcription"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl TranscriptionStep {
    async fn run_inner(&self, ctx: &RecordContext) -> anyhow::Result<()> {
        println!("▶ TranscriptionStep using  file{}", &ctx.audio_file.to_string_lossy());

        if self.openai_api_key.is_empty() {
            println!("▶ TranscriptionStep OPENAI_API_KEY is missing");
            anyhow::bail!("OPENAI_API_KEY is missing");
        }
        let record_dir = record_dir_from_audio(&ctx.base_dir, &ctx.audio_file).await?;
        let output_file = record_dir.join("text.txt");

        let file_content = tokio::fs::read(&ctx.audio_file)
            .await
            .context("failed to read audio file")?;

        let part = Part::bytes(file_content)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;

        let form = Form::new()
            .part("file", part)
            .text("language", "de")
            .text("model", "whisper-1");

        let client = Client::new();

        let response = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.openai_api_key)
            .multipart(form)
            .send()
            .await
            .context("failed to send whisper request")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI error: {}", body);
        }

        let whisper: WhisperResponse = response
            .json()
            .await
            .context("invalid whisper response")?;

        tokio::fs::write(output_file, whisper.text)
            .await
            .context("failed to write transcription file")?;

        Ok(())
    }
}
