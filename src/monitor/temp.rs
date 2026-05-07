use sysinfo::Components;
use std::fs;
use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct TempMonitor {
    last_sensors_check: Instant,
    sensors_rate_limit_secs: u64,
    recent_temps: VecDeque<f32>,
}

impl TempMonitor {
    pub fn new() -> Self {
        Self {
            last_sensors_check: Instant::now().checked_sub(Duration::from_secs(30)).unwrap_or_else(Instant::now),
            sensors_rate_limit_secs: 10,
            recent_temps: VecDeque::with_capacity(5),
        }
    }

    pub fn poll(&mut self, components: &Components) -> Option<f32> {
        for component in components.iter() {
            let label = component.label().to_lowercase();
            if label.contains("cpu") || label.contains("core") || label.contains("package") {
                let temp = component.temperature();
                self.cache_temp(temp);
                return Some(temp);
            }
        }

        if let Some(c) = components.iter().next() {
            let temp = c.temperature();
            self.cache_temp(temp);
            return Some(temp);
        }

        self.poll_sysfs()
    }

    fn poll_sysfs(&mut self) -> Option<f32> {
        for zone in 0..10 {
            let path = format!("/sys/class/thermal/thermal_zone{}/temp", zone);
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(temp_milli) = content.trim().parse::<u64>() {
                    let temp = temp_milli as f32 / 1000.0;
                    if temp > 0.0 && temp < 150.0 {
                        self.cache_temp(temp);
                        return Some(temp);
                    }
                }
            }
        }
        None
    }

    fn cache_temp(&mut self, temp: f32) {
        self.recent_temps.push_back(temp);
        if self.recent_temps.len() > 5 {
            self.recent_temps.pop_front();
        }
    }

    pub fn cached_temp(&self) -> Option<f32> {
        if self.recent_temps.is_empty() {
            return None;
        }
        let sum: f32 = self.recent_temps.iter().sum();
        Some(sum / self.recent_temps.len() as f32)
    }
}

impl Default for TempMonitor {
    fn default() -> Self {
        Self::new()
    }
}
