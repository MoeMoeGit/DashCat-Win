//! System monitoring module

pub mod cpu;
pub mod memory;

pub use cpu::CpuMonitor;
pub use memory::MemoryMonitor;

/// Combined system stats
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

impl SystemStats {
    /// Get current system stats
    pub fn get(cpu: &mut CpuMonitor, memory: &MemoryMonitor) -> Self {
        Self {
            cpu_usage: cpu.usage(),
            memory_usage: memory.usage(),
        }
    }
}