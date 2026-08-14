//! `executiond` — canlı Binance Futures execution daemon'u.
//!
//! Çalıştırma:
//! ```bash
//! EXEC_MODE=LIVE EXEC_DRY_RUN=true ./target/debug/executiond
//! ```

use clap::Parser;
use execution_engine::config::ExecConfig;
use execution_engine::ExecutionEngine;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(name = "executiond", about = "Canlı Binance Futures execution servisi")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value = "3010")]
    port: u16,
    /// Config'teki EXEC_DRY_RUN'ı geçersiz kılar — gerçek emir gönderir.
    #[arg(long)]
    no_dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "execution_engine=info".into()),
        )
        .init();

    let args = Args::parse();
    let mut config = ExecConfig::load_from_env();
    if args.no_dry_run {
        config.dry_run = false;
    }

    println!("========================================");
    println!("🛡️ EXECUTION ENGINE v1.0 (Canlı Binance Futures)");
    println!("========================================");
    println!("Mode     : {}", config.mode.as_str());
    println!("Dry run  : {} {}", config.dry_run, if config.dry_run { "(emir gönderilmez)" } else { "(GERÇEK EMİR)" });
    println!("Base URL : {}", config.base_url);
    if config.dry_run {
        println!("⚠️  DRY_RUN AÇIK — emirler doğrulanır ama borsaya GİTMEZ.");
    }

    let engine = ExecutionEngine::start(config).await?;

    let addr = format!("{}:{}", args.host, args.port);
    engine.spawn_rest(&addr);
    println!("REST API : http://{addr}");
    println!("Login    : POST /api/v1/auth/login");

    tokio::signal::ctrl_c().await?;
    engine.shutdown().await;
    println!("Shutting down executiond...");
    Ok(())
}
