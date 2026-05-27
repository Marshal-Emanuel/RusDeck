use sysinfo::Networks;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::collections::HashMap;

static PREV_STATS: OnceLock<Mutex<HashMap<String, (u64, u64)>>> = OnceLock::new();

pub fn poll_network(networks: &mut Networks) -> (String, String, f64, f64) {
    networks.refresh_list();
    
    let stats_mutex = PREV_STATS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut prev_stats = stats_mutex.lock().unwrap();

    let mut best_interface = String::new();
    let mut best_rx_rate = 0.0;
    let mut best_tx_rate = 0.0;

    for (name, network) in networks.iter() {
        if name == "lo" {
            continue;
        }
        let rx_total = network.total_received();
        let tx_total = network.total_transmitted();
        
        let (prev_rx, prev_tx) = prev_stats.entry(name.clone()).or_insert((rx_total, tx_total));
        
        // Calculate difference (bytes transferred since last poll)
        let rx_rate = rx_total.saturating_sub(*prev_rx) as f64;
        let tx_rate = tx_total.saturating_sub(*prev_tx) as f64;
        
        *prev_rx = rx_total;
        *prev_tx = tx_total;

        // Prioritize physical adapters (wlp, enp, eth) or choose the one with the highest active traffic
        let is_primary = name.starts_with("wlp") || name.starts_with("enp") || name.starts_with("eth") || name.starts_with("wlan") || name.starts_with("eno");
        if best_interface.is_empty() || (is_primary && !best_interface.starts_with("wlp") && !best_interface.starts_with("enp") && !best_interface.starts_with("eth")) || rx_rate > best_rx_rate {
            best_interface = name.clone();
            best_rx_rate = rx_rate;
            best_tx_rate = tx_rate;
        }
    }

    if best_interface.is_empty() {
        best_interface = "eth0".to_string();
    }

    (best_interface, "00:00:00:00:00:00".to_string(), best_rx_rate, best_tx_rate)
}
