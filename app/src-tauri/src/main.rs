mod recording;
mod paths;
mod pipeline;

use recording::Recording;
use std::sync::Mutex;
pub mod agents;
pub mod store;
pub mod processing;
pub mod commands;
pub mod resolvers;
pub mod state;
use crate::agents::agent::Agent;
use crate::commands::sessions::list_sessions;
use crate::state::AppState;
use commands::knowledge_graph::get_knowledge_graph;
use commands::knowledge_overview::get_knowledge_overview;
use commands::recorder::recorder_status;
use commands::recording_start::start_recording;
use commands::recording_stop::stop_recording;
use commands::todo_confirm::confirm_todo;
use commands::todo_ignore::ignore_todo;
use commands::todos::list_todos;
use commands::process_recording::process_recording;


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
            process_recording,
            list_todos,
            confirm_todo,
            ignore_todo,
            get_knowledge_overview,
            get_knowledge_graph,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
