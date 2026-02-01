use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RecordContext {
    pub base_dir: PathBuf,
    pub audio_file: PathBuf,
}
