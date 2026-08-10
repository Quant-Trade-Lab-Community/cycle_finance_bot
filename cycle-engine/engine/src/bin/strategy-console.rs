//! 🧠 STRATEGY konsolu — strateji orkestrasyon merkezi (ayrı binary).
//!
//! `engine` binary'sinden bağımsız, DATA konsolunu etkilemeyen saf
//! orkestrasyon konsolu. `StrategyOrchestrator`'ı barındırır; komutları
//! stdin'den ve `/tmp/strategy_cmd.d` kuyruğundan dinler.
//!
//! ```bash
//! ./target/debug/strategy-console
//! ```

fn main() {
    engine::engine::strategy_console::run_strategy_console();
}
