//! CPU usage monitoring via Windows PDH

use std::time::{Duration, Instant};

use windows::core::*;
use windows::Win32::System::Performance::*;

/// CPU usage monitor using PDH counters
pub struct CpuMonitor {
    query: isize,
    counter: isize,
    last_time: Instant,
    initialized: bool,
}

impl CpuMonitor {
    /// Create a new CPU monitor
    pub fn new() -> Self {
        Self {
            query: 0,
            counter: 0,
            last_time: Instant::now(),
            initialized: false,
        }
    }

    /// Initialize the PDH query and counter
    fn init(&mut self) -> Result<()> {
        unsafe {
            // Open a query
            let mut query: isize = 0;
            if PdhOpenQueryW(None, 0, &mut query) != 0 {
                return Err(Error::from(HRESULT(-1)));
            }
            self.query = query;

            // Add counter for total CPU usage
            let mut counter: isize = 0;
            // Try modern counter first
            let counter_path = w!("\\Processor Information(_Total)\\% Processor Time");
            let result = PdhAddCounterW(self.query, counter_path, 0, &mut counter);
            if result != 0 {
                // Fallback to legacy counter
                let counter_path2 = w!("\\Processor(_Total)\\% Processor Time");
                if PdhAddCounterW(self.query, counter_path2, 0, &mut counter) != 0 {
                    return Err(Error::from(HRESULT(-1)));
                }
            }
            self.counter = counter;

            // Collect initial sample
            PdhCollectQueryData(self.query);

            self.initialized = true;
        }
        Ok(())
    }

    /// Get current CPU usage percentage
    pub fn usage(&mut self) -> f32 {
        // Throttle to 100ms minimum
        if self.last_time.elapsed() < Duration::from_millis(100) {
            return 0.0;
        }

        // Initialize on first use
        if !self.initialized {
            if self.init().is_err() {
                return 0.0;
            }
            self.last_time = Instant::now();
            return 0.0;
        }

        unsafe {
            if PdhCollectQueryData(self.query) != 0 {
                return 0.0;
            }

            let mut value = PDH_FMT_COUNTERVALUE {
                CStatus: 0,
                Anonymous: PDH_FMT_COUNTERVALUE_0 { doubleValue: 0.0 },
            };

            if PdhGetFormattedCounterValue(self.counter, PDH_FMT_DOUBLE, None, &mut value) == 0 {
                self.last_time = Instant::now();
                let val = value.Anonymous.doubleValue as f32;
                return val.max(0.0).min(100.0);
            }
        }

        0.0
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CpuMonitor {
    fn drop(&mut self) {
        if self.initialized && self.query != 0 {
            unsafe {
                PdhCloseQuery(self.query);
            }
        }
    }
}