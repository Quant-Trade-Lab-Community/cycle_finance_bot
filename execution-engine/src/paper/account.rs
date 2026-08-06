use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AccountState {
    pub free_balances: HashMap<String, Decimal>,
    pub locked_balances: HashMap<String, Decimal>,
}

impl AccountState {
    pub fn new(initial_quote: Decimal, initial_base: Decimal) -> Self {
        let mut free = HashMap::new();
        free.insert("USDT".to_string(), initial_quote);
        free.insert("BTC".to_string(), initial_base); // Can be parameterized later

        Self {
            free_balances: free,
            locked_balances: HashMap::new(),
        }
    }

    pub fn get_free(&self, asset: &str) -> Decimal {
        *self.free_balances.get(asset).unwrap_or(&Decimal::ZERO)
    }

    pub fn get_locked(&self, asset: &str) -> Decimal {
        *self.locked_balances.get(asset).unwrap_or(&Decimal::ZERO)
    }

    pub fn lock_funds(&mut self, asset: &str, amount: Decimal) -> Result<(), String> {
        let free = self.get_free(asset);
        if free < amount {
            return Err(format!("Insufficient funds: {} < {}", free, amount));
        }

        *self.free_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) -= amount;
        *self.locked_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) += amount;
        Ok(())
    }

    pub fn unlock_funds(&mut self, asset: &str, amount: Decimal) {
        let locked = self.get_locked(asset);
        let amount_to_unlock = if locked < amount { locked } else { amount };

        *self.locked_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) -= amount_to_unlock;
        *self.free_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) += amount_to_unlock;
    }

    pub fn deduct_locked_funds(&mut self, asset: &str, amount: Decimal) {
        let locked = self.get_locked(asset);
        let amount_to_deduct = if locked < amount { locked } else { amount };
        *self.locked_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) -= amount_to_deduct;
    }

    pub fn add_free_funds(&mut self, asset: &str, amount: Decimal) {
        *self.free_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) += amount;
    }

    pub fn deduct_free_funds(&mut self, asset: &str, amount: Decimal) -> Result<(), String> {
        let free = self.get_free(asset);
        if free < amount {
            return Err(format!("Insufficient funds for fee: {} < {}", free, amount));
        }
        *self.free_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) -= amount;
        Ok(())
    }

    /// Kısa (short) pozisyon için borçlanma: bakiyeyi negatife düşürmeye izin verir.
    pub fn subtract_free_funds_unchecked(&mut self, asset: &str, amount: Decimal) {
        *self.free_balances.entry(asset.to_string()).or_insert(Decimal::ZERO) -= amount;
    }
}
