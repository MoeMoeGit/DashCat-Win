//! Configuration management

pub mod settings;
pub mod autostart;

pub use settings::{Settings, MonitorMode, DisplayMode, CaffeineMode};
pub use autostart::{is_auto_start_enabled, enable_auto_start, disable_auto_start};