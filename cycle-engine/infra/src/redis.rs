use redis::Commands;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Redis kullanılabilirlik durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisHealth {
    Connected,
    Degraded,
}

/// Idempotency and State caching via Redis.
///
/// Gerçek Redis istemcisi: `REDIS_URL` çevre değişkeninden URL alır
/// (varsayılan `redis://127.0.0.1:6379`). Redis yoksa fail-closed davranılır:
/// emir idempotency anahtarı yazılamaz → işlem reddedilir (kayıp emir yok,
/// çoğaltılmış emir yok).
pub struct RedisAdapter {
    conn: Option<Mutex<redis::Connection>>,
}

impl RedisAdapter {
    /// `REDIS_URL` ile bağlanır. Bağlantı katı KC değildir — ilk işlemde doğrulanır.
    pub fn new() -> Self {
        let url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::with_url(&url)
    }

    pub fn with_url(url: &str) -> Self {
        match redis::Client::open(url) {
            Ok(client) => match client.get_connection_with_timeout(Duration::from_secs(5)) {
                Ok(conn) => {
                    println!("Redis: Connected to {}", url);
                    Self { conn: Some(Mutex::new(conn)) }
                }
                Err(e) => {
                    eprintln!("Redis: Bağlantı kurulamadı ({}): {}", url, e);
                    Self { conn: None }
                }
            },
            Err(e) => {
                eprintln!("Redis: Geçersiz URL ({}): {}", url, e);
                Self { conn: None }
            }
        }
    }

    pub fn health(&self) -> RedisHealth {
        match &self.conn {
            Some(c) => {
                let mut guard = c.lock().unwrap();
                if redis::cmd("PING").query::<String>(&mut *guard).is_ok() {
                    RedisHealth::Connected
                } else {
                    RedisHealth::Degraded
                }
            }
            None => RedisHealth::Degraded,
        }
    }

    /// Generates a unique clientOrderId to prevent replay attacks and duplicate orders.
    /// Format: "BOT_UUID_timestamp_nano"
    pub fn generate_client_order_id(&self, bot_uuid: &str) -> String {
        let nano = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}_{}", bot_uuid, nano)
    }

    /// Writes the order ID to Redis with a strict 1-hour TTL (3600 seconds) for Idempotency.
    /// Atomic `SET key 1 EX 3600 NX` — aynı anahtarla ikinci yazma başarısız olur
    /// (çift emir/tekrar koruması). Redis yoksa fail-closed: `Err`.
    pub fn set_idempotency_key(&self, order_id: &str) -> Result<(), &'static str> {
        let conn = self.conn.as_ref().ok_or("Redis unavailable")?;
        let mut guard = conn.lock().unwrap();
        let ttl_seconds: u64 = 3600;
        match redis::cmd("SET")
            .arg(order_id)
            .arg(1u8)
            .arg("EX")
            .arg(ttl_seconds)
            .arg("NX")
            .query::<Option<i64>>(&mut *guard)
        {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err("duplicate: idempotency key already set"),
            Err(_) => Err("Redis command failed"),
        }
    }

    /// Fetches the idempotency status. If the key times out (5s), returns "Pending".
    /// Var olan anahtarla -> "Confirmed"; yoksa / sorun varsa -> "Pending".
    pub fn check_ack_status(&self, order_id: &str) -> String {
        let Some(conn) = self.conn.as_ref() else {
            return "Pending".to_string();
        };
        let mut guard = conn.lock().unwrap();
        match guard.get::<_, Option<String>>(order_id) {
            Ok(Some(_)) => "Confirmed".to_string(),
            _ => "Pending".to_string(),
        }
    }
}

impl Default for RedisAdapter {
    fn default() -> Self {
        Self::new()
    }
}
