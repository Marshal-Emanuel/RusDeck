use sysinfo::Disks;
use std::process::Command;
use std::time::Instant;

static mut LAST_DF_CALL: Option<Instant> = None;
static DF_INTERVAL_SECS: u64 = 10;

pub fn poll_storage(disks: &Disks) -> (f32, f32) {
    let mut total_gb = 0.0;
    let mut available_gb = 0.0;

    for disk in disks.iter() {
        total_gb += disk.total_space() as f32;
        available_gb += disk.available_space() as f32;
    }

    if total_gb == 0.0 {
        return poll_storage_fallback();
    }

    let used_gb = total_gb - available_gb;
    (used_gb / 1_073_741_824.0, total_gb / 1_073_741_824.0)
}

fn poll_storage_fallback() -> (f32, f32) {
    unsafe {
        let now = Instant::now();
        if let Some(last) = LAST_DF_CALL {
            if now.duration_since(last).as_secs() < DF_INTERVAL_SECS {
                return (0.0, 0.0);
            }
        }
        LAST_DF_CALL = Some(now);
    }

    let output = Command::new("df")
        .args(["-B1", "--output=size,avail", "/"])
        .output();

    if let Ok(out) = output {
        let lines = String::from_utf8_lossy(&out.stdout);
        for line in lines.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(size), Ok(av)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                    let total = size;
                    let avail = av;
                    return (avail as f32 / 1_073_741_824.0, total as f32 / 1_073_741_824.0);
                }
            }
        }
    }

    (0.0, 0.0)
}
