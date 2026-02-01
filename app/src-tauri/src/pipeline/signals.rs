use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Context, Result};
use tokio::fs;
use walkdir::WalkDir;

use crate::pipeline::context::RecordContext;
use crate::pipeline::pipeline::PipelineStep;

#[derive(Debug, Deserialize)]
struct EntitiesFile {
    entities: Vec<Entity>,
}
#[derive(Debug, Deserialize)]
struct Entity {
    text: String,
    #[serde(rename = "type")]
    entity_type: String,
}

#[derive(Debug, Serialize)]
struct Signal {
    key: String,
    #[serde(rename = "type")]
    signal_type: String,
    value: String,
    count: usize,
    documents: Vec<String>,
}

fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace('.', "")
        .replace(',', "")
}

pub struct SignalsPipeline;

#[async_trait::async_trait]
impl PipelineStep for SignalsPipeline {
    fn name(&self) -> &'static str {
        "signals"
    }

    async fn run(&self, ctx: &RecordContext) -> Result<(), String> {
        self.run_inner(ctx).await.map_err(|e| e.to_string())
    }
}

impl SignalsPipeline {
    async fn run_inner(&self, ctx: &RecordContext) -> Result<()> {
        let data_root = ctx
            .base_dir
            .parent()
            .and_then(|p| p.parent())
            .context("base_dir is not data/YYYY/MM")?
            .to_path_buf();
        println!("▶ SignalsPipeline scanning data root: {}", data_root.display());
        let mut signals: HashMap<String, Signal> = HashMap::new();
        let mut entities_files_found = 0;
        for entry in WalkDir::new(&data_root)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_name() != "entities.json" {
                continue;
            }
            entities_files_found += 1;


            println!("▶ found entities.json at {}", entry.path().display());

            let record_dir = entry
                .path()
                .parent()
                .context("entities.json without parent dir")?;

            let doc_id = record_dir
                .file_name()
                .context("record dir has no name")?
                .to_string_lossy()
                .to_string();

            let content = fs::read_to_string(entry.path())
                .await
                .with_context(|| format!("failed to read {}", entry.path().display()))?;


            let wrapper: EntitiesFile = match serde_json::from_str(&content) {
                Ok(w) => w,
                Err(err) => {
                    println!(
                        "⚠ invalid entities.json at {} → {}",
                        entry.path().display(),
                        err
                    );
                    continue;
                }
            };

            println!("▶ parsed {} entities", wrapper.entities.len());

            for entity in wrapper.entities {
                let normalized = normalize(&entity.text);
                let key = format!(
                    "{}:{}",
                    entity.entity_type.to_lowercase(),
                    normalized
                );

                let signal = signals.entry(key.clone()).or_insert(Signal {
                    key,
                    signal_type: entity.entity_type.to_lowercase(),
                    value: entity.text.clone(),
                    count: 0,
                    documents: Vec::new(),
                });

                signal.count += 1;

                if !signal.documents.contains(&doc_id) {
                    signal.documents.push(doc_id.clone());
                }
            }
            //let entities: Vec<Entity> = serde_json::from_str(&content)
            //  .with_context(|| format!("invalid JSON in {}", entry.path().display()))?;

        }
        println!("▶ total entities.json files found2: {}", entities_files_found);
        let mut all_signals: Vec<Signal> = signals.into_values().collect();
        all_signals.sort_by(|a, b| b.count.cmp(&a.count));

        let output_path = data_root.join("signals.json");
        println!("▶ wr {}", output_path.display());
        fs::write(
            &output_path,
            serde_json::to_string_pretty(&all_signals)?,
        )
            .await
            .context("failed to write signals.json")?;

        println!("▶ SignalsPipeline wrote {}", output_path.display());

        Ok(())
    }
}
