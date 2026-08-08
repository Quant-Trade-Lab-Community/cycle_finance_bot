#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_rdtsc;

pub struct TscTimer {
    start_tsc: u64,
    tsc_hz: f64,
}

impl TscTimer {
    pub fn new() -> Self {
        // Estimate TSC frequency (Simplified)
        // In a real HFT system, we calibrate this against a reliable clock for 1-2 seconds at startup.
        let hz = 3_000_000_000.0; // Assume 3 GHz for now
        
        let start_tsc = Self::read_tsc();
        Self {
            start_tsc,
            tsc_hz: hz,
        }
    }

    #[inline(always)]
    pub fn read_tsc() -> u64 {
        #[cfg(target_arch = "x86_64")]
        unsafe { _rdtsc() }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback for ARM/Mac
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
        }
    }

    #[inline(always)]
    pub fn elapsed_ns(&self) -> u64 {
        let current = Self::read_tsc();
        if current > self.start_tsc {
            let diff = current - self.start_tsc;
            ((diff as f64 / self.tsc_hz) * 1_000_000_000.0) as u64
        } else {
            0
        }
    }
}
