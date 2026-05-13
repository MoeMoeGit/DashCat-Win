//! DashCat for Windows - System tray application
//!
//! A lightweight system monitoring tool that displays CPU/Memory usage
//! in the system tray with a cute animated cat icon.

mod clipboard;
mod config;
mod monitor;
mod power;
mod tray;

use config::Settings;
use tray::TrayApp;

fn main() {
    // SAFETY: CoInitializeEx is required for COM-based Windows APIs.
    // COINIT_MULTITHREADED is safe for single-threaded application.
    // The returned COM state is automatically cleaned up on thread exit.
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }

    // Load settings or use defaults
    let settings = Settings::load().unwrap_or_default();

    // Run the tray application
    if let Err(e) = TrayApp::run(settings) {
        eprintln!("DashCat error: {}", e);
        std::process::exit(1);
    }
}