use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn recorder_status(state: State<'_, AppState>) -> bool {
    let recorder = state.recorder.lock().unwrap();
    recorder.is_recording()
}
