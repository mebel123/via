use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::store::knowledge::KnowledgeStore;
fn node_id(typ: &str, value: &str) -> String {
    format!(
        "{}:{}",
        typ.to_lowercase(),
        value
            .to_lowercase()
            .replace(' ', "-")
            .replace(',', "")
            .replace('.', "")
    )
}
#[derive(Serialize)]
pub struct GraphMeta {
    pub generated_at: String,
}

#[derive(Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub predicate: String,
    pub confidence: f32,
}

#[derive(Serialize)]
pub struct GraphModel {
    pub meta: GraphMeta,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
fn infer_object_type(r: &crate::store::knowledge::KnowledgeRecord) -> String {
    if r.object_value.contains("GmbH")
        || r.object_value.contains("AG")
        || r.object_value.contains("Logic")
    {
        return "organization".into();
    }

    if r.subject_type == "event" {
        // Event → Zeit, Datum, Aktion
        return "event".into();
    }

    "entity".into()
}
#[tauri::command]
pub async fn get_knowledge_graph(app: AppHandle) -> Result<GraphModel, String> {
    let data_root = app
        .path()
        .app_data_dir()
        .map_err(|_| "app data dir not available")?
        .join("data");

    println!("▶ get_knowledge_graph");

    let knowledge = KnowledgeStore::load_or_create(&data_root)
        .await
        .map_err(|e| e.to_string())?;

    use std::collections::HashMap;

    let mut nodes: HashMap<String, GraphNode> = HashMap::new();
    let mut edges: Vec<GraphEdge> = Vec::new();

    for r in knowledge.all() {
        if r.status == "deprecated" {
            continue;
        }

        if r.confidence < 1.0 {
            continue;
        }

        let subject_node_id = node_id(&r.subject_type, &r.subject_value);
        let object_node_id = node_id("object", &r.object_value);

        nodes.entry(subject_node_id.clone()).or_insert(GraphNode {
            id: subject_node_id.clone(),
            label: r.subject_value.clone(),
            node_type: r.subject_type.clone(),
        });
        let object_type = infer_object_type(r); // small hack to demonstrate

        nodes.entry(object_node_id.clone()).or_insert(GraphNode {
            id: object_node_id.clone(),
            label: r.object_value.clone(),
            node_type: object_type,
        });

        edges.push(GraphEdge {
            from: subject_node_id,
            to: object_node_id,
            predicate: r.predicate.clone(),
            confidence: r.confidence,
        });
    }

    Ok(GraphModel {
        meta: GraphMeta {
            generated_at: Utc::now().to_rfc3339(),
        },
        nodes: nodes.into_values().collect(),
        edges,
    })
}
