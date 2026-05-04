pub mod cpu;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use sysinfo::{System, RefreshKind};

use crate::app::AppState;

pub fn start_monitor_thread(state: Arc<RwLock<AppState>>) {
    thread::spawn(move || {
        let mut sys = System::new_all();
        loop {
            sys.refresh_cpu();
            sys.refresh_memory();
            
            {
                let mut state_guard = state.write().unwrap();
                
                // Update CPU
                if let Some(cpu) = sys.cpus().first() {
                    state_guard.system.cpu_usage = cpu.cpu_usage();
                    state_guard.system.cpu_freq_ghz = cpu.frequency() as f32 / 1000.0;
                }
                
                // Update memory
                let total_mem = sys.total_memory() as f32;
                let used_mem = sys.used_memory() as f32;
                state_guard.system.mem_used_gb = used_mem / 1_073_741_824.0;
                state_guard.system.mem_total_gb = total_mem / 1_073_741_824.0;
                
                // Update swap
                let total_swap = sys.total_swap() as f32;
                let used_swap = sys.used_swap() as f32;
                state_guard.system.swap_used_gb = used_swap / 1_073_741_824.0;
                state_guard.system.swap_total_gb = total_swap / 1_073_741_824.0;
            }
            
            thread::sleep(Duration::from_millis(1000));
        }
    });
}
