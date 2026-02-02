use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result};
use chrono::Utc;
use tokio::fs;
use uuid::Uuid;
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingFile {
    pub doc_id: String,
    pub audio_file: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub steps: HashMap<String, crate::processing::model::ProcessingStep>,
    pub errors: Vec<String>,
}
impl ProcessingFile {
    pub async fn load(
        record_dir: &Path,
    ) -> Result<Self> {
        let path = record_dir.join("processing.json");

        if path.exists() {
            let content = fs::read_to_string(&path).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            return Err(anyhow::anyhow!("Processing file not found"));
        }
    }
    pub async fn load_or_create(
        record_dir: &Path,
        audio_file: &Path,
    ) -> Result<Self> {
        let path = record_dir.join("processing.json");

        if path.exists() {
            let content = fs::read_to_string(&path).await?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self {
                doc_id: Uuid::new_v4().to_string(),
                audio_file: audio_file.file_name().unwrap().to_string_lossy().to_string(),
                started_at: Utc::now().to_rfc3339(),
                finished_at: None,
                steps: HashMap::new(),
                errors: vec![],
            })
        }
    }

    pub async fn save(&self, record_dir: &Path) -> Result<()> {
        let path = record_dir.join("processing.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json).await?;
        Ok(())
    }
}
