//! Mouse wheel reversal via WH_MOUSE_LL hook
//!
//! Uses Windows low-level mouse hook to intercept and reverse wheel events.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

/// State for wheel reversal
static REVERSING: AtomicBool = AtomicBool::new(false);

/// Hook handle (static for global access)
static mut HHOOK: HHOOK = HHOOK::default();

/// Mouse wheel reverser
pub struct WheelReverser {
    enabled: bool,
}

impl WheelReverser {
    /// Create a new wheel reverser
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Enable or disable wheel reversal
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled && !self.enabled {
            self.enable_hook();
        } else if !enabled && self.enabled {
            self.disable_hook();
        }
        self.enabled = enabled;
    }

    /// Check if reversal is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable the mouse hook
    fn enable_hook(&mut self) {
        // SAFETY: SetWindowsHookExW installs a low-level mouse hook.
        // The hook procedure must be in a module (not a closure) and must
        // process messages quickly to avoid system delays.
        unsafe {
            HHOOK = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(Self::hook_proc),
                HINSTANCE::default(),
                0,
            ).unwrap_or_default();

            if !HHOOK.is_invalid() {
                REVERSING.store(true, Ordering::SeqCst);
            }
        }
    }

    /// Disable the mouse hook
    fn disable_hook(&mut self) {
        // SAFETY: UnhookWindowsHookEx removes the hook. Safe with valid handle.
        unsafe {
            if !HHOOK.is_invalid() {
                let _ = UnhookWindowsHookEx(HHOOK);
                HHOOK = HHOOK::default();
            }
            REVERSING.store(false, Ordering::SeqCst);
        }
    }

    /// Low-level mouse hook procedure
    ///
    /// # Safety
    /// This is called by Windows on mouse events. Must return quickly.
    unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // Only process when reversal is enabled
        if REVERSING.load(Ordering::SeqCst) && code >= 0 {
            // Check for wheel event
            let msg = wparam.0 as u32;
            if msg == WM_MOUSEWHEEL {
                // Get the MSLLHOOKSTRUCT from lparam
                let msll = &*(lparam.0 as *const MSLLHOOKSTRUCT);
                let delta = HIWORD(msll.mouseData.0 as u32) as i16;

                // Reverse the wheel direction by negating delta
                let reversed_delta = -delta as u16;
                let new_mouse_data = MAKELONG(LOWORD(msll.mouseData.0 as u32), reversed_delta as u32);

                // Post a reversed wheel message to the window under cursor
                let pt = msll.pt;
                let hwnd = WindowFromPoint(pt);
                let _ = PostMessageW(
                    hwnd,
                    WM_MOUSEWHEEL,
                    WPARAM(new_mouse_data as usize),
                    LPARAM(((pt.y as u16 as usize) << 16) | (pt.x as u16 as usize)),
                );

                // Block the original event
                return LRESULT(1);
            }
        }

        // Pass to next hook
        CallNextHookEx(HHOOK, code, wparam, lparam)
    }
}

impl Default for WheelReverser {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WheelReverser {
    fn drop(&mut self) {
        if self.enabled {
            self.disable_hook();
        }
    }
}

// Helper macros (Windows constants)
const HIWORD: fn(u32) -> i16 = |x| ((x >> 16) & 0xFFFF) as i16;
const LOWORD: fn(u32) -> u16 = |x| (x & 0xFFFF) as u16;
const MAKELONG: fn(u16, u32) -> u32 = |lo, hi| ((hi as u32) << 16) | (lo as u32);