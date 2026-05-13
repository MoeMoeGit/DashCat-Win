//! Clipboard history management
//!
//! Provides clipboard capture, storage, and retrieval.

mod db;

pub use db::{ClipboardDb, ClipboardEntry};

use std::path::PathBuf;

use clipboard_rs::{Clipboard, ClipboardContext};

/// Clipboard manager with history
pub struct ClipboardManager {
    db: ClipboardDb,
    save_images: bool,
    context: ClipboardContext,
}

impl ClipboardManager {
    /// Create clipboard manager
    pub fn new(data_dir: PathBuf, save_images: bool) -> Self {
        let db_path = data_dir.join("clipboard.db");
        let db = ClipboardDb::open(db_path)
            .expect("Failed to open clipboard database");
        
        let context = ClipboardContext::new()
            .expect("Failed to create clipboard context");
        
        Self { db, save_images, context }
    }
    
    /// Get current clipboard content and save to history
    pub fn capture(&mut self) -> bool {
        // Try text first
        if let Ok(text) = self.context.get_text() {
            if !text.is_empty() {
                let preview = Self::make_preview(&text, 100);
                let _ = self.db.add(text.into_bytes(), "text", &preview);
                return true;
            }
        }
        
        // Try image if enabled
        if self.save_images {
            if let Ok(image) = self.context.get_image() {
                // Save image as PNG bytes
                // TODO: implement image serialization
            }
        }
        
        false
    }
    
    /// Copy entry to clipboard
    pub fn copy_entry(&self, entry: &ClipboardEntry) -> bool {
        if entry.content_type == "text" {
            let text = String::from_utf8_lossy(&entry.content).to_string();
            self.context.set_text(text).is_ok()
        } else {
            false
        }
    }
    
    /// Get recent entries
    pub fn get_recent(&self, limit: usize) -> Vec<ClipboardEntry> {
        self.db.get_recent(limit).unwrap_or_default()
    }
    
    /// Search entries
    pub fn search(&self, query: &str, limit: usize) -> Vec<ClipboardEntry> {
        self.db.search(query, limit).unwrap_or_default()
    }
    
    /// Toggle pin
    pub fn toggle_pin(&self, id: i64) {
        let _ = self.db.toggle_pin(id);
    }
    
    /// Delete entry
    pub fn delete(&self, id: i64) {
        let _ = self.db.delete(id);
    }
    
    /// Clear old entries
    pub fn clear_old(&self, days: i32) {
        let _ = self.db.clear_old(days);
    }
    
    /// Clear all
    pub fn clear_all(&self) {
        let _ = self.db.clear_all();
    }
    
    /// Set save images option
    pub fn set_save_images(&mut self, enabled: bool) {
        self.save_images = enabled;
    }
    
    /// Make preview text
    fn make_preview(text: &str, max_len: usize) -> String {
        text.chars()
            .filter(|c| !c.is_control() || *c == ' ')
            .take(max_len)
            .collect()
    }
}