use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
impl EvidenceRecord {
    pub fn merge_full(&mut self, incoming: EvidenceRecord) {
        self.occurrences += incoming.occurrences;

        for d in incoming.documents {
            if !self.documents.contains(&d) {
                self.documents.push(d);
            }
        }

        self.confidences.extend(incoming.confidences);

        for a in incoming.source_agents {
            if !self.source_agents.contains(&a) {
                self.source_agents.push(a);
            }
        }

        self.last_seen = incoming.last_seen;
    }
}
impl EvidenceRecord {
    pub fn from_user_confirmation(k: &KnowledgeRecord) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            key: format!(
                "confirm:{}:{}|{}|{}",
                k.subject_type.to_lowercase(),
                k.subject_value.to_lowercase(),
                k.predicate,
                k.object_value.to_lowercase()
            ),

            subject_type: k.subject_type.clone(),
            subject_value: k.subject_value.clone(),
            predicate: k.predicate.clone(),
            object_value: k.object_value.clone(),

            occurrences: 1,
            documents: k.source_documents.clone(),
            confidences: vec![1.0],
            source_agents: vec!["USER_CONFIRMATION".to_string()],

            extra: {
                let mut m = serde_json::Map::new();
                m.insert(
                    "confirmed_knowledge_id".to_string(),
                    serde_json::Value::String(k.id.clone()),
                );
                m.insert(
                    "action".to_string(),
                    serde_json::Value::String("confirm_relation".to_string()),
                );
                m
            },

            first_seen: now.clone(),
            last_seen: now,
        }
    }
}

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use crate::store::knowledge::KnowledgeRecord;

#[derive(Debug, Default)]
pub struct EvidenceStore {
    records: HashMap<String, EvidenceRecord>,
    pub(crate) path: PathBuf,
}

impl EvidenceStore {
    pub fn merge_record(&mut self, incoming: &EvidenceRecord) {
        let entry = self
            .records
            .entry(incoming.key.clone())
            .or_insert_with(|| incoming.clone());

        for doc in &incoming.documents {
            if !entry.documents.contains(doc) {
                entry.documents.push(doc.clone());
                entry.occurrences += 1;
            }
        }

        for c in &incoming.confidences {
            entry.confidences.push(*c);
        }
        for agent in &incoming.source_agents {
            if !entry.source_agents.contains(agent) {
                entry.source_agents.push(agent.clone());
            }
        }

        for (k, v) in &incoming.extra {
            entry.extra.entry(k.clone()).or_insert(v.clone());
        }
        if incoming.first_seen < entry.first_seen {
            entry.first_seen = incoming.first_seen.clone();
        }

        if incoming.last_seen > entry.last_seen {
            entry.last_seen = incoming.last_seen.clone();
        }
    }
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

        let records = list.into_iter().map(|r| (r.key.clone(), r)).collect();

        Ok(Self { records, path })
    }
    pub fn insert_or_merge(&mut self, record: EvidenceRecord) {
        match self.records.get_mut(&record.key) {
            Some(existing) => {
                existing.merge_full(record);
            }
            None => {
                self.records.insert(record.key.clone(), record);
            }
        }
    }
    pub fn add_or_update(
        &mut self,
        record: EvidenceRecord,
        document_id: &str,
        confidence: f32,
        agent: &str,
    ) {
        let entry = self
            .records
            .entry(record.key.clone())
            .or_insert_with(|| record);

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

    pub fn add_occurrence(&mut self, document_id: &str, confidence: f32, agent: &str) {
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
