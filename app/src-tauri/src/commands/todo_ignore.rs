use chrono::Utc;
use tauri::{AppHandle, Manager};
use crate::store::knowledge::KnowledgeStore;
use serde_json::Value;

#[tauri::command]
pub async fn ignore_todo(
    app: AppHandle,
    knowledge_id: String
) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?
        .join("data");

    println!("▶ todo ignore_todo {}", knowledge_id);

    let mut knowledge = KnowledgeStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    let record = knowledge
        .get_mut(&knowledge_id)
        .ok_or("knowledge record not found")?;

    record.status = "deprecated".into();
    record.approved_by = "user".into();
    record.updated_at = Some(Utc::now().to_rfc3339());

    record.extra.insert(
        "deprecated_reason".into(),
        Value::String("user_ignored".into())
    );

    knowledge.save().await.map_err(|e| e.to_string())?;

    Ok(())
}
