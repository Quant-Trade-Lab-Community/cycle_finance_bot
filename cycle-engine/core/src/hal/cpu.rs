use core_affinity::CoreId;

pub fn pin_to_core(core_id: usize) {
    if let Some(core_ids) = core_affinity::get_core_ids() {
        if core_id < core_ids.len() {
            let id = core_ids[core_id];
            if core_affinity::set_for_current(id) {
                println!("System PINNED to CPU Core: {}", id.id);
            } else {
                eprintln!("Failed to pin to CPU Core: {}", id.id);
            }
        } else {
            eprintln!("Requested core {} exceeds available cores ({})", core_id, core_ids.len());
        }
    } else {
        eprintln!("Failed to retrieve CPU cores for affinity pinning.");
    }
}
