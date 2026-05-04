use sysinfo::Networks;

pub fn poll_network(networks: &mut Networks) -> (String, String, f64, f64) {
    networks.refresh_list();
    
    let mut total_rx = 0;
    let mut total_tx = 0;
    let mut interface_name = String::new();
    let mut mac = String::new();
    
    for (name, network) in networks.iter() {
        interface_name = name.clone();
        mac = "00:00:00:00:00:00".to_string(); // sys-info doesn't provide MAC directly
        total_rx += network.total_received();
        total_tx += network.total_transmitted();
    }
    
    (interface_name, mac, total_rx as f64, total_tx as f64)
}
