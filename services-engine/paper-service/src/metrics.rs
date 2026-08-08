//! Prometheus metrikleri (sıfır-bağımlılık, atomic sayaçlar).
//!
//! `GET /metrics` Prometheus text formatında döner.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct Metrics {
    pub order_place_total: AtomicU64,
    pub order_place_failure_total: AtomicU64,
    pub liquidation_events_total: AtomicU64,
    pub funding_events_total: AtomicU64,
    pub fills_total: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn record_order(&self, success: bool) {
        if success {
            self.order_place_total.fetch_add(1, Ordering::Relaxed);
        } else {
            self.order_place_failure_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_liquidation(&self) {
        self.liquidation_events_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_funding(&self) {
        self.funding_events_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_fill(&self) {
        self.fills_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render(&self, balance_usdt: String) -> String {
        format!(
            "# HELP paper_order_place_total Toplam emir gönderimi\n\
             # TYPE paper_order_place_total counter\n\
             paper_order_place_total {}\n\
             # HELP paper_order_place_failure_total Reddedilen emirler\n\
             # TYPE paper_order_place_failure_total counter\n\
             paper_order_place_failure_total {}\n\
             # HELP paper_liquidation_events_total Likidasyon sayısı\n\
             # TYPE paper_liquidation_events_total counter\n\
             paper_liquidation_events_total {}\n\
             # HELP paper_funding_events_total Funding uygulama sayısı\n\
             # TYPE paper_funding_events_total counter\n\
             paper_funding_events_total {}\n\
             # HELP paper_fills_total Gerçekleşen dolum sayısı\n\
             # TYPE paper_fills_total counter\n\
             paper_fills_total {}\n\
             # HELP paper_account_balance_usdt Hesap bakiyesi (USDT)\n\
             # TYPE paper_account_balance_usdt gauge\n\
             paper_account_balance_usdt {}\n",
            self.order_place_total.load(Ordering::Relaxed),
            self.order_place_failure_total.load(Ordering::Relaxed),
            self.liquidation_events_total.load(Ordering::Relaxed),
            self.funding_events_total.load(Ordering::Relaxed),
            self.fills_total.load(Ordering::Relaxed),
            balance_usdt,
        )
    }
}
