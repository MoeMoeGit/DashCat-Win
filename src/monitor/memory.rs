//! Memory usage monitoring via GlobalMemoryStatusEx

#[repr(C)]
#[allow(non_snake_case)]
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

/// Memory usage monitor
pub struct MemoryMonitor;

impl MemoryMonitor {
    pub fn new() -> Self { Self }

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

            // Call GlobalMemoryStatusEx via FFI
            let result = GlobalMemoryStatusEx(&mut status);
            if result != 0 {
                status.dwMemoryLoad as f32
            } else {
                0.0
            }
        }
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self { Self::new() }
}

#[cfg(windows)]
extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MemoryStatusEx) -> i32;
}