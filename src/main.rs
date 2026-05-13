//! DashCat for Windows - A lightweight system tray application
//! 
//! Features:
//! - Clipboard history management
//! - System monitoring (CPU/Memory)
//! - Sleep prevention (Caffeine)
//! - Mouse wheel reversal

#![allow(unused)]

mod config;

use config::Settings;

fn main() {
    println!("DashCat-Win v0.1.0");
    println!("Loading settings...");
    
    let settings = Settings::load().unwrap_or_default();
    
    println!("Monitor mode: {:?}", settings.monitor_mode);
    println!("Caffeine mode: {:?}", settings.caffeine_mode);
    
    // TODO: Implement tray icon and full functionality
    println!("DashCat-Win is starting...");
}