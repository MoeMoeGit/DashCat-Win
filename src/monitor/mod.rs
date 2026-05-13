//! System monitoring module (CPU/Memory)

mod cpu;
mod memory;

pub use cpu::CpuMonitor;
pub use memory::MemoryMonitor;

/// Monitor information tuple
pub type MonitorInfo = (f32, String);
