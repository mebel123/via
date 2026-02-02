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
pub mod resolvers;
pub mod state;
use crate::agents::agent::Agent;
use crate::agents::context_relation::ContextRelationAgent;
use crate::agents::person_relation::PersonRelationAgent;
use crate::commands::sessions::list_sessions;
use crate::pipeline::knowledge_builder::KnowledgeBuilder;
use crate::pipeline::signals::SignalsPipeline;
use crate::resolvers::context::ResolverContext;
use crate::resolvers::resolver::ResolverRunner;
use crate::resolvers::OrgIdentityResolver;
use crate::state::AppState;
use commands::knowledge_graph::get_knowledge_graph;
use commands::knowledge_overview::get_knowledge_overview;
use commands::recorder::recorder_status;
use commands::todo_confirm::confirm_todo;
use commands::todo_ignore::ignore_todo;
use commands::todos::list_todos;
use pipeline::evidences::EvidencesPipeline;
use pipeline::{
    context::RecordContext,
    entities::EntityExtractionStep,
    pipeline::Pipeline,
    transcription::TranscriptionStep,
};

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
    process_document(&ctx).await?;
    update_global_state(&ctx).await?;
    Ok(())
}
async fn update_global_state(ctx: &RecordContext) -> Result<(), String> {
    Pipeline::new()
        .add_step(SignalsPipeline)
        .run(ctx)
        .await?;

    Pipeline::new()
        .add_step(EvidencesPipeline)
        .run(ctx)
        .await?;

    Pipeline::new()
        .add_step(KnowledgeBuilder)
        .run(ctx)
        .await?;


    let data_root = ctx
        .base_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("invalid data root")?
        .to_path_buf();

    let resolver_ctx = ResolverContext { data_root };

    ResolverRunner::run_all(
        &resolver_ctx,
        &[
            Box::new(OrgIdentityResolver),
            // später:
            // Box::new(PersonIdentityResolver),
        ],
    )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
async fn process_document(ctx: &RecordContext) -> Result<(), String> {
    let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    Pipeline::new()
        .add_step(TranscriptionStep {
            openai_api_key: openai_api_key.clone(),
        })
        .add_step(EntityExtractionStep {
            openai_api_key: openai_api_key.clone(),
        })
        .run(ctx)
        .await?;

    PersonRelationAgent {
        openai_api_key: openai_api_key.clone(),
    }
        .run_document(ctx)
        .await
        .map_err(|e| e.to_string())?;

    ContextRelationAgent {
        openai_api_key,
    }
        .run_document(ctx)
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
            recorder_status,
            list_sessions,
            start_recording,
            stop_recording,
            list_todos,
            confirm_todo,
            ignore_todo,
            get_knowledge_overview,
            get_knowledge_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
