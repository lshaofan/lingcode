use super::{DbConnection, Setting, Transcription};
use chrono::Utc;
use rusqlite::{params, Result};

pub struct SettingsRepository {
    conn: DbConnection,
}

impl SettingsRepository {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Setting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, updated_at FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok(Setting {
                key: row.get(0)?,
                value: row.get(1)?,
                updated_at: row.get::<_, String>(2)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        rows.collect()
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
        Ok(())
    }
}

pub struct TranscriptionRepository {
    conn: DbConnection,
}

impl TranscriptionRepository {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn create(&self, transcription: &Transcription) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transcriptions (text, audio_duration, model_version, language, created_at, app_context)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                transcription.text,
                transcription.audio_duration,
                transcription.model_version,
                transcription.language,
                transcription.created_at.to_rfc3339(),
                transcription.app_context,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<Transcription>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, text, audio_duration, model_version, language, created_at, app_context
             FROM transcriptions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Transcription {
                id: Some(row.get(0)?),
                text: row.get(1)?,
                audio_duration: row.get(2)?,
                model_version: row.get(3)?,
                language: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap_or(Utc::now()),
                app_context: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<Transcription>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, text, audio_duration, model_version, language, created_at, app_context
             FROM transcriptions
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(Transcription {
                id: Some(row.get(0)?),
                text: row.get(1)?,
                audio_duration: row.get(2)?,
                model_version: row.get(3)?,
                language: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap_or(Utc::now()),
                app_context: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn search(&self, query: &str) -> Result<Vec<Transcription>> {
        let conn = self.conn.lock().unwrap();
        let search_pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, text, audio_duration, model_version, language, created_at, app_context
             FROM transcriptions
             WHERE text LIKE ?1
             ORDER BY created_at DESC
             LIMIT 100",
        )?;
        let rows = stmt.query_map(params![search_pattern], |row| {
            Ok(Transcription {
                id: Some(row.get(0)?),
                text: row.get(1)?,
                audio_duration: row.get(2)?,
                model_version: row.get(3)?,
                language: row.get(4)?,
                created_at: row.get::<_, String>(5)?.parse().unwrap_or(Utc::now()),
                app_context: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions", [])?;
        Ok(())
    }
}
