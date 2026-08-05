#![allow(unsafe_code)]
pub mod config;

#[cfg(target_os = "linux")]
use libc::{sched_param, sched_setscheduler, SCHED_FIFO};

/// Safely sets the current thread to the SCHED_FIFO real-time scheduler.
/// On non-Linux platforms or if permissions are lacking, it logs a warning.
pub fn set_rt_thread_priority(priority: i32) {
    #[cfg(target_os = "linux")]
    {
        let param = sched_param {
            sched_priority: priority,
        };
        
        let result = unsafe {
            // 0 means the calling thread
            sched_setscheduler(0, SCHED_FIFO, &param)
        };
        
        if result != 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("Failed to set SCHED_FIFO (requires CAP_SYS_NICE or root): {}", err);
        } else {
            println!("Thread successfully elevated to SCHED_FIFO with priority {}", priority);
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("set_rt_thread_priority is a no-op on non-Linux platforms.");
    }
}
