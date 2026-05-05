use sysinfo::System;

pub struct Process {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_pct: f32,
}

pub fn poll_processes(sys: &System, max_count: usize) -> Vec<Process> {
    let mut processes: Vec<Process> = sys.processes().iter()
        .map(|(pid, process)| {
            let cpu_pct = process.cpu_usage();
            let mem_pct = if sys.total_memory() > 0 {
                (process.memory() as f64 / sys.total_memory() as f64 * 100.0) as f32
            } else {
                0.0
            };

            Process {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_pct,
                mem_pct,
            }
        })
        .collect();

    processes.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());
    processes.truncate(max_count);

    processes
}