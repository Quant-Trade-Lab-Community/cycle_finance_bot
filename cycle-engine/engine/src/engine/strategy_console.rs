//! 🧠 STRATEGY konsolu — cycle-engine üzerinde strateji orkestrasyon merkezi.
//!
//! Ayrı bir binary olarak (`strategy-console`) çalışır; `engine` binary'si
//! saf DATA konsoludur ve bundan etkilenmez. `StrategyOrchestrator`'ı barındırır
//! ve iki giriş kanalını dinler:
//!
//! 1. **Interaktif stdin** (rustyline) — STRATEGY sekmesinde elle komut.
//! 2. **Komut kuyruğu** `/tmp/strategy_cmd.d/*.cmd` — cycle-engine shell'den
//!    (`strat run breakout ...`) gönderilen komutlar.
//!
//! Örnek:
//! ```bash
//! ./target/debug/strategy-console
//! ```
//! Konsolda: `run breakout`, `stop breakout`, `status`, `list`, `help`

use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::time::Duration;
use strategies_engine::StrategyOrchestrator;

const CMD_DIR_DEFAULT: &str = "/tmp/strategy_cmd.d";
const STATUS_FILE_DEFAULT: &str = "/tmp/strategy_status.txt";

enum Input {
    Command(String),
    Exit,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Komut kuyruğundaki (`/tmp/strategy_cmd.d`) her `.cmd` dosyasını okuyup
/// kanala basar ve dosyayı siler. Ayrı bir iş parçacığında çalışır.
fn spawn_cmd_file_poller(cmd_dir: PathBuf, tx: Sender<Input>) {
    std::thread::spawn(move || loop {
        if let Ok(entries) = std::fs::read_dir(&cmd_dir) {
            let mut files: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().map(|e| e == "cmd").unwrap_or(false))
                .collect();
            files.sort();
            for file in files {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    let _ = std::fs::remove_file(&file);
                    for line in content.lines() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }
                        if tx.send(Input::Command(line.to_string())).is_err() {
                            return;
                        }
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    });
}

/// Interaktif stdin'i rustyline ile okur ve kanala basar.
fn spawn_stdin_reader(tx: Sender<Input>) {
    std::thread::spawn(move || {
        let mut rl = match rustyline::DefaultEditor::new() {
            Ok(rl) => rl,
            Err(_) => return,
        };
        loop {
            match rl.readline("strategy> ") {
                Ok(line) => {
                    let line = line.trim().to_string();
                    let _ = rl.add_history_entry(line.as_str());
                    if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
                        let _ = tx.send(Input::Exit);
                        break;
                    }
                    if tx.send(Input::Command(line)).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    let _ = tx.send(Input::Exit);
                    break;
                }
            }
        }
    });
}

/// STRATEGY konsolunu çalıştırır (sonsuz döngü).
pub fn run_strategy_console() {
    let root = std::env::var("CYCLE_ROOT").unwrap_or_else(|_| {
        std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    let root = PathBuf::from(root);
    let strategies_dir = root.join("services-engine/strategies");
    let bin_dir = root.join("target/debug");

    let mut orch = StrategyOrchestrator::new(&strategies_dir, &bin_dir);

    let cmd_dir = PathBuf::from(env_or("STRATEGY_CMD_DIR", CMD_DIR_DEFAULT));
    let status_file = env_or("STRATEGY_STATUS_FILE", STATUS_FILE_DEFAULT);
    let _ = std::fs::create_dir_all(&cmd_dir);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠  STRATEJİ ORKESTRASYON MERKEZİ  (cycle-engine)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Strateji klasörü : {}", strategies_dir.display());
    println!("  Binary klasörü   : {}", bin_dir.display());
    println!("  Komut kuyruğu    : {}", cmd_dir.display());
    println!("  Shell'den yönet  : strat run breakout | strat stop breakout | strat status");
    println!("  Konsolda         : run/stop/list/status/help");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let (tx, rx) = channel::<Input>();
    spawn_cmd_file_poller(cmd_dir, tx.clone());
    spawn_stdin_reader(tx);

    // Konsol etkileşimli olduğundan stdin değil kuyruk dinlenir; STRATEGY
    // sekmesine `tmux send-keys` ile komut göndermek için stdin okuyucu yeterli.
    // Ana döngü: kanal + periyodik tick (ölen alt-süreçleri toplar).
    let mut last_tick = std::time::Instant::now();
    loop {
        match rx.try_recv() {
            Ok(Input::Command(line)) => {
                let resp = orch.process_command(&line);
                print!("{resp}");
                let _ = std::fs::write(&status_file, format!("{}\n{}\n", line, orch.status()));
            }
            Ok(Input::Exit) => {
                println!("👋 Strateji orkestrasyonu kapatılıyor...");
                for name in orch.available() {
                    let _ = orch.stop(&name);
                }
                std::process::exit(0);
            }
            Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }

        if last_tick.elapsed().as_millis() >= 500 {
            orch.tick();
            last_tick = std::time::Instant::now();
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
