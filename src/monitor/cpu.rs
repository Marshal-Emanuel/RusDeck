use sysinfo::{CpuRefreshKind, System};

pub fn poll_cpu(sys: &mut System) -> f32 {
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    
    if let Some(cpu) = sys.cpus().first() {
        cpu.cpu_usage() // Returns 0.0 - 100.0
    } else {
        0.0
    }
}

pub fn get_cpu_freq(sys: &System) -> f32 {
    if let Some(cpu) = sys.cpus().first() {
        cpu.frequency() as f32 / 1000.0 // Convert MHz to GHz
    } else {
        0.0
    }
}
