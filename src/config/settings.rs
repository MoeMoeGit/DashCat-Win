//! User settings and preferences

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Monitor mode - which metric to display
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MonitorMode {
    Combined,
    Cpu,
    Memory,
}

/// Display mode - how to show the information
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DisplayMode {
    Both,        // Percentage + animation
    AnimOnly,    // Animation only
    PctOnly,     // Percentage only
    DualValues,  // CPU% / MEM%
}

/// Sleep prevention mode
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CaffeineMode {
    Off,
    NoSleep,        // Prevent system sleep, screen can turn off
    NoDisplaySleep, // Prevent screen from turning off
}

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "zh-TW")]
    TraditionalChinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "ru")]
    Russian,
}

impl Default for Language {
    fn default() -> Self {
        // Try to detect system language
        Self::English
    }
}

/// Main application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Monitor mode
    pub monitor_mode: MonitorMode,
    
    /// Display mode
    pub display_mode: DisplayMode,
    
    /// Sleep prevention mode
    pub caffeine_mode: CaffeineMode,
    
    /// Save images to clipboard history
    pub save_images: bool,
    
    /// Days to keep clipboard history
    pub history_days: u32,
    
    /// Reverse mouse wheel
    pub reverse_mouse_wheel: bool,
    
    /// Launch at Windows startup
    pub launch_at_startup: bool,
    
    /// UI language
    pub language: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            monitor_mode: MonitorMode::Combined,
            display_mode: DisplayMode::Both,
            caffeine_mode: CaffeineMode::Off,
            save_images: false,
            history_days: 30,
            reverse_mouse_wheel: false,
            launch_at_startup: false,
            language: Language::default(),
        }
    }
}

impl Settings {
    /// Get the path to the settings file
    fn settings_path() -> PathBuf {
        let app_data = std::env::var("APPDATA")
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(app_data)
            .join("DashCat")
            .join("settings.json")
    }

    /// Load settings from file, or return defaults if not found
    pub fn load() -> io::Result<Self> {
        let path = Self::settings_path();
        
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let settings: Settings = serde_json::from_str(&content)
            .unwrap_or_else(|_| Self::default());
        
        Ok(settings)
    }

    /// Save settings to file
    pub fn save(&self) -> io::Result<()> {
        let path = Self::settings_path();
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        
        Ok(())
    }
}
