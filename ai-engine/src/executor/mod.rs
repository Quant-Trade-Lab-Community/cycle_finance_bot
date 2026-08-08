//! İcra katmanı — paper (order ring) ve canlı (executiond) emir gönderimi.
//! `mode` (paper/live/both/none) ve `approval` (auto/human) burada uygulanır.

pub mod live;
pub mod paper;

use crate::config::AiConfig;
use crate::{Action, now_ms};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};

pub struct Executor {
    mode: String,
    approval: String,
    approval_wait_secs: u64,
    paper: Option<paper::PaperExecutor>,
    live: Option<live::LiveExecutor>,
}

impl Executor {
    pub fn new(cfg: &AiConfig) -> Self {
        let paper = match cfg.execution.mode.as_str() {
            "paper" | "both" => paper::PaperExecutor::new(),
            _ => None,
        };
        let live = match cfg.execution.mode.as_str() {
            "live" | "both" => Some(live::LiveExecutor::new(cfg)),
            _ => None,
        };
        Self {
            mode: cfg.execution.mode.clone(),
            approval: cfg.execution.approval.clone(),
            approval_wait_secs: cfg.schedule.approval_wait_secs,
            paper,
            live,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.mode != "none"
    }

    /// Emri icra eder. HITL (human-in-the-loop) modunda insan onayı bekler.
    pub async fn execute(
        &self,
        symbol: &str,
        action: Action,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<String, String> {
        if !action.is_trade() {
            return Err("HOLD emri gönderilmez".into());
        }
        if self.approval == "human" {
            self.await_approval(symbol, action, quantity, price).await?;
        }

        match self.mode.as_str() {
            "none" => Err("execution.mode = none (sadece izleme)".into()),
            "paper" => match &self.paper {
                Some(p) => p.execute(symbol, action, quantity, price),
                None => Err("paper executor başlatılamadı (order ring açılamadı)".into()),
            },
            "live" => match &self.live {
                Some(l) => l.execute(symbol, action, quantity, price).await,
                None => Err("live executor başlatılamadı".into()),
            },
            "both" => {
                let paper_msg = match &self.paper {
                    Some(p) => match p.execute(symbol, action, quantity, price) {
                        Ok(m) => Some(m),
                        Err(e) => return Err(format!("PAPER başarısız: {e}")),
                    },
                    None => None,
                };
                let live_msg = match &self.live {
                    Some(l) => l.execute(symbol, action, quantity, price).await?,
                    None => String::new(),
                };
                Ok(format!(
                    "✅ BOTH: paper={:?} live={}",
                    paper_msg, live_msg
                ))
            }
            other => Err(format!("bilinmeyen execution.mode: {other}")),
        }
    }

    async fn await_approval(
        &self,
        symbol: &str,
        action: Action,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<(), String> {
        let pending = serde_json::json!({
            "ts_ms": now_ms(),
            "symbol": symbol,
            "action": action.as_str(),
            "quantity": quantity.to_string(),
            "price": price.map(|p| p.to_string()),
        });
        let _ = std::fs::write(
            "/tmp/ai_pending.json",
            serde_json::to_string_pretty(&pending).unwrap_or_default(),
        );
        println!(
            "🕐 ONAY BEKLENİYOR: {symbol} {} {} @ {:?}\n   Onaylamak için: echo approve > /tmp/ai_approve.txt",
            action.as_str(),
            quantity,
            price
        );

        let deadline = Instant::now() + Duration::from_secs(self.approval_wait_secs);
        loop {
            if let Ok(content) = std::fs::read_to_string("/tmp/ai_approve.txt") {
                let c = content.trim().to_ascii_lowercase();
                if c == "approve" || c == "1" || c == "evet" || c == "ok" {
                    let _ = std::fs::remove_file("/tmp/ai_approve.txt");
                    return Ok(());
                }
                if c == "reject" || c == "0" || c == "hayır" || c == "no" {
                    let _ = std::fs::remove_file("/tmp/ai_approve.txt");
                    return Err("insan onayı reddetti — emir gönderilmedi".into());
                }
            }
            if Instant::now() >= deadline {
                return Err("onay zaman aşımı — fail-safe: emir gönderilmedi".into());
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    }
}
