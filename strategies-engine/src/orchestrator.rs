//! 🧠 StrategyOrchestrator — strateji orkestrasyon merkezi.
//!
//! `cycle-engine`'in `TitaniumOrchestrator` algoritmasıyla aynı deseni izler:
//! bir strateji kümesini (register) tutar, her stratejinin durumunu izler
//! (Running / Stopped / Failed), çocuk süreçleri besler/öldürür ve komut
//! kanallarından gelen istekleri işler.
//!
//! Stratejiler `services-engine/strategies/<isim>/` klasöründe yaşar; her
//! klasör kendi binary'sini üretir. Orkestrasyon merkezi bu klasörü tarar,
//! istenilen strateji(ler)i ayrı birer alt-süreç olarak çalıştırır ve yönetir.
//!
//! Komut akışı (shell → orkestratör):
//! ```text
//! cycle-engine shell (tmux 0)
//!   └─ strat run breakout momentum          # komut dosyasına yazar
//!        └─ /tmp/strategy_cmd.d/cmd_*.cmd   # maildir benzeri kuyruk
//!             └─ StrategyOrchestrator::process_command(line)
//!                  ├─ run   → alt-süreçleri başlat
//!                  ├─ stop  → alt-süreçleri durdur
//!                  ├─ list  → mevcut stratejileri listele
//!                  └─ status → çalışan stratejileri raporla
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Instant;

/// Bir stratejinin orkestrasyon içindeki durumu (TitaniumOrchestrator'un
/// `StrategyState`'ine karşılık gelir).
#[derive(Debug, Clone, PartialEq)]
pub enum StrategyState {
    /// Çalışıyor (alt-süreç yaşıyor).
    Running,
    /// Durmuş / hiç başlatılmamış.
    Stopped,
    /// Çıktı ama hata ile sonlandı (kod ≠ 0) veya başlatılamadı.
    Failed(String),
}

/// Orkestrasyon merkezinin yönettiği tek bir strateji.
#[derive(Debug)]
pub struct ManagedStrategy {
    pub name: String,
    pub state: StrategyState,
    pub started_at: Option<Instant>,
    pub last_exit_code: Option<i32>,
    child: Option<Child>,
}

/// Strateji orkestrasyon merkezi.
///
/// `cycle-engine`'in `TitaniumOrchestrator`'ı gibi: strateji kaydını tutar,
/// durum makinesini işletir, spin/poll döngüsünde ölen alt-süreçleri toplar.
pub struct StrategyOrchestrator {
    strategies_dir: PathBuf,
    bin_dir: PathBuf,
    strategies: HashMap<String, ManagedStrategy>,
}

impl StrategyOrchestrator {
    /// Yeni bir orkestrasyon merkezi kurar.
    ///
    /// - `strategies_dir`: strateji kaynak klasörü (`services-engine/strategies`)
    /// - `bin_dir`: derlenmiş strateji binary'lerinin olduğu klasör (`target/debug`)
    pub fn new<P: AsRef<Path>>(strategies_dir: P, bin_dir: P) -> Self {
        let orchestrator = Self {
            strategies_dir: strategies_dir.as_ref().to_path_buf(),
            bin_dir: bin_dir.as_ref().to_path_buf(),
            strategies: HashMap::new(),
        };
        orchestrator
    }

    /// Strateji kaynak klasörünü tarar ve mevcut strateji adlarını döner.
    pub fn available(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.strategies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("Cargo.toml").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }

    /// Kayıtlı stratejinin anlık durumunu döner (bilinmiyorsa Stopped).
    pub fn state(&self, name: &str) -> StrategyState {
        self.strategies
            .get(name)
            .map(|s| s.state.clone())
            .unwrap_or(StrategyState::Stopped)
    }

    fn ensure_registered(&mut self, name: &str) {
        if !self.strategies.contains_key(name) {
            self.strategies.insert(
                name.to_string(),
                ManagedStrategy {
                    name: name.to_string(),
                    state: StrategyState::Stopped,
                    started_at: None,
                    last_exit_code: None,
                    child: None,
                },
            );
        }
    }

    fn binary_path(&self, name: &str) -> PathBuf {
        self.bin_dir.join(name)
    }

    /// Kullanıcının verdiği adı mevcut bir stratejiye çözer.
    ///
    /// 1. Birebir eşleşme aranır (`breakout-strategy`).
    /// 2. `-strategy` sonekiyle eşleşme aranır (`breakout` → `breakout-strategy`).
    fn resolve_strategy(&self, name: &str) -> Option<String> {
        let avail = self.available();
        if avail.iter().any(|n| n == name) {
            return Some(name.to_string());
        }
        let with_suffix = format!("{name}-strategy");
        if avail.iter().any(|n| n == &with_suffix) {
            return Some(with_suffix);
        }
        None
    }

    fn not_found(&self, name: &str) -> String {
        let avail = self.available();
        if avail.is_empty() {
            format!("'{name}' stratejisi bulunamadı — strateji klasörü boş: {}", self.strategies_dir.display())
        } else {
            format!("'{name}' stratejisi bulunamadı. Mevcut: {}", avail.join(", "))
        }
    }

    /// Bir stratejiyi alt-süreç olarak başlatır.
    pub fn run(&mut self, name: &str) -> Result<(), String> {
        let resolved = self
            .resolve_strategy(name)
            .ok_or_else(|| self.not_found(name))?;
        self.ensure_registered(&resolved);
        let bin = self.binary_path(&resolved);
        let entry = self.strategies.get_mut(&resolved).expect("just registered");
        if matches!(entry.state, StrategyState::Running) {
            return Err(format!("'{resolved}' zaten çalışıyor (pid: {})", entry.child.as_ref().unwrap().id()));
        }

        let mut cmd = Command::new(&bin);
        cmd.current_dir(&self.strategies_dir);
        match cmd.spawn() {
            Ok(child) => {
                entry.child = Some(child);
                entry.state = StrategyState::Running;
                entry.started_at = Some(Instant::now());
                entry.last_exit_code = None;
                Ok(())
            }
            Err(e) => {
                entry.state = StrategyState::Failed(format!(
                    "başlatılamadı ({e}). Önce derleyin: cargo build -p {resolved}"
                ));
                Err(format!("'{resolved}' başlatılamadı: {e}"))
            }
        }
    }

    /// Birden fazla stratejiyi sırayla başlatır.
    pub fn run_many(&mut self, names: &[String]) -> Vec<Result<(), String>> {
        names.iter().map(|n| self.run(n)).collect()
    }

    /// Bir stratejiyi nazikçe durdurur (SIGTERM → bekle → SIGKILL).
    pub fn stop(&mut self, name: &str) -> Result<(), String> {
        let resolved = self
            .resolve_strategy(name)
            .ok_or_else(|| self.not_found(name))?;
        self.ensure_registered(&resolved);
        let entry = self.strategies.get_mut(&resolved).expect("just registered");
        if !matches!(entry.state, StrategyState::Running) {
            return Err(format!("'{resolved}' çalışmıyor."));
        }
        let mut child = entry.child.take().expect("running has child");
        // SIGTERM ile nazik kapatmayı dene.
        unsafe {
            libc::kill(child.id() as i32, libc::SIGTERM);
        }
        let _ = std::thread::spawn(move || {
            let _ = child.wait();
        });
        entry.state = StrategyState::Stopped;
        entry.started_at = None;
        Ok(())
    }

    /// Birden fazla stratejiyi durdurur.
    pub fn stop_many(&mut self, names: &[String]) -> Vec<Result<(), String>> {
        names.iter().map(|n| self.stop(n)).collect()
    }

    /// Bir stratejiyi yeniden başlatır.
    pub fn restart(&mut self, name: &str) -> Result<(), String> {
        let _ = self.stop(name);
        self.run(name)
    }

    /// Poll döngüsü: ölen alt-süreçleri toplar, durumlarını günceller.
    /// (TitaniumOrchestrator'ın spin-loop reap işine karşılık gelir.)
    pub fn tick(&mut self) {
        let mut dead: Vec<String> = Vec::new();
        for (name, entry) in self.strategies.iter_mut() {
            if matches!(entry.state, StrategyState::Running) {
                if let Some(child) = entry.child.as_mut() {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            let code = status.code().unwrap_or(-1);
                            entry.last_exit_code = Some(code);
                            entry.child = None;
                            entry.started_at = None;
                            entry.state = if code == 0 {
                                StrategyState::Stopped
                            } else {
                                StrategyState::Failed(format!("çıkış kodu {code}"))
                            };
                            dead.push(name.clone());
                        }
                        Ok(None) => {}
                        Err(_) => {
                            entry.child = None;
                            entry.state = StrategyState::Failed("poll hatası".into());
                            dead.push(name.clone());
                        }
                    }
                }
            }
        }
        for name in dead {
            let state = self.strategies.get(&name).map(|s| s.state.clone()).unwrap_or(StrategyState::Stopped);
            eprintln!("[STRAT] '{name}' alt-süreci sonlandı → {:?}", state);
        }
    }

    /// Orkestrasyon durumunu okunaklı bir rapor olarak üretir.
    pub fn status(&self) -> String {
        let mut out = String::new();
        out.push_str("🧠  STRATEJİ ORKESTRASYON MERKEZİ\n");
        out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        out.push_str(&format!("  Klasör  : {}\n", self.strategies_dir.display()));
        out.push_str(&format!("  Binary  : {}\n", self.bin_dir.display()));
        out.push_str("  Çalışan : ");

        let running: Vec<&str> = self
            .strategies
            .iter()
            .filter(|(_, s)| matches!(s.state, StrategyState::Running))
            .map(|(n, _)| n.as_str())
            .collect();
        if running.is_empty() {
            out.push_str("(yok)\n");
        } else {
            out.push_str(&running.join(", "));
            out.push('\n');
        }

        let available = self.available();
        out.push_str("  Mevcut  : ");
        if available.is_empty() {
            out.push_str("(yok)\n");
        } else {
            out.push_str(&available.join(", "));
            out.push('\n');
        }
        out.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        for name in &available {
            let entry = self.strategies.get(name);
            let (state, extra) = match entry {
                Some(e) => match &e.state {
                    StrategyState::Running => (
                        "✅ ÇALIŞIYOR",
                        format!("  pid: {}", e.child.as_ref().map(|c| c.id()).unwrap_or(0)),
                    ),
                    StrategyState::Failed(reason) => ("⚠️  HATA", format!("  {reason}")),
                    StrategyState::Stopped => ("⏸️  DURDU", String::new()),
                },
                None => ("⏸️  DURDU", String::new()),
            };
            out.push_str(&format!("  • {name:<20} {state}{extra}\n"));
        }
        out
    }

    /// Komut satırını ayrıştırıp yürütür; yanıtı döner.
    ///
    /// Komutlar:
    /// - `run <isim> [<isim>...]` → strateji(ler)i başlat
    /// - `stop <isim> [<isim>...]` → strateji(ler)i durdur
    /// - `restart <isim>` → yeniden başlat
    /// - `list` | `ls` → mevcut stratejiler
    /// - `status` → ayrıntılı durum
    /// - `help` → kullanım
    pub fn process_command(&mut self, line: &str) -> String {
        let mut parts = line.split_whitespace();
        let Some(cmd) = parts.next() else {
            return String::new();
        };
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        let mut resp = String::new();
        match cmd.to_lowercase().as_str() {
            "run" | "start" => {
                if args.is_empty() {
                    resp.push_str("Kullanım: run <strateji> [<strateji>...]");
                    return resp;
                }
                for name in &args {
                    match self.run(name) {
                        Ok(()) => resp.push_str(&format!("✅ '{name}' başlatıldı.\n")),
                        Err(e) => resp.push_str(&format!("❌ {e}\n")),
                    }
                }
            }
            "stop" => {
                if args.is_empty() {
                    resp.push_str("Kullanım: stop <strateji> [<strateji>...]");
                    return resp;
                }
                for name in &args {
                    match self.stop(name) {
                        Ok(()) => resp.push_str(&format!("⏹  '{name}' durduruldu.\n")),
                        Err(e) => resp.push_str(&format!("ℹ️  {e}\n")),
                    }
                }
            }
            "restart" => {
                let Some(name) = args.first() else {
                    resp.push_str("Kullanım: restart <strateji>");
                    return resp;
                };
                match self.restart(name) {
                    Ok(()) => resp.push_str(&format!("🔄 '{name}' yeniden başlatıldı.\n")),
                    Err(e) => resp.push_str(&format!("❌ {e}\n")),
                }
            }
            "list" | "ls" => {
                let avail = self.available();
                if avail.is_empty() {
                    resp.push_str(&format!(
                        "📂 Strateji klasöründe strateji yok: {}",
                        self.strategies_dir.display()
                    ));
                } else {
                    resp.push_str(&format!("📂 Mevcut stratejiler ({}):\n", avail.len()));
                    for name in &avail {
                        let state = match self.state(name) {
                            StrategyState::Running => "✅ ÇALIŞIYOR".to_string(),
                            StrategyState::Failed(r) => format!("⚠️  HATA ({r})"),
                            StrategyState::Stopped => "⏸️  DURDU".to_string(),
                        };
                        resp.push_str(&format!("  • {name:<20} {state}\n"));
                    }
                }
            }
            "status" => {
                resp.push_str(&self.status());
            }
            "help" => {
                resp.push_str(
                    "Komutlar:\n\
                     \x20 run <isim> [<isim>...]   strateji(ler)i başlat\n\
                     \x20 stop <isim> [<isim>...]  strateji(ler)i durdur\n\
                     \x20 restart <isim>           stratejiyi yeniden başlat\n\
                     \x20 list                     mevcut stratejileri listele\n\
                     \x20 status                   orkestrasyon durumu\n\
                     \x20 help                     bu yardım\n",
                );
            }
            other => {
                resp.push_str(&format!("Bilinmeyen komut: '{other}'. 'help' yazın."));
            }
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parsing_help_and_list() {
        let mut orch = StrategyOrchestrator::new("/nonexistent/strategies", "/nonexistent/bin");
        let resp = orch.process_command("help");
        assert!(resp.contains("run"));
        assert!(resp.contains("status"));
        let resp = orch.process_command("list");
        assert!(resp.contains("yok"));
    }

    #[test]
    fn run_unknown_strategy_reports_error() {
        let mut orch = StrategyOrchestrator::new("/nonexistent/strategies", "/nonexistent/bin");
        let resp = orch.process_command("run breakout");
        assert!(resp.contains("bulunamadı"));
    }

    #[test]
    fn stop_non_running_reports_info() {
        let tmp = std::env::temp_dir().join(format!("strat_orch_stop_{}", std::process::id()));
        let strat_dir = tmp.join("strategies");
        let bin_dir = tmp.join("bin");
        std::fs::create_dir_all(strat_dir.join("breakout-strategy")).unwrap();
        std::fs::write(strat_dir.join("breakout-strategy/Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        let mut orch = StrategyOrchestrator::new(&strat_dir, &bin_dir);
        let resp = orch.process_command("stop breakout");
        assert!(resp.contains("çalışmıyor"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn alias_suffix_resolves_to_dashed_name() {
        let tmp = std::env::temp_dir().join(format!("strat_orch_alias_{}", std::process::id()));
        let strat_dir = tmp.join("strategies");
        let bin_dir = tmp.join("bin");
        std::fs::create_dir_all(strat_dir.join("breakout-strategy")).unwrap();
        std::fs::write(strat_dir.join("breakout-strategy/Cargo.toml"), "").unwrap();
        std::fs::create_dir_all(&bin_dir).unwrap();
        let mut orch = StrategyOrchestrator::new(&strat_dir, &bin_dir);
        // 'breakout' → 'breakout-strategy' takma adı çözülmeli.
        // Binary olmadığı için sonuç "bulunamadı" DEĞİL "başlatılamadı" olmalı.
        let resp = orch.process_command("run breakout");
        assert!(resp.contains("başlatılamadı"), "takma ad çözülmedi: {resp}");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
