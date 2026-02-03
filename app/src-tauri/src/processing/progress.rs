use tauri::{AppHandle, Emitter};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ProgressEvent {
    pub stage: String,
    pub message: String,
    pub percent: u8,
}
pub trait ProgressEmitter: Send + Sync {
    fn emit(&self, stage: &str, message: &str, percent: u8);
}
pub fn emit_progress(
    app: &AppHandle,
    stage: &str,
    message: &str,
    percent: u8,
) {
    let _ = app.emit(
        "processing:progress",
        ProgressEvent {
            stage: stage.into(),
            message: message.into(),
            percent,
        },
    );
} 

pub struct TauriProgressEmitter {
    app: AppHandle,
}

impl TauriProgressEmitter {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ProgressEmitter for TauriProgressEmitter {
    fn emit(&self, stage: &str, message: &str, percent: u8) {
        let _ = self.app.emit(
            "processing:progress",
            ProgressEvent {
                stage: stage.into(),
                message: message.into(),
                percent,
            },
        );
    }
}
