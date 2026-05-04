mod app;
mod monitor;

use std::sync::{Arc, RwLock};
use std::time::Duration;

use app::AppState;
use monitor::start_monitor_thread;

fn main() {
    let state: Arc<RwLock<AppState>> = Arc::new(RwLock::new(AppState::new()));

    start_monitor_thread(state.clone());

    loop {
        let state_guard = state.read().unwrap();

        let cpu = state_guard.system.cpu_usage;
        let freq = state_guard.system.cpu_freq_ghz;
        let temp = state_guard.system.cpu_temp_c;
        let mem_used = state_guard.system.mem_used_gb;
        let mem_total = state_guard.system.mem_total_gb;
        let swap_used = state_guard.system.swap_used_gb;
        let swap_total = state_guard.system.swap_total_gb;
        let storage_used = state_guard.system.storage_used_gb;
        let storage_total = state_guard.system.storage_total_gb;
        let iface = &state_guard.network.interface;
        let rx = state_guard.network.rx_rate;
        let tx = state_guard.network.tx_rate;

        println!("╔══════════════════════════════════════╗");
        println!("║           RUSDECK MONITOR             ║");
        println!("╠══════════════════════════════════════╣");
        println!("║  CPU: {:>6.1}% @ {:.2} GHz           ║", cpu, freq);
        if let Some(t) = temp {
            println!("║  TEMP: {:.1}°C                       ║", t);
        } else {
            println!("║  TEMP: N/A                           ║");
        }
        println!("╠══════════════════════════════════════╣");
        println!("║  RAM:  {:.1} GB / {:.1} GB            ║", mem_used, mem_total);
        println!("║  SWAP: {:.1} GB / {:.1} GB            ║", swap_used, swap_total);
        println!("╠══════════════════════════════════════╣");
        println!("║  DISK: {:.1} GB / {:.1} GB            ║", storage_used, storage_total);
        println!("╠══════════════════════════════════════╣");
        println!("║  NET: {} RX: {} TX: {}  ║", iface, rx, tx);
        println!("╚══════════════════════════════════════╝");

        drop(state_guard);
        std::thread::sleep(Duration::from_secs(1));
    }
}