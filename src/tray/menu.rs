//! Context menu for the tray icon

use crate::config::{CaffeineMode, MonitorMode, DisplayMode, is_auto_start_enabled};
use windows::core::*;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct TrayMenu { hmenu: HMENU }

impl TrayMenu {
    pub fn new() -> Self { Self { hmenu: HMENU::default() } }

    pub unsafe fn show(&mut self, hwnd: HWND, monitor_mode: MonitorMode, display_mode: DisplayMode, caffeine_mode: CaffeineMode) {
        self.hmenu = CreatePopupMenu().unwrap_or_default();

        // Version info (grayed out, first line)
        let version = env!("CARGO_PKG_VERSION");
        let version_text = format!("DashCat v{}", version);
        let version_wide: Vec<u16> = version_text.encode_utf16().chain(std::iter::once(0)).collect();
        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, PCWSTR(version_wide.as_ptr()));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        // Monitor section
        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, w!("Monitor Mode"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(monitor_mode == MonitorMode::Combined), 100, w!("  Combined"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(monitor_mode == MonitorMode::Cpu), 101, w!("  CPU Only"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(monitor_mode == MonitorMode::Memory), 102, w!("  Memory Only"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        // Display section
        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, w!("Display Mode"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(display_mode == DisplayMode::Both), 200, w!("  Animation + Percent"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(display_mode == DisplayMode::AnimOnly), 201, w!("  Animation Only"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(display_mode == DisplayMode::PctOnly), 202, w!("  Percent Only"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(display_mode == DisplayMode::DualValues), 203, w!("  Dual Values"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        // Caffeine section
        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, w!("Sleep Prevention"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(caffeine_mode == CaffeineMode::Off), 300, w!("  Off"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(caffeine_mode == CaffeineMode::NoSleep), 301, w!("  Prevent System Sleep"));
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(caffeine_mode == CaffeineMode::NoDisplaySleep), 302, w!("  Prevent Display Sleep"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        // Auto-start section
        let auto_enabled = is_auto_start_enabled();
        let _ = AppendMenuW(self.hmenu, MF_STRING | check(auto_enabled), 400, w!("Launch at Login"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        // Quit
        let _ = AppendMenuW(self.hmenu, MF_STRING, 999, w!("Quit DashCat"));

        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(self.hmenu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, None);
        let _ = DestroyMenu(self.hmenu);
    }
}

#[inline]
fn check(condition: bool) -> MENU_ITEM_FLAGS {
    if condition { MF_CHECKED } else { MENU_ITEM_FLAGS(0) }
}

impl Default for TrayMenu {
    fn default() -> Self { Self::new() }
}