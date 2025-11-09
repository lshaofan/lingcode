use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcription {
    pub id: Option<i64>,
    pub text: String,
    pub audio_duration: Option<f64>,
    pub model_version: Option<String>,
    pub language: String,
    pub created_at: DateTime<Utc>,
    pub app_context: Option<String>,
}

impl Transcription {
    pub fn new(text: String) -> Self {
        Self {
            id: None,
            text,
            audio_duration: None,
            model_version: None,
            language: "zh".to_string(),
            created_at: Utc::now(),
            app_context: None,
        }
    }
}
