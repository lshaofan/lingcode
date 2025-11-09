use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

const CURRENT_VERSION: i32 = 1;

pub fn init_database(conn: &Arc<Mutex<Connection>>) -> Result<()> {
    let conn = conn.lock().unwrap();

    // Create metadata table for version tracking
    conn.execute(
        "CREATE TABLE IF NOT EXISTS db_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Check current version
    let version: i32 = conn
        .query_row(
            "SELECT value FROM db_metadata WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version == 0 {
        // First time initialization
        create_initial_schema(&conn)?;
        set_db_version(&conn, CURRENT_VERSION)?;
    } else if version < CURRENT_VERSION {
        // Migration needed
        migrate_database(&conn, version, CURRENT_VERSION)?;
    }

    Ok(())
}

fn create_initial_schema(conn: &Connection) -> Result<()> {
    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Transcriptions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transcriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            audio_duration REAL,
            model_version TEXT,
            language TEXT DEFAULT 'zh',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            app_context TEXT
        )",
        [],
    )?;

    // Create index for faster queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at
         ON transcriptions(created_at DESC)",
        [],
    )?;

    // Insert default settings
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES
         ('language', 'zh'),
         ('model', 'base'),
         ('shortcut', 'Cmd+Shift+S')",
        [],
    )?;

    Ok(())
}

fn set_db_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO db_metadata (key, value) VALUES ('version', ?1)",
        [version],
    )?;
    Ok(())
}

fn migrate_database(conn: &Connection, from_version: i32, to_version: i32) -> Result<()> {
    // Placeholder for future migrations
    tracing::info!(
        "Migrating database from version {} to {}",
        from_version,
        to_version
    );

    // Apply migrations incrementally
    for version in from_version..to_version {
        match version {
            // Future migrations will go here
            // 1 => migrate_v1_to_v2(conn)?,
            // 2 => migrate_v2_to_v3(conn)?,
            _ => {}
        }
    }

    set_db_version(conn, to_version)?;
    Ok(())
}
