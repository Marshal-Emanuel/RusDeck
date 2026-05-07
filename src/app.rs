use std::collections::VecDeque;

pub const HISTORY_MAX: usize = 20;

pub struct AppState {
    pub system: SystemData,
    pub network: NetworkData,
    pub processes: Vec<ProcessInfo>,
    pub logs: VecDeque<LogLine>,
    pub cpu_history: VecDeque<f32>,
    pub load_history: VecDeque<f32>,
}

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_pct: f32,
}

pub struct LogLine {
    pub timestamp: String,
    pub message: String,
}

pub struct SystemData {
    pub cpu_usage: f32,
    pub cpu_freq_ghz: f32,
    pub cpu_load: f32,
    pub cpu_temp_c: Option<f32>,
    pub mem_used_gb: f32,
    pub mem_total_gb: f32,
    pub swap_used_gb: f32,
    pub swap_total_gb: f32,
    pub storage_total_gb: f32,
    pub storage_used_gb: f32,
}

pub struct NetworkData {
    pub interface: String,
    pub ip: String,
    pub mac: String,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rx_history: VecDeque<f64>,
    pub tx_history: VecDeque<f64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            system: SystemData {
                cpu_usage: 0.0,
                cpu_freq_ghz: 0.0,
                cpu_load: 0.0,
                cpu_temp_c: None,
                mem_used_gb: 0.0,
                mem_total_gb: 0.0,
                swap_used_gb: 0.0,
                swap_total_gb: 0.0,
                storage_total_gb: 0.0,
                storage_used_gb: 0.0,
            },
            network: NetworkData {
                interface: "eth0".to_string(),
                ip: "0.0.0.0".to_string(),
                mac: "00:00:00:00:00:00".to_string(),
                rx_rate: 0.0,
                tx_rate: 0.0,
                rx_history: VecDeque::with_capacity(HISTORY_MAX),
                tx_history: VecDeque::with_capacity(HISTORY_MAX),
            },
            processes: Vec::with_capacity(12),
            logs: VecDeque::with_capacity(200),
            cpu_history: VecDeque::with_capacity(HISTORY_MAX),
            load_history: VecDeque::with_capacity(HISTORY_MAX),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
