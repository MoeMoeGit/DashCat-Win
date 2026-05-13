//! Clipboard history database

use rusqlite::{Connection, Result as SqliteResult, params};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Clipboard entry
#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: Vec<u8>,
    pub content_type: String,  // "text" or "image"
    pub is_pinned: bool,
    pub created_at: i64,
    pub preview: String,
}

/// Clipboard database
pub struct ClipboardDb {
    conn: Connection,
}

impl ClipboardDb {
    /// Open or create database
    pub fn open(path: PathBuf) -> SqliteResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let conn = Connection::open(path)?;
        
        // Create table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content BLOB NOT NULL,
                content_type TEXT NOT NULL,
                is_pinned INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                preview TEXT
            )",
            [],
        )?;
        
        // Create index for search
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_preview ON clipboard(preview)",
            [],
        )?;
        
        Ok(Self { conn })
    }
    
    /// Add a new entry
    pub fn add(&self, content: Vec<u8>, content_type: &str, preview: &str) -> SqliteResult<i64> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        self.conn.execute(
            "INSERT INTO clipboard (content, content_type, is_pinned, created_at, preview)
             VALUES (?1, ?2, 0, ?3, ?4)",
            params![&content, content_type, created_at, preview],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    /// Get recent entries (pinned first, then by date)
    pub fn get_recent(&self, limit: usize) -> SqliteResult<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_type, is_pinned, created_at, preview
             FROM clipboard
             ORDER BY is_pinned DESC, created_at DESC
             LIMIT ?1"
        )?;
        
        let entries = stmt.query_map(params![limit as i32], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                is_pinned: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                preview: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>();
        
        entries
    }
    
    /// Search entries
    pub fn search(&self, query: &str, limit: usize) -> SqliteResult<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_type, is_pinned, created_at, preview
             FROM clipboard
             WHERE preview LIKE ?1
             ORDER BY is_pinned DESC, created_at DESC
             LIMIT ?2"
        )?;
        
        let pattern = format!("%{}%", query);
        let entries = stmt.query_map(params![pattern, limit as i32], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                is_pinned: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                preview: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>();
        
        entries
    }
    
    /// Toggle pin status
    pub fn toggle_pin(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE clipboard SET is_pinned = NOT is_pinned WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
    
    /// Delete entry
    pub fn delete(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM clipboard WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }
    
    /// Clear all non-pinned entries older than days
    pub fn clear_old(&self, days: i32) -> SqliteResult<usize> {
        let cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (days as i64 * 24 * 60 * 60);
        
        self.conn.execute(
            "DELETE FROM clipboard WHERE is_pinned = 0 AND created_at < ?1",
            params![cutoff],
        )
    }
    
    /// Clear all history
    pub fn clear_all(&self) -> SqliteResult<usize> {
        self.conn.execute("DELETE FROM clipboard WHERE is_pinned = 0", [])
    }
    
    /// Get entry by id
    pub fn get(&self, id: i64) -> SqliteResult<Option<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, content_type, is_pinned, created_at, preview
             FROM clipboard WHERE id = ?1"
        )?;
        
        let mut entries = stmt.query_map(params![id], |row| {
            Ok(ClipboardEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                is_pinned: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
                preview: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok(entries.pop())
    }
}