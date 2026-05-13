//! Clipboard management module

mod db;
mod manager;

pub use db::ClipboardDb;
pub use manager::{ClipboardItem, ClipboardManager};
