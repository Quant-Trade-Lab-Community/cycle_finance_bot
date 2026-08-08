//! HTTP katmanı: bağlantı havuzu, zaman aşımı, ağırlık takibi, yeniden deneme.
//!
//! - Sunucu saati senkronu: `offset_ms` her istekte timestamp'e eklenir;
//!   `-1021` (timestamp out of window) hatasında yeniden senkronlanır.
//! - Ağırlık takibi: `x-mbx-used-weight-1m` yanıt başlığından `weight_used`'a.
//! - Yeniden deneme yalnızca yeniden denenebilir hatalarda; emir yazımları
//!   `clientOrderId` idempotency'sine dayanır.

use crate::error::{ExecError, Result};
use crate::signer::BinanceSigner;
use parking_lot::RwLock;
use serde_json::Value;
use std::sync::atomic::{AtomicI64, AtomicI32, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_ATTEMPTS: u32 = 3;
const BASE_BACKOFF_MS: u64 = 250;

pub struct HttpClient {
    inner: reqwest::Client,
    base_url: String,
    timeout: Duration,
    /// Sunucu - yerel saat farkı (ms). `sync_server_time` ile güncellenir.
    server_offset_ms: AtomicI64,
    /// Son 1 dakikada kullanılan ağırlık (x-mbx-used-weight-1m).
    weight_used: AtomicI32,
    last_sync: RwLock<u64>,
}

impl HttpClient {
    pub fn new(base_url: String, timeout_ms: u64) -> Result<Arc<Self>> {
        let inner = reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        Ok(Arc::new(Self {
            inner,
            base_url,
            timeout: Duration::from_millis(timeout_ms),
            server_offset_ms: AtomicI64::new(0),
            weight_used: AtomicI32::new(0),
            last_sync: RwLock::new(0),
        }))
    }

    pub fn weight_used(&self) -> i32 {
        self.weight_used.load(Ordering::Relaxed)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn now_ms(&self) -> u64 {
        let local = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        (local + self.server_offset_ms.load(Ordering::Relaxed)).max(0) as u64
    }

    /// Sunucu saatini senkronize eder (drift ölçer). Başarısız olursa eski offset korunur.
    pub async fn sync_server_time(&self) -> Result<()> {
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let url = format!("{}/fapi/v1/time", self.base_url);
        let resp = self.inner.get(&url).timeout(self.timeout).send().await?;
        let status = resp.status();
        let body: Value = resp.json().await?;
        if !status.is_success() {
            return Err(ExecError::InvalidResponse(format!(
                "server time sync failed: http {status}: {body}"
            )));
        }
        let server = body["serverTime"].as_i64().ok_or_else(|| {
            ExecError::InvalidResponse("serverTime missing from /fapi/v1/time".into())
        })?;
        let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        // RTT'nin yarısını düşerek tahmini geçiş gecikmesini telafi et.
        let offset = server - ((before + after) / 2);
        self.server_offset_ms.store(offset, Ordering::Relaxed);
        *self.last_sync.write() = now_unix_ms();
        Ok(())
    }

    pub async fn sync_server_time_if_stale(&self, max_age_ms: u64) -> Result<()> {
        if now_unix_ms().saturating_sub(*self.last_sync.read()) > max_age_ms {
            self.sync_server_time().await?;
        }
        Ok(())
    }

    /// Temel istek. `signed=true` ise `params`'a `timestamp` (+ isteğe bağlı
    /// `recvWindow`) eklenir ve HMAC ile imzalanır.
    pub async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Vec<(String, String)>,
        signer: Option<&BinanceSigner>,
        recv_window: u64,
    ) -> Result<Value> {
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let url = match self.build_url(path, &params, signer, recv_window) {
                Ok(u) => u,
                Err(e) => return Err(e),
            };

            let mut req = self.inner.request(method.clone(), &url).timeout(self.timeout);
            // İmzalı isteklerde API anahtarı X-MBX-APIKEY başlığıyla gider.
            // (Bu başlık olmadan Binance -2014 "API-key format invalid" döner.)
            if let Some(s) = signer {
                req = req.header("X-MBX-APIKEY", s.api_key());
            }
            if method == reqwest::Method::POST || method == reqwest::Method::PUT {
                req = req.header("content-type", "application/x-www-form-urlencoded");
            }

            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    if e.is_timeout() || e.is_connect() {
                        if attempt < MAX_ATTEMPTS {
                            backoff(attempt).await;
                            continue;
                        }
                        return Err(ExecError::Http(e));
                    }
                    return Err(ExecError::Http(e));
                }
            };

            // Ağırlık takibi (yoksa 0).
            if let Some(w) = resp.headers().get("x-mbx-used-weight-1m")
                && let Ok(s) = w.to_str()
                    && let Ok(n) = s.parse::<i32>() {
                        self.weight_used.store(n, Ordering::Relaxed);
                    }

            let status = resp.status();

            // 429 / 418: rate limit — retry-after başlığını oku.
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status == reqwest::StatusCode::from_u16(418).unwrap() {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(2_000);
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(retry_after)).await;
                    continue;
                }
                return Err(ExecError::RateLimit { retry_after_ms: retry_after });
            }

            let text = resp.text().await?;
            let body: Value = if text.trim().is_empty() {
                Value::Null
            } else {
                match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(ExecError::InvalidResponse(format!(
                            "http {status}, body not json: {e} (first 200 bytes: {})",
                            text.chars().take(200).collect::<String>()
                        )));
                    }
                }
            };

            if !status.is_success() {
                let code = body.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
                let msg = body.get("msg").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let err = ExecError::Binance {
                    http_status: status.as_u16(),
                    code,
                    msg,
                };
                // Timestamp drift: sunucu saatini yeniden senkronla ve tekrar dene.
                if code == -1021 {
                    let _ = self.sync_server_time().await;
                    if attempt < MAX_ATTEMPTS {
                        continue;
                    }
                }
                if err.is_retryable() && attempt < MAX_ATTEMPTS {
                    backoff(attempt).await;
                    continue;
                }
                return Err(err);
            }

            return Ok(body);
        }
    }

    fn build_url(
        &self,
        path: &str,
        params: &[(String, String)],
        signer: Option<&BinanceSigner>,
        recv_window: u64,
    ) -> Result<String> {
        let mut p = params.to_vec();
        let mut signed = false;
        if let Some(s) = signer {
            let ts = self.now_ms().to_string();
            p.push(("timestamp".to_string(), ts));
            if recv_window > 0 {
                p.push(("recvWindow".to_string(), recv_window.to_string()));
            }
            p.sort_by_key(|(k, _)| k.clone());
            let qs = build_query(&p);
            let sig = s.sign(&qs);
            p.push(("signature".to_string(), sig));
            signed = true;
        }
        let _ = signed;
        let qs = build_query(&p);
        if qs.is_empty() {
            Ok(format!("{}{}", self.base_url, path))
        } else {
            Ok(format!("{}{}?{}", self.base_url, path, qs))
        }
    }
}

/// Değerler yalnızca güvenli karakterler içerir; olası kaçış için küçük encoder.
fn encode_value(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn build_query(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", encode_value(k), encode_value(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn backoff(attempt: u32) -> impl std::future::Future<Output = ()> {
    let ms = BASE_BACKOFF_MS * (1 << (attempt - 1));
    tokio::time::sleep(Duration::from_millis(ms))
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
