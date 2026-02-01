mod recording;
mod paths;
mod pipeline;
use recording::Recording;
use std::sync::Mutex;
use tauri::{AppHandle, State};
pub mod agents;
pub mod store;
pub mod processing;
pub mod commands;
use pipeline::{
    context::RecordContext,
    pipeline::Pipeline,
    transcription::TranscriptionStep,
    entities::EntityExtractionStep,
};
use crate::agents::context_relation::ContextRelationAgent;
use crate::pipeline::signals::SignalsPipeline;
use crate::agents::person_relation::PersonRelationAgent;
use crate::agents::agent::Agent;
use crate::commands::sessions::list_sessions;

struct AppState {
    recorder: Mutex<Recording>,
}

#[tauri::command]
fn start_recording(
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {

    let mut recorder = state.recorder.lock().unwrap();
    recorder.start(&app)?;
    Ok(())
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    let audio_path = {
        let mut recorder = state.recorder.lock().unwrap();
        recorder.stop()?
    };

    let base_dir = audio_path
        .parent()
        .ok_or("audio file has no parent directory")?
        .to_path_buf();

    let ctx = RecordContext {
        base_dir: base_dir.clone(),
        audio_file: audio_path.clone(),
    };

    let pipeline = Pipeline::new()
        .add_step(TranscriptionStep {
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
        })
        .add_step(EntityExtractionStep {
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
        });

    pipeline.run(&ctx).await?;

    let signals_pipeline = SignalsPipeline;

    Pipeline::new()
        .add_step(signals_pipeline)
        .run(&ctx)
        .await?;


    let agent = PersonRelationAgent {
        openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
    };
    agent
        .run_document(&ctx)
        .await
        .map_err(|e| e.to_string())?;

    let agent = ContextRelationAgent {
        openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
    };
    agent
        .run_document(&ctx)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            recorder: Mutex::new(Recording::new())
        })
        .invoke_handler(tauri::generate_handler![
            list_sessions,
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
