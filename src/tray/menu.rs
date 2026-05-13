//! Context menu for the tray icon

use crate::config::{CaffeineMode, DisplayMode, MonitorMode, Settings};

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::HWND;

/// Tray menu handler
pub struct TrayMenu {
    hmenu: HMENU,
}

impl TrayMenu {
    /// Create a new menu handler
    pub fn new() -> Self {
        Self {
            hmenu: HMENU::default(),
        }
    }

    /// Build the context menu based on current settings
    pub fn build(&mut self, settings: &Settings) {
        unsafe {
            self.hmenu = CreatePopupMenu();

            // === Monitor Section ===
            AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, windows::core::w!("Monitor"));
            
            let monitor_modes = [
                (MonitorMode::Combined, "Combined"),
                (MonitorMode::Cpu, "CPU"),
                (MonitorMode::Memory, "Memory"),
            ];
            
            for (i, (mode, label)) in monitor_modes.iter().enumerate() {
                let checked = settings.monitor_mode == *mode;
                let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
                let label_wide = format!("  {}", label);
                let label_wide: Vec<u16> = label_wide.encode_utf16().chain(std::iter::once(0)).collect();
                AppendMenuW(self.hmenu, flags, 100 + i as usize, PCWSTR(label_wide.as_ptr()));
            }

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Display Submenu ===
            let display_menu = CreatePopupMenu();
            let display_modes = [
                (DisplayMode::Both, "Percentage & Animation"),
                (DisplayMode::AnimOnly, "Animation Only"),
                (DisplayMode::PctOnly, "Percentage Only"),
                (DisplayMode::DualValues, "Dual Values"),
            ];
            
            for (i, (mode, label)) in display_modes.iter().enumerate() {
                let checked = settings.display_mode == *mode;
                let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
                let label_wide: Vec<u16> = label.encode_utf16().chain(std::iter::once(0)).collect();
                AppendMenuW(display_menu, flags, 200 + i as usize, PCWSTR(label_wide.as_ptr()));
            }

            AppendMenuW(self.hmenu, MF_STRING | MF_POPUP, display_menu.0 as usize, windows::core::w!("Display"));

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Sleep Prevention Section ===
            AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, windows::core::w!("Sleep Prevention"));
            
            let caffeine_modes = [
                (CaffeineMode::Off, "Off"),
                (CaffeineMode::NoSleep, "Prevent System Sleep"),
                (CaffeineMode::NoDisplaySleep, "Prevent Display Sleep"),
            ];
            
            for (i, (mode, label)) in caffeine_modes.iter().enumerate() {
                let checked = settings.caffeine_mode == *mode;
                let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
                let label_wide = format!("  {}", label);
                let label_wide: Vec<u16> = label_wide.encode_utf16().chain(std::iter::once(0)).collect();
                AppendMenuW(self.hmenu, flags, 300 + i as usize, PCWSTR(label_wide.as_ptr()));
            }

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Clipboard Section ===
            AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, windows::core::w!("Clipboard"));

            let save_images_flags = MF_STRING | if settings.save_images { MF_CHECKED } else { 0 };
            AppendMenuW(self.hmenu, save_images_flags, 400, windows::core::w!("  Save Images"));

            AppendMenuW(self.hmenu, MF_STRING, 401, windows::core::w!("  Clear History"));

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Other Settings ===
            let reverse_scroll_flags = MF_STRING | if settings.reverse_mouse_wheel { MF_CHECKED } else { 0 };
            AppendMenuW(self.hmenu, reverse_scroll_flags, 500, windows::core::w!("Reverse Mouse Wheel"));

            let launch_flags = MF_STRING | if settings.launch_at_startup { MF_CHECKED } else { 0 };
            AppendMenuW(self.hmenu, launch_flags, 501, windows::core::w!("Launch at Startup"));

            // Language submenu
            let lang_menu = CreatePopupMenu();
            AppendMenuW(self.hmenu, MF_STRING | MF_POPUP, lang_menu.0 as usize, windows::core::w!("Language"));

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Help ===
            AppendMenuW(self.hmenu, MF_STRING, 600, windows::core::w!("Check for Updates..."));
            AppendMenuW(self.hmenu, MF_STRING, 601, windows::core::w!("View on GitHub"));

            AppendMenuW(self.hmenu, MF_SEPARATOR, 0, windows::core::w!(""));

            // === Quit ===
            AppendMenuW(self.hmenu, MF_STRING, 999, windows::core::w!("Quit DashCat"));
        }
    }

    /// Show the context menu at the cursor position
    pub fn show(&self, hwnd: HWND) {
        unsafe {
            let mut point = POINT { x: 0, y: 0 };
            GetCursorPos(&mut point);

            // Required for the menu to close when clicking outside
            SetForegroundWindow(hwnd);

            TrackPopupMenu(
                self.hmenu,
                TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                0,
                hwnd,
                None,
            );
        }
    }

    /// Handle menu item selection
    pub fn handle_command(&mut self, command_id: usize) -> Option<MenuAction> {
        match command_id {
            100..=102 => {
                // Monitor mode
                Some(MenuAction::SetMonitorMode(match command_id {
                    100 => MonitorMode::Combined,
                    101 => MonitorMode::Cpu,
                    102 => MonitorMode::Memory,
                    _ => MonitorMode::Combined,
                }))
            }
            200..=203 => {
                // Display mode
                Some(MenuAction::SetDisplayMode(match command_id {
                    200 => DisplayMode::Both,
                    201 => DisplayMode::AnimOnly,
                    202 => DisplayMode::PctOnly,
                    203 => DisplayMode::DualValues,
                    _ => DisplayMode::Both,
                }))
            }
            300..=302 => {
                // Caffeine mode
                Some(MenuAction::SetCaffeineMode(match command_id {
                    300 => CaffeineMode::Off,
                    301 => CaffeineMode::NoSleep,
                    302 => CaffeineMode::NoDisplaySleep,
                    _ => CaffeineMode::Off,
                }))
            }
            400 => Some(MenuAction::ToggleSaveImages),
            401 => Some(MenuAction::ClearHistory),
            500 => Some(MenuAction::ToggleReverseScroll),
            501 => Some(MenuAction::ToggleLaunchAtStartup),
            600 => Some(MenuAction::CheckUpdates),
            601 => Some(MenuAction::ViewGitHub),
            999 => Some(MenuAction::Quit),
            _ => None,
        }
    }
}

impl Default for TrayMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TrayMenu {
    fn drop(&mut self) {
        unsafe {
            if !self.hmenu.is_invalid() {
                DestroyMenu(self.hmenu);
            }
        }
    }
}

/// Actions that can be triggered from the menu
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    SetMonitorMode(MonitorMode),
    SetDisplayMode(DisplayMode),
    SetCaffeineMode(CaffeineMode),
    ToggleSaveImages,
    ClearHistory,
    ToggleReverseScroll,
    ToggleLaunchAtStartup,
    CheckUpdates,
    ViewGitHub,
    Quit,
}
