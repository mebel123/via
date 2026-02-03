use std::sync::Arc;
use chrono::Utc;
use tauri::{AppHandle, Manager};
use crate::store::evidence::{EvidenceRecord, EvidenceStore};
use crate::store::knowledge::KnowledgeStore;
use crate::pipeline::knowledge_builder::KnowledgeBuilder;
use crate::pipeline::pipeline::Pipeline;
use crate::pipeline::context::RecordContext;
use crate::processing::progress::TauriProgressEmitter;

#[tauri::command]
pub async fn confirm_todo(
    app: AppHandle,
    knowledge_id: String
) -> Result<(), String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?
        .join("data");
    println!(
        "▶ todo confirm_todo {}",
        knowledge_id
    );

    let knowledge = KnowledgeStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    let record = knowledge
        .get_by_id(&knowledge_id)
        .ok_or("knowledge record not found")?
        .clone();
    println!(
        "▶ todo record with id {}",
        record.id
    );

    let mut evidence = EvidenceStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    let ev = EvidenceRecord::from_user_confirmation(&record);
    evidence.insert_or_merge(ev);
    evidence.save().await.map_err(|e| e.to_string())?;
    let mut knowledge = KnowledgeStore::load_or_create(&data_root).await.map_err(|e| e.to_string())?;

    if let Some(r) = knowledge.get_mut(&knowledge_id) {
        r.status = "approved".into();
        r.approved_by = "user".into();
        r.updated_at = Some(Utc::now().to_rfc3339());
    }
    let progress = TauriProgressEmitter::new(app.clone());

    knowledge.save().await.map_err(|e| e.to_string())?;
    let ctx = RecordContext {
        base_dir: data_root.clone(),
        audio_file: data_root.clone(),
        progress: Some(Arc::new(progress)),
    };

    Pipeline::new()
        .add_step(KnowledgeBuilder)
        .run(&ctx)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
