//! DashCat for Windows - System tray application

mod config;
mod monitor;
mod power;
mod tray;

use config::Settings;
use tray::TrayApp;

fn main() {
    // Initialize COM (required for some Windows APIs)
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    // Load or create default settings
    let settings = Settings::load().unwrap_or_default();

    // Create and run the tray application
    if let Err(e) = TrayApp::run(settings) {
        eprintln!("Error: {}", e);
    }
}