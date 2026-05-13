//! Sleep prevention using SetThreadExecutionState

use crate::config::CaffeineMode;

use windows::Win32::System::Power::{SetThreadExecutionState, EXECUTION_STATE};

// Execution state flags
const ES_CONTINUOUS: EXECUTION_STATE = EXECUTION_STATE(0x80000000);
const ES_SYSTEM_REQUIRED: EXECUTION_STATE = EXECUTION_STATE(0x00000001);
const ES_DISPLAY_REQUIRED: EXECUTION_STATE = EXECUTION_STATE(0x00000002);

/// Sleep prevention handler
pub struct Caffeine {
    current_mode: CaffeineMode,
}

impl Caffeine {
    pub fn new() -> Self {
        Self { current_mode: CaffeineMode::Off }
    }

    /// Set caffeine mode
    pub fn set_mode(&mut self, mode: CaffeineMode) {
        if self.current_mode == mode {
            return;
        }

        self.current_mode = mode;

        unsafe {
            match mode {
                CaffeineMode::Off => {
                    // Clear all flags - allow normal sleep
                    SetThreadExecutionState(ES_CONTINUOUS);
                }
                CaffeineMode::NoSleep => {
                    // Prevent system from sleeping (display can turn off)
                    SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
                }
                CaffeineMode::NoDisplaySleep => {
                    // Prevent both system and display from sleeping
                    SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
                }
            }
        }
    }
}

impl Default for Caffeine {
    fn default() -> Self { Self::new() }
}