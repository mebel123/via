use std::path::PathBuf;
use std::sync::Arc;
use crate::processing::progress::ProgressEmitter;

#[derive(Clone)]
pub struct RecordContext {
    pub base_dir: PathBuf,
    pub audio_file: PathBuf,
    pub progress: Option<Arc<dyn ProgressEmitter>>,
}

impl RecordContext {
    pub fn emit(&self, stage: &str, message: &str, percent: u8) {
        if let Some(p) = &self.progress {
            p.emit(stage, message, percent);
        }
    }
}
