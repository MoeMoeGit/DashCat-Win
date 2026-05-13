//! SQLite database for clipboard history

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;

/// Clipboard database handler
pub struct ClipboardDb {
    conn: Connection,
}

impl ClipboardDb {
    /// Create or open the database
    pub fn new() -> SqliteResult<Self> {
        let db_path = Self::get_db_path();

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// Get the database file path
    fn get_db_path() -> PathBuf {
        let app_data = std::env::var("APPDATA")
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(app_data)
            .join("DashCat")
            .join("clipboard.db")
    }

    /// Initialize database tables
    fn init_tables(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT,
                image_path TEXT,
                source_app TEXT NOT NULL DEFAULT '',
                is_pinned INTEGER NOT NULL DEFAULT 0,
                created_at REAL NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_history(created_at)",
            [],
        )?;

        Ok(())
    }

    /// Insert a new clipboard item
    pub fn insert(&self, item: &NewClipboardItem) -> SqliteResult<i64> {
        let now = chrono::Utc::now().timestamp() as f64;

        self.conn.execute(
            "INSERT INTO clipboard_history (content, image_path, source_app, is_pinned, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                item.content,
                item.image_path,
                item.source_app,
                0,
                now
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get recent clipboard items
    pub fn get_recent(&self, limit: usize) -> SqliteResult<Vec<ClipboardRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, image_path, source_app, is_pinned, created_at
             FROM clipboard_history
             ORDER BY is_pinned DESC, created_at DESC
             LIMIT ?1"
        )?;

        let items = stmt.query_map([limit as i64], |row| {
            Ok(ClipboardRecord {
                id: row.get(0)?,
                content: row.get(1)?,
                image_path: row.get(2)?,
                source_app: row.get(3)?,
                is_pinned: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        })?;

        items.collect()
    }

    /// Search clipboard history
    pub fn search(&self, query: &str, limit: usize) -> SqliteResult<Vec<ClipboardRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, image_path, source_app, is_pinned, created_at
             FROM clipboard_history
             WHERE content LIKE ?1
             ORDER BY is_pinned DESC, created_at DESC
             LIMIT ?2"
        )?;

        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let items = stmt.query_map(rusqlite::params![pattern, limit as i64], |row| {
            Ok(ClipboardRecord {
                id: row.get(0)?,
                content: row.get(1)?,
                image_path: row.get(2)?,
                source_app: row.get(3)?,
                is_pinned: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        })?;

        items.collect()
    }

    /// Toggle pin status
    pub fn toggle_pin(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE clipboard_history SET is_pinned = 1 - is_pinned WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    /// Delete an item
    pub fn delete(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM clipboard_history WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    /// Clear all unpinned items
    pub fn clear_unpinned(&self) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM clipboard_history WHERE is_pinned = 0",
            [],
        )?;
        Ok(())
    }

    /// Clear expired items
    pub fn clear_expired(&self, days: u32) -> SqliteResult<()> {
        let cutoff = chrono::Utc::now().timestamp() as f64 - (days as f64 * 86400.0);
        self.conn.execute(
            "DELETE FROM clipboard_history WHERE is_pinned = 0 AND created_at < ?1",
            [cutoff],
        )?;
        Ok(())
    }
}

/// New clipboard item for insertion
pub struct NewClipboardItem {
    pub content: Option<String>,
    pub image_path: Option<String>,
    pub source_app: String,
}

/// Clipboard record from database
#[derive(Debug, Clone)]
pub struct ClipboardRecord {
    pub id: i64,
    pub content: Option<String>,
    pub image_path: Option<String>,
    pub source_app: String,
    pub is_pinned: bool,
    pub created_at: f64,
}
