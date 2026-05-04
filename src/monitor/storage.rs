use sysinfo::Disks;
use std::process::Command;

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
    let output = Command::new("df")
        .args(["-B1", "--output=size,avail"])
        .output();

    if let Ok(out) = output {
        let lines = String::from_utf8_lossy(&out.stdout);
        let mut total: u64 = 0;
        let mut avail: u64 = 0;

        for line in lines.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let (Ok(size), Ok(av)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                    total += size;
                    avail += av;
                }
            }
        }

        let used = total - avail;
        return (used as f32 / 1_073_741_824.0, total as f32 / 1_073_741_824.0);
    }

    (0.0, 0.0)
}
