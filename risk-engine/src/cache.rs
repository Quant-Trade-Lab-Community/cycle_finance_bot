use parking_lot::RwLock;
use std::sync::Arc;

/// Cached risk parameters calculated every 60 seconds by the Risk Worker.
#[derive(Clone, Default)]
pub struct RiskParameters {
    pub max_position_size: f64,
    pub volatility_index: f64,
}

pub struct RiskCache {
    params: Arc<RwLock<RiskParameters>>,
}

impl RiskCache {
    pub fn new() -> Self {
        Self {
            params: Arc::new(RwLock::new(RiskParameters::default())),
        }
    }

    /// Read the latest parameters without blocking the core tick loop.
    /// In a zero-latency scenario, an AtomicPtr Swap might be used instead.
    pub fn read_params(&self) -> RiskParameters {
        self.params.read().clone()
    }

    /// Risk worker updates the parameters every 60 seconds.
    pub fn update_params(&self, new_params: RiskParameters) {
        let mut w = self.params.write();
        *w = new_params;
        println!("RiskCache: Parameters updated.");
    }
}
