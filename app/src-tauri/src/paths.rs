use std::fs;
use chrono::Local;
use tauri::{AppHandle, Manager};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

pub async fn record_dir_from_audio(base_dir: &Path, audio_file: &Path) -> Result<PathBuf> {
    let stem = audio_file
        .file_stem()
        .and_then(|s| s.to_str())
        .context("audio file has no valid file stem")?;

    let record_dir = base_dir.join(stem);

    tokio::fs::create_dir_all(&record_dir)
        .await
        .context("failed to create record directory")?;

    Ok(record_dir)
}
pub fn next_recording_path(app: &AppHandle) -> PathBuf {
    let now = Local::now();
    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();

    let mut base = app
        .path()
        .app_data_dir()
        .expect("app data dir not available");

    base.push("data");
    base.push(year);
    base.push(month);

    fs::create_dir_all(&base).expect("failed to create directories");

    let mut index = 1;
    loop {
        let filename = format!("record{:04}.wav", index);
        let candidate = base.join(&filename);
        if !candidate.exists() {
            return candidate;
        }
        index += 1;
    }
}
