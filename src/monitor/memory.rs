use sysinfo::System;

pub fn poll_memory(sys: &System) -> (f32, f32, f32, f32) {
    let total = sys.total_memory() as f32 / 1_073_741_824.0;
    let used = sys.used_memory() as f32 / 1_073_741_824.0;
    let total_swap = sys.total_swap() as f32 / 1_073_741_824.0;
    let used_swap = sys.used_swap() as f32 / 1_073_741_824.0;
    (used, total, used_swap, total_swap)
}
