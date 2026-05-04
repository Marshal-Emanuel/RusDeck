use sysinfo::Components;
use std::fs;

pub fn poll_temp(components: &Components) -> Option<f32> {
    for component in components.iter() {
        let label = component.label().to_lowercase();
        if label.contains("cpu") || label.contains("core") || label.contains("package") {
            return Some(component.temperature());
        }
    }

    if let Some(c) = components.iter().next() {
        return Some(c.temperature());
    }

    poll_temp_fallback()
}

fn poll_temp_fallback() -> Option<f32> {
    if let Ok(content) = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(temp_milli) = content.trim().parse::<u64>() {
            return Some(temp_milli as f32 / 1000.0);
        }
    }

    let output = std::process::Command::new("sensors")
        .output().ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains("CPU") && line.contains("+") {
            if let Some(temp_str) = line.split_whitespace().find(|s| s.contains("°C")) {
                let temp_clean = temp_str.replace("°C", "").replace("+", "");
                if let Ok(t) = temp_clean.trim().parse::<f32>() {
                    return Some(t);
                }
            }
        }
    }

    None
}
