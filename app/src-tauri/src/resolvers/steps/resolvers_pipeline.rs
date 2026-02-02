// src-tauri/src/pipeline/steps/resolvers_pipeline.rs
use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::pipeline::context::RecordContext;
use crate::pipeline::pipeline::PipelineStep;

use crate::resolvers::context::ResolverContext;
use crate::resolvers::resolver::{ResolverRunner, Resolver};

pub struct ResolversPipeline {
    pub resolvers: Vec<Box<dyn Resolver>>,
}

#[async_trait]
impl PipelineStep for ResolversPipeline {
    fn name(&self) -> &'static str {
        "resolvers"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl ResolversPipeline {
    async fn run_inner(&self, ctx: &RecordContext) -> Result<()> {
        let data_root = ctx
            .base_dir
            .parent()
            .and_then(|p| p.parent())
            .context("base_dir is not data/YYYY/MM")?
            .to_path_buf();

        let rctx = ResolverContext::new(data_root);
        ResolverRunner::run_all(&rctx, &self.resolvers).await?;
        Ok(())
    }
}
