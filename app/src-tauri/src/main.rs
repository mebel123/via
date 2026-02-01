mod recording;
mod paths;

use recording::Recording;
use std::sync::Mutex;

struct AppState {
    recorder: Mutex<Recording>,
}
use tauri::{AppHandle, State};

#[tauri::command]
fn start_recording(
    app: AppHandle,
    state: State<AppState>
) -> Result<(), String> {
    let mut recorder = state.recorder.lock().unwrap();
    recorder.start(&app)
}

#[tauri::command]
fn stop_recording(
    state: State<AppState>
) -> Result<(), String> {
    let mut recorder = state.recorder.lock().unwrap();
    recorder.stop()
}
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            recorder: Mutex::new(Recording::new()),
        })
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}