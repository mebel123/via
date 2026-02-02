use anyhow::{Context, Result};

use crate::pipeline::context::RecordContext;
use crate::pipeline::pipeline::PipelineStep;
use crate::store::evidence::EvidenceStore;
use crate::store::knowledge::{KnowledgeRecord, KnowledgeStore};

pub struct KnowledgeBuilder;

#[async_trait::async_trait]
impl PipelineStep for KnowledgeBuilder {
    fn name(&self) -> &'static str {
        "knowledge-builder"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl KnowledgeBuilder {
    async fn run_inner(&self, ctx: &RecordContext) -> Result<()> {
        // data/YYYY/MM → data
        let data_root = ctx
            .base_dir
            .parent()
            .and_then(|p| p.parent())
            .context("base_dir is not data/YYYY/MM")?
            .to_path_buf();

        println!("▶ KnowledgeBuilder scanning {}", data_root.display());

        let evidence_store = EvidenceStore::load_or_create(&data_root).await?;
        let mut knowledge = KnowledgeStore::load_or_create(&data_root).await?;

        let mut created = 0;
        let mut updated = 0;

        for ev in evidence_store.all() {
            let id = Self::knowledge_id(ev);

            let record = KnowledgeRecord::new(
                id.clone(),
                ev.subject_type.clone(),
                ev.subject_value.clone(),
                ev.predicate.clone(),
                ev.object_value.clone(),
                ev.avg_confidence(),
                "KNOWLEDGE_BUILDER",
                ev.documents.clone(),
            );

            if knowledge.get_mut(&id).is_some() {
                updated += 1;
            } else {
                created += 1;
            }

            knowledge.append_or_update(record);
        }

        knowledge.save().await?;

        println!(
            "▶ KnowledgeBuilder finished (created={}, updated={})",
            created, updated
        );

        Ok(())
    }

    fn knowledge_id(ev: &crate::store::evidence::EvidenceRecord) -> String {
        format!(
            "{}:{}|{}:{}",
            ev.subject_type.to_lowercase(),
            ev.subject_value.to_lowercase(),
            ev.predicate,
            ev.object_value.to_lowercase()
        )
    }
}
