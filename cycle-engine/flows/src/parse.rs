//! Akış ayrıştırıcı — her akış kendi stream'ini kendi event tipine çevirir.
//!
//! Mevcut `EventParser`'ın tanıdığı stream'ler (`@trade`, `@depth`,
//! `@forceOrder`, `@markPrice`) doğrudan yeniden kullanılır; yeni stream'ler
//! (`!openInterest@arr`, `@lastPrice`, `@indexPrice`) için ek ayrıştırıcılar
//! vardır. Yeni event tipi EKLENMEZ — mevcut tiplere eşlenir (torn-read/ring
//! kuralları ve dış tüketici match'leri bozulmaz).

use pipeline::tick::EventParser;
use rust_decimal::prelude::*;
use simd_json;
use simd_json::prelude::*;
use transport::events::OwnedEvent;
use transport::flow::FlowKind;

/// Bir ham WS mesajını bu akışa ait `OwnedEvent`(ler)e çevirir.
/// Tekli stream'ler 0..1 event, `!openInterest@arr` çoklu event döner.
pub fn parse_for(kind: FlowKind, bytes: &mut [u8]) -> Vec<OwnedEvent> {
    match kind {
        FlowKind::OpenInterest => parse_open_interest(bytes),
        FlowKind::LastPrice => parse_price_like(bytes, PriceField::Last),
        FlowKind::IndexPrice => parse_price_like(bytes, PriceField::Index),
        _ => EventParser::parse(bytes).into_iter().collect(),
    }
}

enum PriceField {
    Last,
    Index,
}

/// `@lastPrice@1s` ve `@indexPrice@1s` → `FundingRate` (mevcut event).
///
/// - lastPrice: `p` → mark_price alanına taşınır (lastprice tablosu `price` olarak yazar).
/// - indexPrice: `i` → index_price alanına taşınır (indexprice tablosu `price` olarak yazar).
///
/// `FundingRate` seçildi çünkü `DataValidator` bu tip için ek kural
/// uygulamaz; böylece mevcut doğrulama kuralları bozulmaz.
fn parse_price_like(bytes: &mut [u8], field: PriceField) -> Vec<OwnedEvent> {
    let mut out = Vec::new();
    let parsed = match simd_json::to_borrowed_value(bytes) {
        Ok(v) => v,
        Err(_) => return out,
    };
    let data = match parsed.get("data") {
        Some(d) => d,
        None => return out,
    };
    let (Some(sym), Some(raw)) = (
        data.get("s").and_then(|v| v.as_str()),
        match field {
            PriceField::Last => data.get("p").and_then(|v| v.as_str()),
            PriceField::Index => data.get("i").and_then(|v| v.as_str()),
        },
    ) else {
        return out;
    };
    let price = match Decimal::from_str(raw) {
        Ok(v) => v,
        Err(_) => return out,
    };
    let ts = data.get("E").and_then(|v| v.as_u64()).unwrap_or(0);

    match field {
        PriceField::Last => out.push(OwnedEvent::new_funding_rate(sym, price, Decimal::ZERO, Decimal::ZERO, ts)),
        PriceField::Index => out.push(OwnedEvent::new_funding_rate(sym, Decimal::ZERO, price, Decimal::ZERO, ts)),
    }
    out
}

/// `!openInterest@arr` (tüm semboller, 5 sn) — `data` bir dizidir; her öğe
/// `OpenInterest` event'ine çevrilir.
fn parse_open_interest(bytes: &mut [u8]) -> Vec<OwnedEvent> {
    let mut out = Vec::new();
    let parsed = match simd_json::to_borrowed_value(bytes) {
        Ok(v) => v,
        Err(_) => return out,
    };
    let arr = match parsed.get("data").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return out,
    };
    for item in arr {
        let (Some(sym), Some(oi)) = (
            item.get("s").and_then(|v| v.as_str()),
            item.get("i").and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        let oi = match Decimal::from_str(oi) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ts = item.get("E").and_then(|v| v.as_u64()).unwrap_or(0);
        out.push(OwnedEvent::new_open_interest(sym, oi, ts));
    }
    out
}
