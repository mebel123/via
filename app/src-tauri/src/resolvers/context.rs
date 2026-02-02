// src-tauri/src/resolvers/context.rs
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ResolverContext {
    /// Root folder of the user data, example: .../app_data_dir
    pub data_root: PathBuf,
}

impl ResolverContext {
    pub fn new(data_root: PathBuf) -> Self {
        Self { data_root }
    }
}
