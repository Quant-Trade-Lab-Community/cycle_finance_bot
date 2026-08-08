pub mod redis;
pub mod clickhouse;
pub mod ai;
pub mod vault;
pub mod telemetry;
pub mod binance;

pub use redis::{RedisAdapter, RedisHealth};

/// Adapter altyapısını başlatır: Redis bağlantısını kurar ve sağlığını loglar.
/// (Vault sağlık kontrolü async olduğu için ayrıca `vault.health()` çağrılır.)
pub fn init_adapter() {
    let redis = RedisAdapter::new();
    match redis.health() {
        RedisHealth::Connected => println!("Adapter initialized | Redis: connected"),
        RedisHealth::Degraded => println!("Adapter initialized | Redis: DEGRADED (fail-closed)"),
    }
}
