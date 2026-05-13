//! DashCat-Win - Windows system tray cat
//!
//! A lightweight Windows system tray application that combines:
//! - Clipboard history management
//! - System monitoring (CPU/Memory)
//! - Sleep prevention
//! - Mouse wheel reversal

#![windows_subsystem = "windows"]

mod config;
mod tray;

use std::sync::Arc;
use tray::TrayApp;

fn main() {
    // Initialize logging (to file for debugging)
    init_logging();

    // Load configuration
    let config = config::Settings::load().unwrap_or_default();

    // Create and run tray application
    let app = Arc::new(TrayApp::new(config));
    
    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
    }
}

fn init_logging() {
    // For Windows GUI apps, logging to file
    // In release, we may skip this entirely
    #[cfg(debug_assertions)]
    {
        // Debug mode: also log to stdout (visible in terminal if attached)
    }
}
