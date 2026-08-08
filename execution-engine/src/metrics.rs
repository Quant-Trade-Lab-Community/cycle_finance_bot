//! Operasyonel metrikler: sayaçlar + emir gecikme histogramı.
//!
//! `hdrhistogram` ile yüksek çözünürlüklü gecikme dağılımı; `GET /metrics`
//! Prometheus uyumlu metin döndürür.

use hdrhistogram::Histogram;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Metrics {
    pub orders_submitted: AtomicU64,
    pub orders_filled: AtomicU64,
    pub orders_rejected: AtomicU64,
    pub orders_cancelled: AtomicU64,
    pub ws_reconnects: AtomicU64,
    pub resyncs: AtomicU64,
    pub http_errors: AtomicU64,
    pub rate_limited: AtomicU64,
    latency: Mutex<Histogram<u64>>,
}

impl Default for Metrics {
    fn default() -> Self {
        let mut h = Histogram::<u64>::new(3).expect("histogram");
        h.auto(true);
        Self {
            latency: Mutex::new(h),
            orders_submitted: AtomicU64::new(0),
            orders_filled: AtomicU64::new(0),
            orders_rejected: AtomicU64::new(0),
            orders_cancelled: AtomicU64::new(0),
            ws_reconnects: AtomicU64::new(0),
            resyncs: AtomicU64::new(0),
            http_errors: AtomicU64::new(0),
            rate_limited: AtomicU64::new(0),
        }
    }
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_order(&self, ok: bool) {
        if ok {
            self.orders_submitted.fetch_add(1, Ordering::Relaxed);
        } else {
            self.orders_rejected.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_fill(&self) {
        self.orders_filled.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cancel(&self) {
        self.orders_cancelled.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_reconnect(&self) {
        self.ws_reconnects.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_resync(&self) {
        self.resyncs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_http_error(&self) {
        self.http_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rate_limited(&self) {
        self.rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    /// Emir gidiş-dönüş gecikmesini histograma kaydet.
    pub fn record_latency_us(&self, us: u64) {
        self.latency.lock().record(us).ok();
    }

    pub fn latency_summary(&self) -> (u64, u64, u64) {
        let h = self.latency.lock();
        (
            h.value_at_quantile(0.50),
            h.value_at_quantile(0.99),
            h.max(),
        )
    }

    pub fn render_prometheus(&self) -> String {
        let (p50, p99, max) = self.latency_summary();
        format!(
            "# HELP exec_orders_submitted Gönderilen emir sayısı\n# TYPE exec_orders_submitted counter\nexec_orders_submitted {}\n\
             # TYPE exec_orders_filled counter\nexec_orders_filled {}\n\
             # TYPE exec_orders_rejected counter\nexec_orders_rejected {}\n\
             # TYPE exec_orders_cancelled counter\nexec_orders_cancelled {}\n\
             # TYPE exec_ws_reconnects counter\nexec_ws_reconnects {}\n\
             # TYPE exec_resyncs counter\nexec_resyncs {}\n\
             # TYPE exec_http_errors counter\nexec_http_errors {}\n\
             # TYPE exec_rate_limited counter\nexec_rate_limited {}\n\
             # TYPE exec_order_latency_us gauge\nexec_order_latency_us_p50 {}\n\
             exec_order_latency_us_p99 {}\n\
             exec_order_latency_us_max {}\n",
            self.orders_submitted.load(Ordering::Relaxed),
            self.orders_filled.load(Ordering::Relaxed),
            self.orders_rejected.load(Ordering::Relaxed),
            self.orders_cancelled.load(Ordering::Relaxed),
            self.ws_reconnects.load(Ordering::Relaxed),
            self.resyncs.load(Ordering::Relaxed),
            self.http_errors.load(Ordering::Relaxed),
            self.rate_limited.load(Ordering::Relaxed),
            p50,
            p99,
            max,
        )
    }
}
