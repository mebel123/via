use serde::{Serialize, Deserialize};
use chrono::Utc;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub key: String,

    pub subject_type: String,
    pub subject_value: String,
    pub predicate: String,
    pub object_value: String,

    pub occurrences: usize,
    pub documents: Vec<String>,
    pub confidences: Vec<f32>,
    pub source_agents: Vec<String>,

    pub extra: serde_json::Map<String, Value>,

    pub first_seen: String,
    pub last_seen: String,
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;

#[derive(Debug, Default)]
pub struct EvidenceStore {
    records: HashMap<String, EvidenceRecord>,
    path: PathBuf,
}

impl EvidenceStore {
    pub async fn load_or_create(record_dir: &Path) -> Result<Self> {
        let path = record_dir.join("evidence.json");

        if !path.exists() {
            return Ok(Self {
                records: HashMap::new(),
                path,
            });
        }

        let raw = fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;

        let list: Vec<EvidenceRecord> =
            serde_json::from_str(&raw).context("invalid evidence.json")?;

        let records = list
            .into_iter()
            .map(|r| (r.key.clone(), r))
            .collect();

        Ok(Self { records, path })
    }

    pub fn add_or_update(
        &mut self,
        record: EvidenceRecord,
        document_id: &str,
        confidence: f32,
        agent: &str,
    ) {
        let entry = self.records.entry(record.key.clone()).or_insert_with(|| record);

        entry.add_occurrence(&document_id, confidence, agent);
    }

    pub async fn save(&self) -> Result<()> {
        let list: Vec<&EvidenceRecord> = self.records.values().collect();

        let json = serde_json::to_string_pretty(&list)?;
        fs::write(&self.path, json)
            .await
            .with_context(|| format!("failed to write {}", self.path.display()))?;

        Ok(())
    }

    pub fn all(&self) -> Vec<&EvidenceRecord> {
        self.records.values().collect()
    }
}

impl EvidenceRecord {
    pub fn new(
        key: String,
        subject_type: String,
        subject_value: String,
        predicate: String,
        object_value: String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            key,
            subject_type,
            subject_value,
            predicate,
            object_value,
            occurrences: 0,
            documents: Vec::new(),
            confidences: Vec::new(),
            source_agents: Vec::new(),
            extra: serde_json::Map::new(),
            first_seen: now.clone(),
            last_seen: now,
        }
    }

    pub fn add_occurrence(
        &mut self,
        document_id: &str,
        confidence: f32,
        agent: &str,
    ) {
        if !self.documents.contains(&document_id.to_string()) {
            self.documents.push(document_id.to_string());
            self.occurrences += 1;
        }

        self.confidences.push(confidence);

        if !self.source_agents.contains(&agent.to_string()) {
            self.source_agents.push(agent.to_string());
        }

        self.last_seen = Utc::now().to_rfc3339();
    }

    pub fn avg_confidence(&self) -> f32 {
        if self.confidences.is_empty() {
            0.0
        } else {
            self.confidences.iter().sum::<f32>() / self.confidences.len() as f32
        }
    }
}
