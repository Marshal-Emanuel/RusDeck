pub mod cpu;
pub mod logs;
pub mod memory;
pub mod network;
pub mod processes;
pub mod storage;
pub mod temp;

use std::thread;
use std::time::Duration;
use std::sync::{Arc, RwLock, mpsc::Sender};
use sysinfo::{System, Networks, CpuRefreshKind, Disks, Components};

use crate::app::{AppState, HISTORY_MAX, ProcessInfo, LogLine};

pub fn start_monitor_thread(state: Arc<RwLock<AppState>>, repaint_tx: Sender<()>) {
    let mut log_buffer = logs::LogBuffer::new(200);
    let mut temp_monitor = temp::TempMonitor::new();

    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new();
        let mut disks = Disks::new();
        let mut components = Components::new();

        let mut prev_rx: f64 = 0.0;
        let mut prev_tx: f64 = 0.0;

        loop {
            sys.refresh_cpu_specifics(CpuRefreshKind::everything());
            sys.refresh_memory();
            networks.refresh();
            disks.refresh();
            components.refresh();
            log_buffer.poll();

            let cpu_usage = sys.cpus().first().map(|c| c.cpu_usage()).unwrap_or(0.0);
            let cpu_freq = sys.cpus().first().map(|c| c.frequency() as f32 / 1000.0).unwrap_or(0.0);
            let load = System::load_average();

            {
                let mut state_guard = state.write().unwrap();

                state_guard.system.cpu_usage = cpu_usage;
                state_guard.system.cpu_freq_ghz = cpu_freq;
                state_guard.system.cpu_load = load.one as f32;
                state_guard.cpu_history.push_back(cpu_usage);
                if state_guard.cpu_history.len() > HISTORY_MAX {
                    state_guard.cpu_history.pop_front();
                }

                state_guard.load_history.push_back(load.one as f32);
                if state_guard.load_history.len() > HISTORY_MAX {
                    state_guard.load_history.pop_front();
                }

                let (used, total, swap_used, swap_total) = memory::poll_memory(&sys);
                state_guard.system.mem_used_gb = used;
                state_guard.system.mem_total_gb = total;
                state_guard.system.swap_used_gb = swap_used;
                state_guard.system.swap_total_gb = swap_total;

                if let Some(temp) = temp_monitor.poll(&components) {
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

                if prev_rx > 0.0 {
                    state_guard.network.rx_history.push_back(rx - prev_rx);
                    state_guard.network.tx_history.push_back(tx - prev_tx);
                    if state_guard.network.rx_history.len() > HISTORY_MAX {
                        state_guard.network.rx_history.pop_front();
                    }
                    if state_guard.network.tx_history.len() > HISTORY_MAX {
                        state_guard.network.tx_history.pop_front();
                    }
                }
                prev_rx = rx;
                prev_tx = tx;

                let process_list = processes::poll_processes(&sys, 12);
                state_guard.processes.clear();
                for p in process_list {
                    state_guard.processes.push(ProcessInfo {
                        pid: p.pid,
                        name: p.name,
                        cpu_pct: p.cpu_pct,
                        mem_pct: p.mem_pct,
                    });
                }

                let recent_logs = log_buffer.get_recent(200);
                state_guard.logs.clear();
                for log in recent_logs {
                    state_guard.logs.push_back(LogLine {
                        timestamp: log.timestamp.clone(),
                        message: log.message.clone(),
                        age: log.age,
                    });
                }
            }

            let _ = repaint_tx.send(());

            thread::sleep(Duration::from_millis(1000));
        }
    });
}
