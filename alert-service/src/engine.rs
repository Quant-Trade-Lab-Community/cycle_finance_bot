//! Uyarı motoru: fiyat akışını değerlendirir, koşul sağlanınca tetikler.

use crate::audio;
use crate::config::{AlertRule, Condition};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Armed,
    Triggered,
}

#[derive(Debug)]
struct Runtime {
    state: State,
    last_trigger_ts: u64,
    last_side_above: Option<bool>,
}

impl Runtime {
    fn new() -> Self {
        Self { state: State::Armed, last_trigger_ts: 0, last_side_above: None }
    }
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub symbol: String,
    pub condition: Condition,
    pub price: Decimal,
    pub voice: String,
}

#[derive(Clone)]
pub struct AlertEngine {
    alerts: Arc<Mutex<Vec<AlertRule>>>,
    runtimes: Arc<Mutex<HashMap<usize, Runtime>>>,
    pub events: flume::Sender<AlertEvent>,
}

impl AlertEngine {
    pub fn new(alerts: Vec<AlertRule>) -> Self {
        let (tx, _rx) = flume::unbounded();
        Self {
            alerts: Arc::new(Mutex::new(alerts)),
            runtimes: Arc::new(Mutex::new(HashMap::new())),
            events: tx,
        }
    }

    pub fn new_with_rx(alerts: Vec<AlertRule>) -> (Self, flume::Receiver<AlertEvent>) {
        let (tx, rx) = flume::unbounded();
        let engine = Self {
            alerts: Arc::new(Mutex::new(alerts)),
            runtimes: Arc::new(Mutex::new(HashMap::new())),
            events: tx,
        };
        (engine, rx)
    }

    /// Runtime'da yeni uyarı ekler.
    pub fn add(&self, rule: AlertRule) {
        self.alerts.lock().unwrap().push(rule);
    }

    pub fn list(&self) -> Vec<AlertRule> {
        self.alerts.lock().unwrap().clone()
    }

    /// Gelen fiyat için tüm ilgili uyarıları değerlendirir.
    pub fn on_price(&self, symbol: &str, price: Decimal) {
        let now = now_secs();
        let mut triggered: Vec<AlertEvent> = Vec::new();
        let alerts = self.alerts.lock().unwrap().clone();
        let mut rt = self.runtimes.lock().unwrap();

        for (i, alert) in alerts.iter().enumerate() {
            if !alert.symbol.eq_ignore_ascii_case(symbol) {
                continue;
            }
            let state = rt.entry(i).or_insert_with(Runtime::new);
            self.evaluate(alert, state, price, now, &mut triggered);
        }
        drop(rt);

        for ev in triggered {
            let _ = self.events.send(ev);
        }
    }

    fn evaluate(
        &self,
        alert: &AlertRule,
        rt: &mut Runtime,
        price: Decimal,
        now: u64,
        out: &mut Vec<AlertEvent>,
    ) {        let tol = price * alert.tolerance_pct;
        let target = alert.price;

        let should_trigger = match alert.condition {
            Condition::Above => {
                // Armed iken üstüne çık → tetikle; tekrar altına inmeden yeniden tetikleme
                match rt.state {
                    State::Armed => price >= target,
                    State::Triggered => {
                        if price < target - tol {
                            rt.state = State::Armed;
                        }
                        false
                    }
                }
            }
            Condition::Below => match rt.state {
                State::Armed => price <= target,
                State::Triggered => {
                    if price > target + tol {
                        rt.state = State::Armed;
                    }
                    false
                }
            },
            Condition::Touch => {
                let near = (price - target).abs() <= tol.max(Decimal::ONE * Decimal::from_str("0.00000001").unwrap());
                match rt.state {
                    State::Armed => near,
                    State::Triggered => {
                        if !near {
                            rt.state = State::Armed;
                        }
                        false
                    }
                }
            }
            Condition::Cross => {
                let side_above = price >= target;
                let crossed = match rt.last_side_above {
                    Some(prev) if prev != side_above => true,
                    _ => false,
                };
                rt.last_side_above = Some(side_above);
                crossed
            }
        };

        if should_trigger {
            // cooldown kontrolü
            if alert.cooldown_sec > 0 && now.saturating_sub(rt.last_trigger_ts) < alert.cooldown_sec {
                return;
            }
            rt.last_trigger_ts = now;
            rt.state = State::Triggered;
            out.push(AlertEvent {
                symbol: alert.symbol.clone(),
                condition: alert.condition,
                price,
                voice: alert.voice.clone(),
            });

            if !alert.repeat {
                // tek seferlik: bu uyarıyı devre dışı bırak (repeat=false → devamlı Triggered kalır,
                // re-arm mantığı aşağıdaki kollarda çalışmaz)
                rt.state = State::Triggered;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.alerts.lock().unwrap().len()
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Event'leri dinleyip ses üreten task'ı başlatır.
pub fn spawn_alert_sink(rx: flume::Receiver<AlertEvent>) {
    std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() {
            audio::trigger(&ev.voice, &ev.symbol, ev.condition.as_str(), ev.price);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn rule(symbol: &str, cond: Condition, price: &str) -> AlertRule {
        AlertRule {
            symbol: symbol.into(),
            condition: cond,
            price: Decimal::from_str(price).unwrap(),
            tolerance_pct: Decimal::from_str("0.0005").unwrap(),
            voice: String::new(),
            cooldown_sec: 0,
            repeat: true,
        }
    }

    fn collect(engine: &AlertEngine, rx: &flume::Receiver<AlertEvent>, symbol: &str, price: &str) -> Vec<AlertEvent> {
        engine.on_price(symbol, Decimal::from_str(price).unwrap());
        let mut out = Vec::new();
        while let Ok(ev) = rx.try_recv() {
            out.push(ev);
        }
        out
    }

    #[test]
    fn test_above_fires_once_and_rearms() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Above, "64300")]);
        // 1. fiyat hedef üstünde → tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
        // 2. fiyat hâlâ üstünde → tetiklenmez (re-arm yok)
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64450").len(), 0);
        // 3. hedef altına iner → re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64200").len(), 0);
        // 4. tekrar üstüne çıkar → yeniden tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
    }

    #[test]
    fn test_below_fires_once_and_rearms() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Below, "64000")]);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63900").len(), 1);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63800").len(), 0);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64100").len(), 0); // re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63950").len(), 1);
    }

    #[test]
    fn test_cross_fires_on_each_crossing() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("ETHUSDT", Condition::Cross, "3200")]);
        // ilk tick: yön belirlenir, tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3100").len(), 0);
        // aynı yönde: tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3000").len(), 0);
        // üstüne çıkar → tetiklenir
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3300").len(), 1);
        // hâlâ üstünde → tetiklenmez
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3400").len(), 0);
        // altına iner → tetiklenir
        assert_eq!(collect(&engine, &rx, "ETHUSDT", "3100").len(), 1);
    }

    #[test]
    fn test_touch_fires_when_near() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![rule("BTCUSDT", Condition::Touch, "64400")]);
        // uzakta: tetiklenmez
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "60000").len(), 0);
        // tol (64400*0.0005=32.2) içinde → tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64420").len(), 1);
        // hâlâ yakın → tetiklenmez
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64410").len(), 0);
        // uzaklaşır → re-arm
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0);
        // yaklaşır → tekrar
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "64400").len(), 1);
    }

    #[test]
    fn test_cooldown_blocks_retrigger() {
        let (engine, rx) = AlertEngine::new_with_rx(vec![AlertRule {
            symbol: "BTCUSDT".into(),
            condition: Condition::Cross,
            price: Decimal::from_str("64000").unwrap(),
            tolerance_pct: Decimal::from_str("0.0005").unwrap(),
            voice: String::new(),
            cooldown_sec: 3600,
            repeat: true,
        }]);
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0); // ilk yön
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "63000").len(), 1); // tetiklenir
        assert_eq!(collect(&engine, &rx, "BTCUSDT", "65000").len(), 0); // cooldown engeller
    }
}
