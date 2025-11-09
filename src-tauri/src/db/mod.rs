mod models;
mod repository;
mod schema;

pub use models::*;
pub use repository::*;
pub use schema::*;

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

pub struct Database {
    conn: DbConnection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let conn = Arc::new(Mutex::new(conn));

        // Initialize schema
        schema::init_database(&conn)?;

        Ok(Self { conn })
    }

    pub fn connection(&self) -> DbConnection {
        Arc::clone(&self.conn)
    }
}
