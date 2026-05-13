//! CPU usage monitoring using Windows PDH

use std::time::{Duration, Instant};

use windows::Win32::System::Performance::*;

/// CPU usage monitor
pub struct CpuMonitor {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
    last_time: Instant,
}

impl CpuMonitor {
    /// Create a new CPU monitor
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let mut query = PDH_HQUERY::default();
            PdhOpenQueryW(None, 0, &mut query)?;

            // Add counter for total CPU usage
            let mut counter = PDH_HCOUNTER::default();
            let counter_path = windows::core::w!("\\Processor(_Total)\\% Processor Time");
            PdhAddCounterW(query, counter_path, 0, &mut counter)?;

            // Collect initial sample
            PdhCollectQueryData(query)?;

            Ok(Self {
                query,
                counter,
                last_time: Instant::now(),
            })
        }
    }

    /// Get current CPU usage percentage
    pub fn usage(&mut self) -> f32 {
        // Minimum interval between samples
        if self.last_time.elapsed() < Duration::from_millis(100) {
            return 0.0;
        }

        unsafe {
            if PdhCollectQueryData(self.query).is_err() {
                return 0.0;
            }

            let mut value = PDH_FMT_COUNTERVALUE::default();
            if PdhGetFormattedCounterValue(self.counter, PDH_FMT_DOUBLE, None, &mut value).is_ok() {
                self.last_time = Instant::now();
                return value.u.doubleValue() as f32;
            }
        }

        0.0
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            query: PDH_HQUERY::default(),
            counter: PDH_HCOUNTER::default(),
            last_time: Instant::now(),
        })
    }
}

impl Drop for CpuMonitor {
    fn drop(&mut self) {
        unsafe {
            if !self.query.is_invalid() {
                PdhCloseQuery(self.query);
            }
        }
    }
}
