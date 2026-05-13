//! CPU usage monitoring via Windows PDH

use std::time::{Duration, Instant};

use windows::core::*;
use windows::Win32::System::Performance::*;

/// CPU usage monitor using PDH counters
/// 
/// Uses Windows Performance Data Helper (PDH) API to query
/// "\\Processor(_Total)\\% Processor Time" counter.
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
    /// 
    /// # Safety
    /// Calls Windows PDH API functions which are safe when passed valid parameters.
    fn init(&mut self) -> Result<()> {
        // SAFETY: PDH API calls with valid parameters. Query and counter handles
        // are checked for validity before use.
        unsafe {
            // Open a query
            let mut query: isize = 0;
            if PdhOpenQueryW(None, 0, &mut query) != 0 {
                return Err(Error::from(HRESULT(-1)));
            }
            self.query = query;

            // Add counter for total CPU usage
            let mut counter: isize = 0;
            // Try modern counter first (Processor Information supports NUMA)
            let counter_path = w!("\\Processor Information(_Total)\\% Processor Time");
            let result = PdhAddCounterW(self.query, counter_path, 0, &mut counter);
            if result != 0 {
                // Fallback to legacy counter for older Windows versions
                let counter_path2 = w!("\\Processor(_Total)\\% Processor Time");
                if PdhAddCounterW(self.query, counter_path2, 0, &mut counter) != 0 {
                    return Err(Error::from(HRESULT(-1)));
                }
            }
            self.counter = counter;

            // Collect initial sample (required before getting formatted value)
            PdhCollectQueryData(self.query);

            self.initialized = true;
        }
        Ok(())
    }

    /// Get current CPU usage percentage
    /// 
    /// Returns a value between 0.0 and 100.0.
    /// Throttled to minimum 100ms between updates.
    pub fn usage(&mut self) -> f32 {
        // Throttle to 100ms minimum to avoid excessive API calls
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

        // SAFETY: Query and counter handles were validated during init().
        // PDH API calls are safe with valid handles.
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
                return val.clamp(0.0, 100.0);
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
        // SAFETY: Query handle was validated during init().
        // PdhCloseQuery is safe to call with a valid handle.
        if self.initialized && self.query != 0 {
            unsafe {
                PdhCloseQuery(self.query);
            }
        }
    }
}