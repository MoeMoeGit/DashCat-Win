//! Configuration settings

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Monitor display mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum MonitorMode {
    #[default]
    Combined,
    Cpu,
    Memory,
}


/// Display mode for tray icon
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum DisplayMode {
    #[default]
    Both,
    AnimOnly,
    PctOnly,
    DualValues,
}


/// Sleep prevention mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum CaffeineMode {
    #[default]
    Off,
    NoSleep,
    NoDisplaySleep,
}


/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub monitor_mode: MonitorMode,
    pub display_mode: DisplayMode,
    pub caffeine_mode: CaffeineMode,
    pub save_images: bool,
    pub reverse_mouse_wheel: bool,
    pub launch_at_startup: bool,
    pub language: String,
    pub history_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            monitor_mode: MonitorMode::default(),
            display_mode: DisplayMode::default(),
            caffeine_mode: CaffeineMode::default(),
            save_images: false,
            reverse_mouse_wheel: false,
            launch_at_startup: false,
            language: "en".to_string(),
            history_limit: 50,
        }
    }
}

impl Settings {
    /// Load settings from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::settings_path();
        
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let settings: Settings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::settings_path();
        
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        
        Ok(())
    }

    /// Get the settings file path
    fn settings_path() -> PathBuf {
        // On Windows this will be %APPDATA%\DashCat\settings.json
        // On Linux for testing: ~/.local/share/DashCat/settings.json
        if let Some(data_dir) = std::env::var_os("APPDATA") {
            PathBuf::from(data_dir).join("DashCat").join("settings.json")
        } else if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".local/share/DashCat/settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }
}