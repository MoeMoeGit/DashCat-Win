//! Context menu for the tray icon

use windows::core::*;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct TrayMenu { hmenu: HMENU }

impl TrayMenu {
    pub fn new() -> Self { Self { hmenu: HMENU::default() } }

    pub unsafe fn show(&mut self, hwnd: HWND) {
        self.hmenu = CreatePopupMenu().unwrap_or_default();

        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, w!("Monitor"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 100, w!("  Combined"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 101, w!("  CPU"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 102, w!("  Memory"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        let _ = AppendMenuW(self.hmenu, MF_STRING | MF_GRAYED, 0, w!("Sleep Prevention"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 300, w!("  Off"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 301, w!("  Prevent System Sleep"));
        let _ = AppendMenuW(self.hmenu, MF_STRING, 302, w!("  Prevent Display Sleep"));
        let _ = AppendMenuW(self.hmenu, MF_SEPARATOR, 0, w!(""));

        let _ = AppendMenuW(self.hmenu, MF_STRING, 999, w!("Quit DashCat"));

        let mut point = POINT { x: 0, y: 0 };
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(self.hmenu, TPM_RIGHTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, None);
        let _ = DestroyMenu(self.hmenu);
    }
}

impl Default for TrayMenu {
    fn default() -> Self { Self::new() }
}