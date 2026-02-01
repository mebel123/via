use anyhow::{Context, Result};
use std::path::Path;
pub async fn load_document_id(record_dir: &Path) -> Result<String> {
    let path = record_dir.join("processing.json");

    let raw = tokio::fs::read_to_string(&path)
        .await
        .context("failed to read processing.json")?;

    let v: serde_json::Value = serde_json::from_str(&raw)
        .context("invalid processing.json")?;

    let doc_id = v["doc_id"]
        .as_str()
        .context("processing.json missing doc_id")?;

    Ok(doc_id.to_string())
}
