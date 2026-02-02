use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::resolvers::context::ResolverContext;
use crate::store::evidence::EvidenceStore;
use crate::store::knowledge::KnowledgeStore;

#[async_trait]
pub trait Resolver: Send + Sync {
    fn name(&self) -> &'static str;

    /// Run one resolver pass and persist changes.
    async fn run(&self, ctx: &ResolverContext) -> Result<()>;
}

/// Runs multiple resolvers sequentially.
pub struct ResolverRunner;

impl ResolverRunner {
    pub async fn run_all(
        ctx: &ResolverContext,
        resolvers: &[Box<dyn Resolver>],
    ) -> Result<()> {
        for r in resolvers {
            println!("▶ ResolverRunner: running {}", r.name());
            r.run(ctx)
                .await
                .with_context(|| format!("resolver failed: {}", r.name()))?;
        }
        Ok(())
    }
}

/// Shared IO helpers for resolvers.
/// Keep this minimal and boring.
pub struct ResolverIO;

impl ResolverIO {
    /// Loads the global aggregated evidence at {data_root}/evidence.json
    pub async fn load_root_evidence(ctx: &ResolverContext) -> Result<EvidenceStore> {
        EvidenceStore::load_or_create(ctx.data_root.as_path())
            .await
            .context("failed to load root evidence store")
    }

    /// Loads global knowledge at {data_root}/knowledge.json
    pub async fn load_knowledge(ctx: &ResolverContext) -> Result<KnowledgeStore> {
        KnowledgeStore::load_or_create(ctx.data_root.as_path())
            .await
            .context("failed to load knowledge store")
    }
}
