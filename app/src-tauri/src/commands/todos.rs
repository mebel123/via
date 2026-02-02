// src-tauri/src/commands/todos.rs
use crate::store::knowledge::KnowledgeStore;
use serde::Serialize;
use tauri::{AppHandle, Manager};


#[derive(Serialize)]
pub struct TodoItem {
    pub id: String,

    pub kind: String,
    pub status: String,
    pub date: String,
    pub title: String,

    pub target_type: String,
    pub target_id: String,

    pub confidence: f32,

    // NEU
    pub source_document: Option<String>, // record0006 etc
}
#[tauri::command]
pub async fn list_todos(app: AppHandle) -> Result<Vec<TodoItem>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?
        .join("data");

    println!("▶ list_todos scanning {}", data_root.display());

    let knowledge = KnowledgeStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    let mut todos = Vec::new();

    for record in knowledge.all() {
        if record.status != "candidate" {
            continue;
        }
        if record.approved_by != "none" {
            continue;
        }

        let date = record
            .updated_at
            .clone()
            .unwrap_or_else(|| record.created_at.clone());

        let title = format!(
            "{} {} {}",
            record.subject_value,
            record.predicate,
            record.object_value
        );
        let source_document = record
            .source_documents
            .get(0)
            .cloned();
        todos.push(TodoItem {
            id: record.id.clone(),

            kind: "confirm_relation".to_string(),
            status: "open".to_string(),

            date,
            title,

            target_type: "knowledge_record".to_string(),
            target_id: record.id.clone(),

            confidence: record.confidence,
            source_document: source_document
        });
    }
    println!("▶ list_todos done count {} knowledge count {}", todos.len(), knowledge.all().len());
    // neueste zuerst
    todos.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(todos)
}
