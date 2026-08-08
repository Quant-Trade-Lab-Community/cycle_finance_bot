//! Emir öncesi risk kontrolleri: sembol blocklist, max notional,
//! dakika başına emir limiti (kayan pencere).

use crate::config::ExecConfig;
use crate::error::{ExecError, Result};
use crate::order::OrderRequest;
use parking_lot::Mutex;
use rust_decimal::Decimal;
use std::collections::{HashSet, VecDeque};
use std::time::Instant;

pub struct RiskChecks {
    max_notional_usdt: Decimal,
    max_orders_per_min: u32,
    blocklist: HashSet<String>,
    /// Dakika penceresindeki emir zamanları.
    order_window: Mutex<VecDeque<Instant>>,
}

impl RiskChecks {
    pub fn new(config: &ExecConfig) -> Self {
        Self {
            max_notional_usdt: config.max_notional_usdt,
            max_orders_per_min: config.max_orders_per_min,
            blocklist: config.symbol_blocklist.clone(),
            order_window: Mutex::new(VecDeque::new()),
        }
    }

    pub fn max_notional(&self) -> Decimal {
        self.max_notional_usdt
    }

    pub fn set_max_notional(&mut self, v: Decimal) {
        self.max_notional_usdt = v;
    }

    pub fn set_max_orders_per_min(&mut self, v: u32) {
        self.max_orders_per_min = v;
    }

    pub fn set_blocklist(&mut self, list: HashSet<String>) {
        self.blocklist = list;
    }

    pub fn blocklist(&self) -> &HashSet<String> {
        &self.blocklist
    }

    /// Emir gönderim öncesi risk kontrolü (kill switch ayrıca denetlenir).
    pub fn check(&self, order: &OrderRequest) -> Result<()> {
        let symbol = order.symbol.to_uppercase();
        if self.blocklist.contains(&symbol) {
            return Err(ExecError::Risk(format!("{symbol} blocklist'te — emir reddedildi")));
        }

        if self.max_notional_usdt > Decimal::ZERO {
            let notional = order.estimated_notional();
            if notional > Decimal::ZERO && notional > self.max_notional_usdt {
                return Err(ExecError::Risk(format!(
                    "notional {notional} USDT, üst sınır {} USDT aşıldı",
                    self.max_notional_usdt
                )));
            }
        }

        self.rate_limit().map_err(ExecError::Risk)
    }

    /// Kayan pencere: son 60 sn'deki emir sayısı.
    pub fn rate_limit(&self) -> std::result::Result<(), String> {
        if self.max_orders_per_min == 0 {
            return Ok(());
        }
        let mut w = self.order_window.lock();
        let cutoff = Instant::now() - std::time::Duration::from_secs(60);
        while let Some(&t) = w.front() {
            if t < cutoff {
                w.pop_front();
            } else {
                break;
            }
        }
        if w.len() >= self.max_orders_per_min as usize {
            return Err(format!(
                "dakikada {} emir limiti doldu",
                self.max_orders_per_min
            ));
        }
        Ok(())
    }

    /// Başarılı gönderim sonrası pencereye kaydet.
    pub fn record_order(&self) {
        let mut w = self.order_window.lock();
        w.push_back(Instant::now());
        let cutoff = Instant::now() - std::time::Duration::from_secs(60);
        while let Some(&t) = w.front() {
            if t < cutoff {
                w.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn orders_in_window(&self) -> usize {
        self.order_window.lock().len()
    }
}
