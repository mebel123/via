use std::path::PathBuf;
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn stop_recording(state: State<'_, AppState>) -> Result<PathBuf, String> {
    let mut recorder = state.recorder.lock().unwrap();
    recorder.stop()
}
