use anyhow::{Context, Result};
use tokio::fs;
use walkdir::WalkDir;

use crate::pipeline::context::RecordContext;
use crate::pipeline::pipeline::PipelineStep;
use crate::store::evidence::{EvidenceRecord, EvidenceStore};

pub struct EvidencesPipeline;

#[async_trait::async_trait]
impl PipelineStep for EvidencesPipeline {
    fn name(&self) -> &'static str {
        "evidences"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl EvidencesPipeline {
    async fn run_inner(&self, ctx: &RecordContext) -> Result<()> {
        let data_root = ctx
            .base_dir
            .parent()
            .and_then(|p| p.parent())
            .context("base_dir is not data/YYYY/MM")?
            .to_path_buf();

        println!("▶ EvidencesPipeline scanning {}", data_root.display());

        // globale evidence.json
        let mut root_store = EvidenceStore::load_or_create(&data_root).await?;

        let mut evidence_files_found = 0;

        for entry in WalkDir::new(&data_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_name() != "evidence.json" {
                continue;
            }

            // skip globale evidence.json, sonst Selbstaggregation
            if entry.path().parent() == Some(data_root.as_path()) {
                continue;
            }

            evidence_files_found += 1;

            let record_dir = entry
                .path()
                .parent()
                .context("evidence.json without parent dir")?;

            let store = EvidenceStore::load_or_create(record_dir).await?;

            println!(
                "▶ found evidence.json at {} with {} records",
                entry.path().display(),
                store.all().len()
            );

            for record in store.all() {
                // jede EvidenceRecord enthält ihre Dokumente bereits
                // wir fügen sie 1:1 zur globalen Evidence hinzu
                root_store.merge_record(record);
            }
        }

        println!(
            "▶ EvidencesPipeline processed {} evidence.json files",
            evidence_files_found
        );

        root_store.save().await?;
        println!(
            "▶ EvidencesPipeline wrote {}",
            root_store.path.display()
        );

        Ok(())
    }
}