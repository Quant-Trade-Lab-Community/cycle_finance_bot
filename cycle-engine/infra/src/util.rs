//! Dayanıklılık yardımcıları — cycle-engine mimari deseni.
//!
//! Bu modül, tüm engine/servislerin ortak kullandığı "çökme-üretmeyen"
//! başlatma kurallarını sağlar:
//!
//! 1. `bind_or_exit` — port bağlama hatasında panic yerine net mesaj + temiz çıkış.
//! 2. `single_instance` — ikiz süreç önleme (çift süreç → çift yazma/port çakışması).
//!
//! Felsefe: dış kaynak (ağ, port, dosya) hataları `panic!` değil, açık bir
//! hata + kontrollü çıkış üretir. Systemd `Restart=always` ile ayağa kaldırır,
//! panic-loop yerine tek örnek garantisi verir.

use std::io::Write;

/// TCP listener'ı bağlar. Hata durumunda (ör. port dolu) panic YERİNE net bir
/// mesaj basar ve `exit(1)` ile temiz çıkar — systemd Restart=always yeniden dener.
pub async fn bind_or_exit(
    addr: impl tokio::net::ToSocketAddrs + std::fmt::Debug,
    service: &str,
) -> tokio::net::TcpListener {
    let addr_debug = format!("{addr:?}");
    match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            let mut err = std::io::stderr().lock();
            let _ = writeln!(err, "[{service}] ❌ {addr_debug} bağlanamadı: {e}");
            let _ = writeln!(err, "[{service}]   Bu servis zaten çalışıyor olabilir (tek örnek koruması uygulandı mı?)");
            let _ = writeln!(err, "[{service}]   Kontrol: systemctl --user list-units 'cycle-*' | pgrep -af {service}");
            std::process::exit(1);
        }
    }
}

/// Tek örnek koruması — `lock_name` için `flock` tabanlı kilit dosyası alır.
///
/// İkinci bir örnek başlatılırsa (ör. elle + systemd aynı anda) net mesajla
/// çıkar; böylece çift süreç → çift yazma / port çakışması engellenir.
///
/// Kilit, süreç yaşadığı sürece tutulur (fd açık bırakılır).
pub fn single_instance(lock_name: &str) -> std::io::Result<()> {
    let path = format!("/tmp/cycle_{lock_name}.lock");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&path)?;

    use std::os::unix::io::AsRawFd;
    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        let e = std::io::Error::last_os_error();
        if e.raw_os_error() == Some(libc::EWOULDBLOCK) || e.raw_os_error() == Some(libc::EAGAIN) {
            let mut err = std::io::stderr().lock();
            let _ = writeln!(err, "⚠️  {lock_name} zaten çalışıyor (tek örnek koruması) — bu süreç çıkıyor.");
            std::process::exit(1);
        }
        return Err(e);
    }
    // fd açık kalsın ki kilit sürecin ömrü boyunca sürsün.
    std::mem::forget(file);
    Ok(())
}
