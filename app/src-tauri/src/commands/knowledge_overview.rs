use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::pipeline::context::RecordContext;
use crate::store::knowledge::KnowledgeStore;

#[derive(Serialize)]
pub struct KnowledgeOverview {
    pub persons: Vec<String>,
    pub organizations: Vec<String>,
    pub events: Vec<String>,
    pub relations: Vec<KnowledgeRelation>,
}

#[derive(Serialize)]
pub struct KnowledgeRelation {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f32,
}
#[tauri::command]
pub async fn get_knowledge_overview(app: AppHandle) -> Result<KnowledgeOverview, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?
        .join("data");

    println!("▶ get_knowledge_overview");

    let knowledge = KnowledgeStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    use std::collections::HashSet;

    let mut persons = HashSet::new();
    let mut organizations = HashSet::new();
    let mut events = HashSet::new();
    let mut relations = Vec::new();

    for record in knowledge.all() {
        if record.status == "deprecated" {
            continue;
        }
        if record.status != "approved" {
            continue;
        }
        if record.confidence < 1.0 {
            continue;
        }

        if record.status != "candidate" && record.status != "approved" {
            continue;
        }

        // subject sammeln
        match record.subject_type.as_str() {
            "person" => {
                persons.insert(record.subject_value.clone());
            }
            "organization" => {
                organizations.insert(record.subject_value.clone());
            }
            "event" => {
                events.insert(record.subject_value.clone());
            }
            _ => {}
        }

        // object kann auch entity sein
        match record.object_value.as_str() {
            v if record.predicate == "associated_with" => {
                // heuristisch: Organisationen landen hier oft
                organizations.insert(v.to_string());
            }
            _ => {}
        }

        relations.push(KnowledgeRelation {
            subject: record.subject_value.clone(),
            predicate: record.predicate.clone(),
            object: record.object_value.clone(),
            confidence: record.confidence,
        });
    }

    let mut persons: Vec<String> = persons.into_iter().collect();
    let mut organizations: Vec<String> = organizations.into_iter().collect();
    let mut events: Vec<String> = events.into_iter().collect();

    persons.sort();
    organizations.sort();
    events.sort();

    Ok(KnowledgeOverview {
        persons,
        organizations,
        events,
        relations,
    })
}
