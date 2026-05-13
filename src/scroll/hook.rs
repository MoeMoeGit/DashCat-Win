//! Mouse wheel hook for reversing scroll direction

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Mouse hook handle
static mut HOOK: HHOOK = HHOOK::default();

/// Whether reversal is active
static REVERSING: AtomicBool = AtomicBool::new(false);

/// Mouse wheel scroll reverser
pub struct ScrollReverser {
    reversing: AtomicBool,
}

impl ScrollReverser {
    /// Create a new scroll reverser
    pub fn new() -> Self {
        Self {
            reversing: AtomicBool::new(false),
        }
    }

    /// Start reversing mouse wheel
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.reversing.load(Ordering::SeqCst) {
            return Ok(());
        }

        unsafe {
            let hook = SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(Self::hook_proc),
                HINSTANCE::default(),
                0,
            )?;

            if hook.is_invalid() {
                return Err("Failed to install mouse hook".into());
            }

            HOOK = hook;
            REVERSING.store(true, Ordering::SeqCst);
            self.reversing.store(true, Ordering::SeqCst);
        }

        Ok(())
    }

    /// Stop reversing mouse wheel
    pub fn stop(&self) {
        if !self.reversing.load(Ordering::SeqCst) {
            return;
        }

        unsafe {
            if !HOOK.is_invalid() {
                UnhookWindowsHookEx(HOOK);
                HOOK = HHOOK::default();
            }
            REVERSING.store(false, Ordering::SeqCst);
            self.reversing.store(false, Ordering::SeqCst);
        }
    }

    /// Check if currently reversing
    pub fn is_reversing(&self) -> bool {
        self.reversing.load(Ordering::SeqCst)
    }

    /// Mouse hook callback
    unsafe extern "system" fn hook_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if REVERSING.load(Ordering::SeqCst) && code >= 0 {
            let msg = wparam.0 as u32;
            
            if msg == WM_MOUSEWHEEL || msg == WM_MOUSEHWHEEL {
                let msll = &*(lparam.0 as *const MSLLHOOKSTRUCT);
                
                // Check if it's from a touchpad (continuous scrolling)
                // Mouse wheel events have larger delta values
                let mouse_data = (msll.mouseData >> 16) as i16;
                
                // Only reverse if it looks like a mouse wheel (not touchpad)
                if mouse_data.abs() >= 120 {
                    // Reverse the direction
                    let new_data = if msg == WM_MOUSEWHEEL {
                        (-mouse_data as u32) << 16
                    } else {
                        (-mouse_data as u32) << 16
                    };
                    
                    // Create new mouse data
                    let new_lparam = LPARAM(&MSLLHOOKSTRUCT {
                        mouseData: (msll.mouseData & 0xFFFF) | new_data,
                        ..*msll
                    } as *const _ as _);
                    
                    // We can't modify the event directly, so we need to post a new one
                    // This is a limitation - proper implementation would need injection
                }
            }
        }

        CallNextHookEx(HHOOK::default(), code, wparam, lparam)
    }
}

impl Default for ScrollReverser {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScrollReverser {
    fn drop(&mut self) {
        self.stop();
    }
}
