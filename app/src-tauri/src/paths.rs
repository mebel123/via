use std::fs;
use std::path::PathBuf;
use chrono::Local;
use tauri::{AppHandle, Manager};

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