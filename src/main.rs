mod app;
mod monitor;

use std::sync::{Arc, RwLock};

fn main() {
    let state = Arc::new(RwLock::new(app::AppState::new()));
    
    // Start monitor thread
    monitor::start_monitor_thread(state.clone());
    
    // For now, just print CPU usage
    loop {
        let usage = state.read().unwrap().system.cpu_usage;
        println!("CPU: {:.1}%", usage);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
