use crossbeam::epoch::{self, Atomic, Owned};
use std::sync::atomic::Ordering;

/// System configuration using lock-free epoch-based reclamation.
/// Prevents use-after-free without using Mutex/RwLock in the tick loop.
pub struct GlobalConfig {
    pub max_positions: usize,
    pub active_api_version: &'static str,
}

pub struct ConfigManager {
    // crossbeam_epoch::Atomic provides safe, lock-free memory reclamation
    current_config: Atomic<GlobalConfig>,
}

impl ConfigManager {
    pub fn new(initial: GlobalConfig) -> Self {
        Self {
            current_config: Atomic::new(initial),
        }
    }

    /// Read configuration. The returned guard ensures the config is not dropped
    /// while the current thread is holding it (epoch pinning).
    pub fn read_config<'a>(&'a self, guard: &'a epoch::Guard) -> &'a GlobalConfig {
        let ptr = self.current_config.load(Ordering::Acquire, guard);
        unsafe { ptr.as_ref().unwrap() }
    }

    /// Swap configuration globally. Old config is queued for garbage collection
    /// once no threads are pinning the epoch.
    pub fn swap_config(&self, new_config: GlobalConfig) {
        let guard = epoch::pin();
        let new_ptr = Owned::new(new_config);
        
        let old_ptr = self.current_config.swap(new_ptr, Ordering::Release, &guard);
        
        if !old_ptr.is_null() {
            unsafe {
                // Queue the old configuration for deletion safely.
                guard.defer_destroy(old_ptr);
            }
        }
        println!("Config: Successfully swapped lock-free configuration.");
    }
}
