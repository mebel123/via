use std::collections::HashMap;
use super::context::RecordContext;
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use crate::paths::record_dir_from_audio;
use crate::agents::agent::Agent;
use crate::processing::model::ProcessingStep;
use crate::processing::processing::ProcessingFile;

#[async_trait]
pub trait PipelineStep: Send + Sync {
    fn name(&self) -> &'static str;
    async fn run(&self, ctx: &RecordContext) -> Result<(), String>;
}


pub struct Pipeline {
    steps: Vec<Box<dyn PipelineStep>>,
    agents: Vec<Box<dyn Agent>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            agents: Vec::new(),
        }
    }

    pub fn add_step(mut self, step: impl PipelineStep + 'static) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn add_agent(mut self, agent: impl Agent + 'static) -> Self {
        self.agents.push(Box::new(agent));
        self
    }

    pub async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        let record_dir = record_dir_from_audio(&ctx.base_dir, &ctx.audio_file)
            .await
            .map_err(|e| e.to_string())?;

        let mut processing =
            ProcessingFile::load_or_create(&record_dir, &ctx.audio_file)
                .await
                .map_err(|e| e.to_string())?;

        // ---------- deterministic pipeline steps ----------
        for step in &self.steps {
            let step_name = step.name().to_string();
            println!("▶ Pipeline step: {}", step_name);

            let result = step.run(ctx).await;

            let step_entry = ProcessingStep {
                status: if result.is_ok() { "done" } else { "error" }.into(),
                output: None,
                finished_at: Some(Utc::now().to_rfc3339()),
            };

            if let Err(err) = result {
                processing
                    .errors
                    .push(format!("{}: {}", step_name, err));
            }

            processing.steps.insert(step_name, step_entry);
            processing
                .save(&record_dir)
                .await
                .map_err(|e| e.to_string())?;
        }

        // ---------- heuristic / AI agents ----------
        for agent in &self.agents {
            let agent_name = agent.name().to_string();
            println!("▶ Agent: {}", agent_name);

            let result = agent.run_document(ctx).await;

            let step_entry = ProcessingStep {
                status: if result.is_ok() { "done" } else { "error" }.into(),
                output: None,
                finished_at: Some(Utc::now().to_rfc3339()),
            };

            if let Err(err) = result {
                processing
                    .errors
                    .push(format!("{}: {}", agent_name, err));
            }

            processing.steps.insert(agent_name, step_entry);
            processing
                .save(&record_dir)
                .await
                .map_err(|e| e.to_string())?;
        }

        processing.finished_at = Some(Utc::now().to_rfc3339());
        processing
            .save(&record_dir)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
