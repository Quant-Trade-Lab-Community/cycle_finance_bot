//! Binance USDT-M Futures user-data stream olayları.
//!
//! `decoder` ham JSON'u bu tiplere çevirir. Sayısal alanlar string gelir;
//! `rust_decimal`'e çevrilir, geçersiz değerler `0` kabul edilir (yanıt
//! biçimi borsa tarafından garantili olmadığından savunmacı yaklaşım).

use rust_decimal::Decimal;

fn dec(s: Option<&str>) -> Decimal {
    s.and_then(|v| v.parse().ok()).unwrap_or(Decimal::ZERO)
}

/// ACCOUNT_UPDATE içindeki bakiye deltası.
#[derive(Debug, Clone, Default)]
pub struct AccountUpdateBalance {
    pub asset: String,
    pub wallet_balance: Decimal,
    pub cross_wallet_balance: Decimal,
}

/// ACCOUNT_UPDATE içindeki pozisyon deltası.
#[derive(Debug, Clone, Default)]
pub struct AccountUpdatePosition {
    pub symbol: String,
    pub position_side: String,
    pub position_amt: Decimal,
    pub entry_price: Decimal,
    pub un_realized_profit: Decimal,
    pub margin_type: String,
    pub isolated_wallet: Decimal,
}

/// ORDER_TRADE_UPDATE içindeki emir nesnesi.
#[derive(Debug, Clone, Default)]
pub struct OrderUpdate {
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub orig_qty: Decimal,
    pub price: Decimal,
    pub avg_price: Decimal,
    pub stop_price: Decimal,
    pub execution_type: String,
    pub status: String,
    pub order_id: i64,
    pub last_filled_qty: Decimal,
    pub cumulative_filled_qty: Decimal,
    pub last_filled_price: Decimal,
    pub commission: Decimal,
    pub commission_asset: String,
    pub transaction_time: u64,
    pub trade_id: i64,
    pub is_maker: bool,
    pub reduce_only: bool,
    pub working_type: String,
    pub orig_type: String,
    pub position_side: String,
    pub close_position: bool,
    pub activation_price: Decimal,
    pub callback_rate: Decimal,
    pub realized_profit: Decimal,
    pub price_protect: bool,
    pub status_code: i32,
}

/// MARGIN_CALL içindeki tek marj bakiyesi.
#[derive(Debug, Clone, Default)]
pub struct MarginCallBalance {
    pub symbol: String,
    pub position_side: String,
    pub position_amt: Decimal,
    pub margin_type: String,
    pub isolated_wallet: Decimal,
    pub entry_price: Decimal,
    pub un_realized_profit: Decimal,
    pub maint_margin: Decimal,
}

/// User-data stream olay sınıfı.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum UserDataEvent {
    /// listenKey süresi doldu → yeni key + tam yeniden eşitleme.
    ListenKeyExpired { event_time: u64 },
    /// Marj çağrısı (likidasyon tehdidi).
    MarginCall {
        event_time: u64,
        cross_wallet_balance: Decimal,
        balances: Vec<MarginCallBalance>,
    },
    /// Hesap deltası (bakiye + pozisyon). `reason`: ORDER / MARGIN_TRANSFER / ...
    AccountUpdate {
        event_time: u64,
        transaction_time: u64,
        update_time: u64,
        reason: String,
        balances: Vec<AccountUpdateBalance>,
        positions: Vec<AccountUpdatePosition>,
    },
    /// Emir durumu değişikliği (NEW/TRADE/CANCELED/...).
    OrderTradeUpdate {
        event_time: u64,
        transaction_time: u64,
        order: OrderUpdate,
    },
    /// Kaldıraç / marj tipi / pozisyon modu değişikliği.
    AccountConfigUpdate {
        event_time: u64,
        symbol: Option<String>,
        leverage: Option<u32>,
        margin_type: Option<String>,
        dual_side_position: Option<bool>,
    },
    /// Bilinmeyen / ayrıştırılamayan olay.
    Unknown { event_type: String, raw: serde_json::Value },
}

impl UserDataEvent {
    /// Ham JSON payload'dan olayı ayrıştırır.
    pub fn parse(raw: &serde_json::Value) -> UserDataEvent {
        let event_type = raw.get("e").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let event_time = raw.get("E").and_then(|v| v.as_u64()).unwrap_or(0);

        match event_type.as_str() {
            "listenKeyExpired" => UserDataEvent::ListenKeyExpired { event_time },
            "MARGIN_CALL" => {
                let cw = raw.get("cw").and_then(|v| v.as_str()).map(|s| s.to_string());
                let balances = raw
                    .get("p")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(parse_margin_call_balance).collect())
                    .unwrap_or_default();
                UserDataEvent::MarginCall {
                    event_time,
                    cross_wallet_balance: dec(cw.as_deref()),
                    balances,
                }
            }
            "ACCOUNT_UPDATE" => {
                let a = raw.get("a").cloned().unwrap_or(serde_json::Value::Null);
                let transaction_time = raw.get("T").and_then(|v| v.as_u64()).unwrap_or(0);
                let update_time = a.get("u").and_then(|v| v.as_u64()).unwrap_or(0);
                let reason = a.get("m").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let balances = a
                    .get("B")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(parse_account_balance).collect())
                    .unwrap_or_default();
                let positions = a
                    .get("P")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(parse_account_position).collect())
                    .unwrap_or_default();
                UserDataEvent::AccountUpdate {
                    event_time,
                    transaction_time,
                    update_time,
                    reason,
                    balances,
                    positions,
                }
            }
            "ORDER_TRADE_UPDATE" => {
                let transaction_time = raw.get("T").and_then(|v| v.as_u64()).unwrap_or(0);
                let order = raw.get("o").map(parse_order_update).unwrap_or_default();
                UserDataEvent::OrderTradeUpdate {
                    event_time,
                    transaction_time,
                    order,
                }
            }
            "ACCOUNT_CONFIG_UPDATE" => {
                let transaction_time = raw.get("T").and_then(|v| v.as_u64()).unwrap_or(0);
                let ac = raw.get("ac").cloned().unwrap_or(serde_json::Value::Null);
                let ai = raw.get("ai").cloned().unwrap_or(serde_json::Value::Null);
                let symbol = ac.get("s").and_then(|v| v.as_str()).map(|s| s.to_string());
                let leverage = ac.get("l").and_then(|v| v.as_u64()).map(|v| v as u32);
                let margin_type = ac.get("t").and_then(|v| v.as_str()).map(|s| s.to_string());
                let dual = ai.get("j").and_then(|v| v.as_bool());
                let _ = transaction_time;
                UserDataEvent::AccountConfigUpdate {
                    event_time,
                    symbol,
                    leverage,
                    margin_type,
                    dual_side_position: dual,
                }
            }
            _ => UserDataEvent::Unknown {
                event_type,
                raw: raw.clone(),
            },
        }
    }
}

fn parse_margin_call_balance(v: &serde_json::Value) -> MarginCallBalance {
    MarginCallBalance {
        symbol: v.get("s").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        position_side: v.get("ps").and_then(|x| x.as_str()).unwrap_or("BOTH").to_string(),
        position_amt: dec(v.get("pa").and_then(|x| x.as_str())),
        margin_type: v.get("mt").and_then(|x| x.as_str()).unwrap_or("cross").to_string(),
        isolated_wallet: dec(v.get("iw").and_then(|x| x.as_str())),
        entry_price: dec(v.get("mp").and_then(|x| x.as_str())),
        un_realized_profit: dec(v.get("up").and_then(|x| x.as_str())),
        maint_margin: dec(v.get("mm").and_then(|x| x.as_str())),
    }
}

fn parse_account_balance(v: &serde_json::Value) -> AccountUpdateBalance {
    AccountUpdateBalance {
        asset: v.get("a").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        wallet_balance: dec(v.get("wb").and_then(|x| x.as_str())),
        cross_wallet_balance: dec(v.get("cw").and_then(|x| x.as_str())),
    }
}

fn parse_account_position(v: &serde_json::Value) -> AccountUpdatePosition {
    AccountUpdatePosition {
        symbol: v.get("s").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        position_side: v.get("ps").and_then(|x| x.as_str()).unwrap_or("BOTH").to_string(),
        position_amt: dec(v.get("pa").and_then(|x| x.as_str())),
        entry_price: dec(v.get("ep").and_then(|x| x.as_str())),
        un_realized_profit: dec(v.get("up").and_then(|x| x.as_str())),
        margin_type: v.get("mt").and_then(|x| x.as_str()).unwrap_or("cross").to_string(),
        isolated_wallet: dec(v.get("iw").and_then(|x| x.as_str())),
    }
}

fn parse_order_update(v: &serde_json::Value) -> OrderUpdate {
    OrderUpdate {
        symbol: v.get("s").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        client_order_id: v.get("c").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        side: v.get("S").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        order_type: v.get("o").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        time_in_force: v.get("f").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        orig_qty: dec(v.get("q").and_then(|x| x.as_str())),
        price: dec(v.get("p").and_then(|x| x.as_str())),
        avg_price: dec(v.get("ap").and_then(|x| x.as_str())),
        stop_price: dec(v.get("sp").and_then(|x| x.as_str())),
        execution_type: v.get("x").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        status: v.get("X").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        order_id: v.get("i").and_then(|x| x.as_i64()).unwrap_or(0),
        last_filled_qty: dec(v.get("l").and_then(|x| x.as_str())),
        cumulative_filled_qty: dec(v.get("z").and_then(|x| x.as_str())),
        last_filled_price: dec(v.get("L").and_then(|x| x.as_str())),
        commission: dec(v.get("n").and_then(|x| x.as_str())),
        commission_asset: v.get("N").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        transaction_time: v.get("T").and_then(|x| x.as_u64()).unwrap_or(0),
        trade_id: v.get("t").and_then(|x| x.as_i64()).unwrap_or(0),
        is_maker: v.get("m").and_then(|x| x.as_bool()).unwrap_or(false),
        reduce_only: v.get("R").and_then(|x| x.as_bool()).unwrap_or(false),
        working_type: v.get("wt").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        orig_type: v.get("ot").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        position_side: v.get("ps").and_then(|x| x.as_str()).unwrap_or("BOTH").to_string(),
        close_position: v.get("cp").and_then(|x| x.as_bool()).unwrap_or(false),
        activation_price: dec(v.get("AP").and_then(|x| x.as_str())),
        callback_rate: dec(v.get("cr").and_then(|x| x.as_str())),
        realized_profit: dec(v.get("rp").and_then(|x| x.as_str())),
        price_protect: v.get("pP").and_then(|x| x.as_bool()).unwrap_or(false),
        status_code: v.get("ss").and_then(|x| x.as_i64()).map(|x| x as i32).unwrap_or(0),
    }
}
