use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStep {
    pub status: String,
    pub output: Option<String>,
    pub finished_at: Option<String>,
}
