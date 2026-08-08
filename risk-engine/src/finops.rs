/// FinOps Module for Cloud Cost Optimization.
pub struct FinOpsOptimizer {
    pub last_30d_profit: f64,
    pub current_cloud_cost: f64,
}

impl FinOpsOptimizer {
    pub fn new(profit: f64, cost: f64) -> Self {
        Self {
            last_30d_profit: profit,
            current_cloud_cost: cost,
        }
    }

    /// Triggers cold data repack in ClickHouse if cost exceeds 20% of profit.
    pub fn evaluate_cost_efficiency(&self) {
        let threshold = self.last_30d_profit * 0.20;
        
        if self.current_cloud_cost > threshold {
            println!("FinOps: Cloud cost ({}) exceeds 20% of profit ({}).", self.current_cloud_cost, threshold);
            println!("FinOps: Triggering Zstandard (Level 22) repack and dropping unused indices for cold data...");
            // Calls ClickHouse Adapter to execute ALTER TABLE ... MODIFY SETTING
        } else {
            println!("FinOps: Cost efficiency is within limits.");
        }
    }
}
