//! Pozisyon ve risk yönetimi için birim testler.
//! Tüm hesaplamalar rust_decimal ile yapılır (f64 yok).

use rust_decimal::Decimal;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use execution_engine::paper::position::{PositionManager, PositionSide};
    use execution_engine::paper::risk::RiskManager;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_position_open_long_and_unrealized_pnl() {
        let mut pm = PositionManager::new();
        let entry = dec("50000");
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("1"), entry, dec("10"));
        assert_eq!(realized, Decimal::ZERO);
        assert_eq!(closed, Decimal::ZERO);

        let pos = pm.get("BTCUSDT").unwrap();
        assert_eq!(pos.side, PositionSide::Long);
        assert_eq!(pos.quantity, dec("1"));

        // Mark fiyat yükselirse long +PnL
        let pnl = pos.unrealized_pnl(dec("51000"));
        assert_eq!(pnl, dec("1000"));

        // Likidasyon fiyatı: 50000 * (1 - 0.1 + 0.005) = 45250
        assert_eq!(pos.liquidation_price(dec("0.005")), dec("45250"));
    }

    #[test]
    fn test_position_close_realizes_pnl() {
        let mut pm = PositionManager::new();
        pm.apply_fill("BTCUSDT", dec("1"), dec("50000"), dec("10"));
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("-1"), dec("51000"), dec("10"));
        assert_eq!(realized, dec("1000"));
        assert_eq!(closed, dec("1"));
        assert!(pm.get("BTCUSDT").is_none());
    }

    #[test]
    fn test_position_flip_short() {
        let mut pm = PositionManager::new();
        pm.apply_fill("BTCUSDT", dec("2"), dec("50000"), dec("10"));
        // 3 satış: 2 kapanış + 1 short
        let (realized, closed) = pm.apply_fill("BTCUSDT", dec("-3"), dec("49000"), dec("10"));
        // Kapanıştaki 2 birim: (49000 - 50000) * 2 = -2000
        assert_eq!(realized, dec("-2000"));
        assert_eq!(closed, dec("2"));

        let pos = pm.get("BTCUSDT").unwrap();
        assert_eq!(pos.side, PositionSide::Short);
        assert_eq!(pos.quantity, dec("-1"));
        assert_eq!(pos.avg_entry_price, dec("49000"));
    }

    #[test]
    fn test_risk_max_position_rejection() {
        let pm = PositionManager::new();
        let risk = RiskManager::new(dec("10000"), dec("10"), dec("20"), dec("0.05"), dec("1000"));
        let err = risk.check_order(&pm, "BTCUSDT", dec("11"), dec("50000"), dec("20"), dec("10000"));
        assert!(err.is_err());
    }

    #[test]
    fn test_risk_leverage_margin_rejection() {
        let pm = PositionManager::new();
        let risk = RiskManager::new(dec("10000"), dec("10"), dec("20"), dec("0.05"), dec("1000"));
        // 1 BTC * 50000 = 50000 notional, 20x kaldıraç → 2500 marj → yeterli
        assert!(risk.check_order(&pm, "BTCUSDT", dec("1"), dec("50000"), dec("20"), dec("10000")).is_ok());
        // 0.1 BTC * 50000 = 5000 notional, 20x → 250 marj; 100k fiyatla 2 BTC = 200000 notional → 10000 marj = cash → ok sınırda
        assert!(risk.check_order(&pm, "BTCUSDT", dec("2"), dec("100000"), dec("20"), dec("10000")).is_ok());
        // 3 BTC * 100000 = 300000 notional → 15000 marj > 10000 cash → red
        assert!(risk.check_order(&pm, "BTCUSDT", dec("3"), dec("100000"), dec("20"), dec("10000")).is_err());
    }

    #[test]
    fn test_risk_drawdown_breach() {
        let pm = PositionManager::new();
        let mut risk = RiskManager::new(dec("10000"), dec("10"), dec("20"), dec("0.05"), dec("1000"));
        let mut mark_prices = std::collections::HashMap::new();
        mark_prices.insert("BTCUSDT".to_string(), dec("45000"));

        // cash + unrealized ile equity düşür (büyük kayıp)
        let cash = dec("10000");
        // drawdown > %5 için equity < 9500 gerekir; on_mark_tick unrealized'e bakar
        let liquidated = risk.on_mark_tick(&pm, &mark_prices, cash);
        assert!(liquidated.is_empty());
        assert_eq!(risk.status, execution_engine::paper::risk::RiskStatus::Ok);
    }

    #[test]
    fn test_liquidation_trigger_on_long() {
        let mut pm = PositionManager::new();
        pm.apply_fill("BTCUSDT", dec("1"), dec("50000"), dec("10"));
        let mut risk = RiskManager::new(dec("10000"), dec("10"), dec("20"), dec("0.05"), dec("1000"));
        let mut mark_prices = std::collections::HashMap::new();
        // Likidasyon fiyatı 45250; 45000'e düşerse likidasyon tetiklenir
        mark_prices.insert("BTCUSDT".to_string(), dec("45000"));
        let liquidated = risk.on_mark_tick(&pm, &mark_prices, dec("10000"));
        assert_eq!(liquidated, vec!["BTCUSDT".to_string()]);
        assert_eq!(risk.status, execution_engine::paper::risk::RiskStatus::Liquidation);
    }
}
