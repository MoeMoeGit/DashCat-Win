//! Clipboard manager - monitors and manages clipboard history

use super::db::{ClipboardDb, ClipboardRecord, NewClipboardItem};
use crate::config::Settings;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows::Win32::System::DataExchange::*;
use windows::Win32::Foundation::HWND;

/// Clipboard item (public representation)
#[derive(Debug, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: Option<String>,
    pub image_path: Option<String>,
    pub source_app: String,
    pub is_pinned: bool,
    pub created_at: f64,
}

impl From<ClipboardRecord> for ClipboardItem {
    fn from(record: ClipboardRecord) -> Self {
        Self {
            id: record.id,
            content: record.content,
            image_path: record.image_path,
            source_app: record.source_app,
            is_pinned: record.is_pinned,
            created_at: record.created_at,
        }
    }
}

impl ClipboardItem {
    /// Check if this is an image item
    pub fn is_image(&self) -> bool {
        self.image_path.is_some()
    }
}

/// Clipboard manager
pub struct ClipboardManager {
    db: Arc<Mutex<ClipboardDb>>,
    settings: Arc<Mutex<Settings>>,
    last_sequence: u32,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new(settings: Arc<Mutex<Settings>>) -> Result<Self, Box<dyn std::error::Error>> {
        let db = ClipboardDb::new()?;
        
        // Clean up expired items on startup
        let days = settings.lock().unwrap().history_days;
        db.clear_expired(days)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            settings,
            last_sequence: 0,
        })
    }

    /// Check for clipboard changes
    pub fn check_for_changes(&mut self) -> Option<ClipboardItem> {
        unsafe {
            let sequence = GetClipboardSequenceNumber();
            if sequence == 0 || sequence == self.last_sequence {
                return None;
            }
            self.last_sequence = sequence;

            // Try to get clipboard content
            if !OpenClipboard(HWND::default()).as_bool() {
                return None;
            }

            // Try text first
            let text_data = GetClipboardData(13); // CF_UNICODETEXT
            if !text_data.is_invalid() {
                let ptr = GlobalLock(text_data);
                if !ptr.is_null() {
                    let text = std::ffi::CStr::from_ptr(ptr as *const i8)
                        .to_string_lossy()
                        .into_owned();
                    GlobalUnlock(text_data);
                    CloseClipboard();

                    // Truncate if too long
                    let text = if text.len() > 10000 {
                        text.chars().take(10000).collect()
                    } else {
                        text
                    };

                    // Get source app (would need additional work)
                    let source_app = String::new();

                    let item = NewClipboardItem {
                        content: Some(text),
                        image_path: None,
                        source_app,
                    };

                    let id = self.db.lock().unwrap().insert(&item).ok()?;
                    return Some(ClipboardItem {
                        id,
                        content: item.content,
                        image_path: None,
                        source_app: item.source_app,
                        is_pinned: false,
                        created_at: chrono::Utc::now().timestamp() as f64,
                    });
                }
            }

            CloseClipboard();
        }

        None
    }

    /// Get recent items
    pub fn get_recent(&self, limit: usize) -> Vec<ClipboardItem> {
        self.db
            .lock()
            .unwrap()
            .get_recent(limit)
            .unwrap_or_default()
            .into_iter()
            .map(ClipboardItem::from)
            .collect()
    }

    /// Search items
    pub fn search(&self, query: &str) -> Vec<ClipboardItem> {
        self.db
            .lock()
            .unwrap()
            .search(query, 200)
            .unwrap_or_default()
            .into_iter()
            .map(ClipboardItem::from)
            .collect()
    }

    /// Toggle pin status
    pub fn toggle_pin(&self, id: i64) {
        self.db.lock().unwrap().toggle_pin(id).ok();
    }

    /// Delete item
    pub fn delete(&self, id: i64) {
        // Also delete associated image file
        if let Ok(record) = self.db.lock().unwrap().get_recent(1000)
            .into_iter()
            .find(|r| r.id == id)
        {
            if let Some(path) = record.image_path {
                std::fs::remove_file(&path).ok();
                if let Some(thumb) = Self::thumbnail_path(&path) {
                    std::fs::remove_file(thumb).ok();
                }
            }
        }
        self.db.lock().unwrap().delete(id).ok();
    }

    /// Clear all history
    pub fn clear_all(&self) {
        // Delete all image files
        if let Ok(app_data) = std::env::var("APPDATA") {
            let images_dir = PathBuf::from(app_data).join("DashCat").join("Images");
            if images_dir.exists() {
                std::fs::remove_dir_all(&images_dir).ok();
                std::fs::create_dir_all(&images_dir).ok();
            }
        }
        self.db.lock().unwrap().clear_unpinned().ok();
    }

    /// Get thumbnail path from image path
    fn thumbnail_path(image_path: &str) -> Option<String> {
        let path = PathBuf::from(image_path);
        let stem = path.file_stem()?.to_str()?;
        let ext = path.extension()?.to_str()?;
        let parent = path.parent()?;
        Some(parent.join(format!("{}_thumb.{}", stem, ext)).to_string_lossy().into_owned())
    }
}
