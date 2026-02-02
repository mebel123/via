use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrganizationCluster {
    pub cluster_id: String,
    pub normalized: String,
    pub variants: Vec<String>,
    pub confidence: f32,
    pub status: String, // candidate | approved | deprecated
    pub source_agent: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct KnowledgeClusters {
    pub organizations: Vec<OrganizationCluster>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeRecord {
    pub id: String,

    pub subject_type: String,
    pub subject_value: String,

    pub predicate: String,
    pub object_value: String,

    pub status: String, // candidate | approved | deprecated
    pub confidence: f32,
    pub approved_by: String, // none | user | agent

    pub source_documents: Vec<String>,
    pub source_agent: String,

    pub subject_identity_id: Option<String>,
    pub object_identity_id: Option<String>,

    pub created_at: String,
    pub updated_at: Option<String>,

    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Default)]
pub struct KnowledgeStore {
    records: HashMap<String, KnowledgeRecord>,
    clusters: KnowledgeClusters,
    path: PathBuf,
}
impl KnowledgeStore {
    pub fn clusters_mut(&mut self) -> &mut KnowledgeClusters {
        &mut self.clusters
    }

    pub fn clusters(&self) -> &KnowledgeClusters {
        &self.clusters
    }
    pub fn get_by_id(&self, id: &str) -> Option<&KnowledgeRecord> {
        self.records.get(id)
    }
    pub async fn load_or_create(data_root: &Path) -> Result<Self> {
        let path = data_root.join("knowledge.json");

        if !path.exists() {
            return Ok(Self {
                records: HashMap::new(),
                clusters: Default::default(),
                path,
            });
        }

        let raw = fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;

        let json: serde_json::Value =
            serde_json::from_str(&raw).context("invalid knowledge.json")?;
        let list = json
            .get("records")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut records = HashMap::new();
        for v in list {
            let r: KnowledgeRecord = serde_json::from_value(v)?;
            records.insert(r.id.clone(), r);
        }

        let clusters: KnowledgeClusters = match json.get("clusters") {
            Some(c) => serde_json::from_value(c.clone())?,
            None => KnowledgeClusters::default(),
        };

        Ok(Self {
            records,
            clusters,
            path,
        })
    }

    pub fn all(&self) -> Vec<&KnowledgeRecord> {
        self.records.values().collect()
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut KnowledgeRecord> {
        self.records.get_mut(id)
    }

    pub fn append_or_update(&mut self, record: KnowledgeRecord) {
        match self.records.get_mut(&record.id) {
            Some(existing) => {
                Self::merge(existing, record);
            }
            None => {
                self.records.insert(record.id.clone(), record);
            }
        }
    }

    fn merge(existing: &mut KnowledgeRecord, incoming: KnowledgeRecord) {
        // confidence: keep max for now
        existing.confidence = existing.confidence.max(incoming.confidence);

        // documents: union
        for d in incoming.source_documents {
            if !existing.source_documents.contains(&d) {
                existing.source_documents.push(d);
            }
        }

        // status escalation only
        if existing.status != "approved" {
            existing.status = incoming.status;
        }

        existing.updated_at = Some(Utc::now().to_rfc3339());

        // merge extra shallow
        for (k, v) in incoming.extra {
            existing.extra.entry(k).or_insert(v);
        }
    }
    pub async fn save(&self) -> Result<()> {
        let list: Vec<&KnowledgeRecord> = self.records.values().collect();

        let json = serde_json::json!({
            "records": list,
            "clusters": self.clusters,
            "name_mappings": []
        });

        let pretty = serde_json::to_string_pretty(&json)?;
        fs::write(&self.path, pretty)
            .await
            .with_context(|| format!("failed to write {}", self.path.display()))?;

        Ok(())
    }
}

impl KnowledgeRecord {
    pub fn new(
        id: String,
        subject_type: String,
        subject_value: String,
        predicate: String,
        object_value: String,
        confidence: f32,
        source_agent: &str,
        source_documents: Vec<String>,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            subject_type,
            subject_value,
            predicate,
            object_value,
            status: "candidate".into(),
            confidence,
            approved_by: "none".into(),
            source_documents,
            source_agent: source_agent.into(),
            subject_identity_id: None,
            object_identity_id: None,
            created_at: now,
            updated_at: None,
            extra: serde_json::Map::new(),
        }
    }
}
