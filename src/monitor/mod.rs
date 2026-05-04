pub mod cpu;
pub mod memory;
pub mod network;
pub mod storage;
pub mod temp;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use sysinfo::{System, Networks, CpuRefreshKind, Disks, Components};

use crate::app::AppState;

pub fn start_monitor_thread(state: Arc<RwLock<AppState>>) {
    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new();
        let mut disks = Disks::new();
        let mut components = Components::new();
        loop {
            sys.refresh_cpu_specifics(CpuRefreshKind::everything());
            sys.refresh_memory();
            networks.refresh();
            disks.refresh();
            components.refresh();

            {
                let mut state_guard = state.write().unwrap();

                if let Some(cpu) = sys.cpus().first() {
                    state_guard.system.cpu_usage = cpu.cpu_usage();
                    state_guard.system.cpu_freq_ghz = cpu.frequency() as f32 / 1000.0;
                }

                let (used, total, swap_used, swap_total) = memory::poll_memory(&sys);
                state_guard.system.mem_used_gb = used;
                state_guard.system.mem_total_gb = total;
                state_guard.system.swap_used_gb = swap_used;
                state_guard.system.swap_total_gb = swap_total;

                if let Some(temp) = temp::poll_temp(&components) {
                    state_guard.system.cpu_temp_c = Some(temp);
                }

                let (used, total) = storage::poll_storage(&disks);
                state_guard.system.storage_used_gb = used;
                state_guard.system.storage_total_gb = total;

                let (iface, mac, rx, tx) = network::poll_network(&mut networks);
                state_guard.network.interface = iface;
                state_guard.network.mac = mac;
                state_guard.network.rx_rate = rx;
                state_guard.network.tx_rate = tx;
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });
}
