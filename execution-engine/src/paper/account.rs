use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AccountState {
    pub free_balances: HashMap<String, f64>,
    pub locked_balances: HashMap<String, f64>,
}

impl AccountState {
    pub fn new(initial_quote: f64, initial_base: f64) -> Self {
        let mut free = HashMap::new();
        free.insert("USDT".to_string(), initial_quote);
        free.insert("BTC".to_string(), initial_base); // Can be parameterized later
        
        Self {
            free_balances: free,
            locked_balances: HashMap::new(),
        }
    }

    pub fn get_free(&self, asset: &str) -> f64 {
        *self.free_balances.get(asset).unwrap_or(&0.0)
    }

    pub fn get_locked(&self, asset: &str) -> f64 {
        *self.locked_balances.get(asset).unwrap_or(&0.0)
    }

    pub fn lock_funds(&mut self, asset: &str, amount: f64) -> Result<(), String> {
        let free = self.get_free(asset);
        if free < amount {
            return Err(format!("Insufficient funds: {} < {}", free, amount));
        }
        
        *self.free_balances.entry(asset.to_string()).or_insert(0.0) -= amount;
        *self.locked_balances.entry(asset.to_string()).or_insert(0.0) += amount;
        Ok(())
    }

    pub fn unlock_funds(&mut self, asset: &str, amount: f64) {
        let locked = self.get_locked(asset);
        let amount_to_unlock = if locked < amount { locked } else { amount };
        
        *self.locked_balances.entry(asset.to_string()).or_insert(0.0) -= amount_to_unlock;
        *self.free_balances.entry(asset.to_string()).or_insert(0.0) += amount_to_unlock;
    }

    pub fn deduct_locked_funds(&mut self, asset: &str, amount: f64) {
        let locked = self.get_locked(asset);
        let amount_to_deduct = if locked < amount { locked } else { amount };
        *self.locked_balances.entry(asset.to_string()).or_insert(0.0) -= amount_to_deduct;
    }
    
    pub fn add_free_funds(&mut self, asset: &str, amount: f64) {
        *self.free_balances.entry(asset.to_string()).or_insert(0.0) += amount;
    }

    pub fn deduct_free_funds(&mut self, asset: &str, amount: f64) -> Result<(), String> {
        let free = self.get_free(asset);
        if free < amount {
            return Err(format!("Insufficient funds for fee: {} < {}", free, amount));
        }
        *self.free_balances.entry(asset.to_string()).or_insert(0.0) -= amount;
        Ok(())
    }
}
