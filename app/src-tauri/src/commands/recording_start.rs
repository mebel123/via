use tauri::{AppHandle, State};
use crate::state::AppState;

#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    state: State<AppState>,
) -> Result<(), String> {

    let mut recorder = state.recorder.lock().unwrap();
    recorder.start(&app)?;
    Ok(())
}
