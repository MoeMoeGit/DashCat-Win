//! Memory usage monitoring using Windows GlobalMemoryStatusEx

use windows::Win32::Foundation::LONGLONG;

/// Memory status information
#[repr(C)]
struct MemoryStatusEx {
    dwLength: u32,
    dwMemoryLoad: u32,
    ullTotalPhys: u64,
    ullAvailPhys: u64,
    ullTotalPageFile: u64,
    ullAvailPageFile: u64,
    ullTotalVirtual: u64,
    ullAvailVirtual: u64,
    ullAvailExtendedVirtual: u64,
}

extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MemoryStatusEx) -> i32;
}

/// Memory usage monitor
pub struct MemoryMonitor;

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new() -> Self {
        Self
    }

    /// Get current memory usage percentage
    pub fn usage(&self) -> f32 {
        unsafe {
            let mut status = MemoryStatusEx {
                dwLength: std::mem::size_of::<MemoryStatusEx>() as u32,
                dwMemoryLoad: 0,
                ullTotalPhys: 0,
                ullAvailPhys: 0,
                ullTotalPageFile: 0,
                ullAvailPageFile: 0,
                ullTotalVirtual: 0,
                ullAvailVirtual: 0,
                ullAvailExtendedVirtual: 0,
            };

            if GlobalMemoryStatusEx(&mut status) != 0 {
                return status.dwMemoryLoad as f32;
            }
        }

        0.0
    }

    /// Get memory info tuple (usage, description)
    pub fn info(&self) -> (f32, String) {
        let usage = self.usage();
        (usage, format!("{}%", usage.round()))
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}
