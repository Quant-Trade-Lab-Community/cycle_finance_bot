pub mod redis;
pub mod clickhouse;
pub mod ai;
pub mod vault;
pub mod telemetry;
pub mod binance;

pub fn init_adapter() {
    println!("Adapter initialized");
}
