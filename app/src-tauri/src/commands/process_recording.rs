use crate::pipeline::context::RecordContext;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use crate::processing::document::process_document;
use crate::processing::progress::{emit_progress, TauriProgressEmitter};
use crate::processing::state_global::update_global_state;
#[tauri::command]
pub async fn process_recording(app: AppHandle, audio_path: PathBuf) -> Result<(), String> {
    let base_dir = audio_path
        .parent()
        .ok_or("audio file has no parent directory")?
        .to_path_buf();
    let progress = TauriProgressEmitter::new(app.clone());

    let ctx = RecordContext {
        base_dir: base_dir.clone(),
        audio_file: audio_path,
        progress: Some(Arc::new(progress)),
    };
    emit_progress(&app, "start", "Verarbeitung gestartet", 5);

    process_document(&ctx).await?;
    update_global_state(&ctx).await?;
    emit_progress(&app, "done", "Fertig", 100);
    Ok(())
}
