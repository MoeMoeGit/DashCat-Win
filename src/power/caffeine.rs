//! Caffeine - prevent system sleep using SetThreadExecutionState

use windows::Win32::System::Power::*;
use windows::Win32::System::Threading::EXECUTION_STATE;

use crate::config::CaffeineMode;

/// Sleep prevention handler
pub struct Caffeine {
    current_mode: CaffeineMode,
}

impl Caffeine {
    /// Create a new caffeine handler
    pub fn new() -> Self {
        Self {
            current_mode: CaffeineMode::Off,
        }
    }

    /// Set the caffeine mode
    pub fn set_mode(&mut self, mode: CaffeineMode) {
        self.current_mode = mode;
        
        unsafe {
            match mode {
                CaffeineMode::Off => {
                    // Reset to default
                    SetThreadExecutionState(ES_CONTINUOUS);
                }
                CaffeineMode::NoSleep => {
                    // Prevent system from sleeping, but allow display to turn off
                    SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
                }
                CaffeineMode::NoDisplaySleep => {
                    // Prevent display from turning off and system from sleeping
                    SetThreadExecutionState(
                        ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
                    );
                }
            }
        }
    }

    /// Get current mode
    pub fn current_mode(&self) -> CaffeineMode {
        self.current_mode
    }
}

impl Default for Caffeine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Caffeine {
    fn drop(&mut self) {
        // Reset when dropping
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}
