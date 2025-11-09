use crate::db::{Database, SettingsRepository, Transcription, TranscriptionRepository};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn get_setting(db: State<Arc<Database>>, key: String) -> Result<Option<String>, String> {
    let repo = SettingsRepository::new(db.connection());
    repo.get(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(db: State<Arc<Database>>, key: String, value: String) -> Result<(), String> {
    let repo = SettingsRepository::new(db.connection());
    repo.set(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_settings(db: State<Arc<Database>>) -> Result<Vec<crate::db::Setting>, String> {
    let repo = SettingsRepository::new(db.connection());
    repo.get_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_setting(db: State<Arc<Database>>, key: String) -> Result<(), String> {
    let repo = SettingsRepository::new(db.connection());
    repo.delete(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_transcription(
    db: State<Arc<Database>>,
    transcription: Transcription,
) -> Result<i64, String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.create(&transcription).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_transcription(db: State<Arc<Database>>, id: i64) -> Result<Option<Transcription>, String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.get_by_id(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent_transcriptions(
    db: State<Arc<Database>>,
    limit: usize,
) -> Result<Vec<Transcription>, String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.get_recent(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_transcriptions(
    db: State<Arc<Database>>,
    query: String,
) -> Result<Vec<Transcription>, String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.search(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_transcription(db: State<Arc<Database>>, id: i64) -> Result<(), String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_all_transcriptions(db: State<Arc<Database>>) -> Result<(), String> {
    let repo = TranscriptionRepository::new(db.connection());
    repo.delete_all().map_err(|e| e.to_string())
}
