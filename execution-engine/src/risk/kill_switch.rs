//! Kill switch — dosya + bayrak tabanlı acil durdurma.
//!
//! Dosya varsa (veya bayrak açıksa) tüm yazma işlemleri reddedilir.
//! Opsel müdahale: `touch /tmp/exec_kill_switch`, REST veya CLI.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct KillSwitch {
    path: String,
    /// Dosyadan bağımsız yerel bayrak (REST/CLI ile kontrol).
    flag: AtomicBool,
}

impl KillSwitch {
    pub fn new(path: String) -> Self {
        Self {
            path,
            flag: AtomicBool::new(false),
        }
    }

    pub fn is_open(&self) -> bool {
        self.flag.load(Ordering::Relaxed) || Path::new(&self.path).exists()
    }

    /// Acil durum bayrağını açar ve dosyayı yazar.
    pub fn engage(&self) -> std::io::Result<()> {
        self.flag.store(true, Ordering::Relaxed);
        std::fs::write(&self.path, b"KILL SWITCH ENGAGED\n")?;
        Ok(())
    }

    /// Bayrağı ve dosyayı kaldırır (yalnızca bilinçli kararla).
    pub fn release(&self) -> std::io::Result<()> {
        self.flag.store(false, Ordering::Relaxed);
        let _ = std::fs::remove_file(&self.path);
        Ok(())
    }

    pub fn engaged_by_flag(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}
