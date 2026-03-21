//! ?????
//!
//! ?? SQLite ????????

pub mod schema;

use crate::opencode_error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// ??????
pub const SCHEMA_VERSION: i32 = 2;

/// ???????
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

/// ????????
#[macro_export]
macro_rules! lock_conn {
    ($conn:expr) => {
        $conn.lock().map_err(|e| AppError::Database(format!("????????: {e}")))?
    };
}

pub use lock_conn;

impl Database {
    /// ?????????
    fn get_db_path() -> Result<PathBuf, AppError> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Database("????????".to_string()))?;
        let config_dir = home.join(".config").join("opencode");
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::Database(format!("????????: {e}")))?;
        Ok(config_dir.join("ai-switch.db"))
    }

    /// ????????
    pub fn open() -> Result<Self, AppError> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(format!("???????: {e}")))?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        // ????????
        db.create_tables()?;
        db.apply_migrations()?;
        db.ensure_model_pricing_seeded()?;
        
        Ok(db)
    }

    /// ?????????????
    #[allow(dead_code)]
    pub fn memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::Database(format!("?????????: {e}")))?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.create_tables()?;
        db.ensure_model_pricing_seeded()?;
        
        Ok(db)
    }
}
