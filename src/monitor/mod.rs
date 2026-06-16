pub mod cpu;
pub mod logs;
pub mod memory;
pub mod network;
pub mod processes;
pub mod storage;
pub mod temp;

use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};
use sysinfo::{System, Networks, CpuRefreshKind, Disks, Components};

use crate::app::{AppState, HISTORY_MAX, ProcessInfo, LogLine};

const REFRESH_CPU: u64 = 1;
const REFRESH_MEMORY: u64 = 3;
const REFRESH_DISK: u64 = 10;
const REFRESH_TEMP: u64 = 5;
const REFRESH_NETWORK: u64 = 1;
const REFRESH_PROCESSES: u64 = 2;


pub fn start_monitor_thread(state: Arc<RwLock<AppState>>, ctx: egui::Context) {
    let mut log_buffer = logs::LogBuffer::new(200);
    let mut temp_monitor = temp::TempMonitor::new();

    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut networks = Networks::new();
        let mut disks = Disks::new();
        let mut components = Components::new();

        let start = Instant::now();
        let mut last_cpu = start;
        let mut last_memory = start;
        let mut last_disk = start;
        let mut last_temp = start;
        let mut last_network = start;
        let mut last_processes = start;


        // Perform initial reads to populate immediately on startup
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        sys.refresh_memory();
        networks.refresh();
        disks.refresh();
        components.refresh();
        sys.refresh_processes();
        // Initial seed: drain whatever backlog journalctl -f emits immediately
        log_buffer.poll();

        loop {
            let now = Instant::now();
            let mut state_changed = false;

            // 1. CPU (1s)
            if now.duration_since(last_cpu) >= Duration::from_secs(REFRESH_CPU) {
                let cpu_usage = cpu::poll_cpu(&mut sys);
                let cpu_freq = cpu::get_cpu_freq(&sys);
                last_cpu = now;

                let load = System::load_average();

                if let Ok(mut state_guard) = state.write() {
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
                }
                state_changed = true;
            }

            // 2. Network (1s)
            if now.duration_since(last_network) >= Duration::from_secs(REFRESH_NETWORK) {
                networks.refresh();
                last_network = now;

                let (iface, mac, rx_rate, tx_rate) = network::poll_network(&mut networks);

                if let Ok(mut state_guard) = state.write() {
                    state_guard.network.interface = iface;
                    state_guard.network.mac = mac;
                    state_guard.network.rx_rate = rx_rate;
                    state_guard.network.tx_rate = tx_rate;

                    state_guard.network.rx_history.push_back(rx_rate);
                    state_guard.network.tx_history.push_back(tx_rate);
                    if state_guard.network.rx_history.len() > HISTORY_MAX {
                        state_guard.network.rx_history.pop_front();
                    }
                    if state_guard.network.tx_history.len() > HISTORY_MAX {
                        state_guard.network.tx_history.pop_front();
                    }
                }
                state_changed = true;
            }

            // 3. Processes (2s)
            if now.duration_since(last_processes) >= Duration::from_secs(REFRESH_PROCESSES) {
                sys.refresh_processes();
                last_processes = now;

                let process_list = processes::poll_processes(&sys, 12);

                if let Ok(mut state_guard) = state.write() {
                    state_guard.processes.clear();
                    for p in &process_list {
                        state_guard.processes.push(ProcessInfo {
                            pid: p.pid,
                            name: p.name.clone(),
                            cpu_pct: p.cpu_pct,
                            mem_pct: p.mem_pct,
                        });
                    }
                }
                state_changed = true;
            }

            // 4. Memory (3s)
            if now.duration_since(last_memory) >= Duration::from_secs(REFRESH_MEMORY) {
                sys.refresh_memory();
                last_memory = now;

                let (used, total, swap_used, swap_total) = memory::poll_memory(&sys);

                if let Ok(mut state_guard) = state.write() {
                    state_guard.system.mem_used_gb = used;
                    state_guard.system.mem_total_gb = total;
                    state_guard.system.swap_used_gb = swap_used;
                    state_guard.system.swap_total_gb = swap_total;
                }
                state_changed = true;
            }

            // 5. Temperature (5s)
            if now.duration_since(last_temp) >= Duration::from_secs(REFRESH_TEMP) {
                components.refresh();
                last_temp = now;

                let cpu_temp = temp_monitor.poll(&components);

                if let Some(temp) = cpu_temp {
                    if let Ok(mut state_guard) = state.write() {
                        state_guard.system.cpu_temp_c = Some(temp);
                    }
                    state_changed = true;
                }
            }

            // 6. Disk Storage (10s)
            if now.duration_since(last_disk) >= Duration::from_secs(REFRESH_DISK) {
                disks.refresh();
                last_disk = now;

                let (disk_used, disk_total) = storage::poll_storage(&disks);

                if let Ok(mut state_guard) = state.write() {
                    state_guard.system.storage_used_gb = disk_used;
                    state_guard.system.storage_total_gb = disk_total;
                }
                state_changed = true;
            }

            // 7. System Logs — non-blocking channel drain (runs every loop tick, ~200ms)
            {
                log_buffer.poll();
                let recent_logs = log_buffer.get_recent(200);

                if let Ok(mut state_guard) = state.write() {
                    state_guard.logs.clear();
                    for log in recent_logs {
                        state_guard.logs.push_back(LogLine {
                            timestamp: log.timestamp.clone(),
                            message: log.message.clone(),
                        });
                    }
                }
                state_changed = true;
            }

            if state_changed {
                ctx.request_repaint();
            }

            thread::sleep(Duration::from_millis(200));
        }
    });
}
