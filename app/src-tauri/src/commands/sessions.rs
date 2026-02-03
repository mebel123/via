use serde::Serialize;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;
use tokio::fs;
use crate::processing::processing::ProcessingFile;
#[derive(Serialize)]
pub struct SessionEntity {
    pub entity_type: String,
    pub value: String,
}
#[derive(Serialize)]
pub struct SessionSummary {
    pub id: String,
    pub date: String,
    pub title: String,
    pub raw: String,
    pub entities: Vec<SessionEntity>,
}

#[tauri::command]
pub async fn list_sessions(app: AppHandle) -> Result<Vec<SessionSummary>, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?;

    println!("▶ list_sessions scanning {}", data_root.display());

    let mut sessions = Vec::new();

    for entry in WalkDir::new(&data_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let record_dir = entry.path();

        let entities_path = record_dir.join("entities.json");
        let processing_path = record_dir.join("processing.json");
        let text_path = record_dir.join("text.txt");

        if !processing_path.exists() || !text_path.exists() || !entities_path.exists() {
            continue;
        }




        let processing =
            ProcessingFile::load(&record_dir)
                .await
                .map_err(|e| e.to_string())?;


        let date = processing.started_at;
        let id = processing.doc_id;
        
        let raw = fs::read_to_string(&text_path)
            .await
            .map_err(|e| e.to_string())?;

        let entities_raw = fs::read_to_string(&entities_path)
            .await
            .map_err(|e| e.to_string())?;

        let entities_json: serde_json::Value =
            serde_json::from_str(&entities_raw).map_err(|e| e.to_string())?;
        
        use std::collections::HashSet;

        let mut entities_set: HashSet<(String, String)> = HashSet::new();

        if let Some(items) = entities_json.get("entities").and_then(|v| v.as_array()) {
            for item in items {
                let entity_type = item.get("type").and_then(|v| v.as_str());
                let text = item.get("text").and_then(|v| v.as_str());

                if let (Some(t), Some(value)) = (entity_type, text) {
                    entities_set.insert((t.to_string(), value.to_string()));
                }
            }
        }

        let entities = entities_set
            .into_iter()
            .map(|(entity_type, value)| SessionEntity {
                entity_type,
                value,
            })
            .collect::<Vec<_>>();
 


        let title = raw
            .lines()
            .next()
            .unwrap_or("Session")
            .chars()
            .take(80)
            .collect::<String>();

        sessions.push(SessionSummary {
            id,
            date,
            title,
            raw,
            entities,
        });
    }

    sessions.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(sessions)
}
