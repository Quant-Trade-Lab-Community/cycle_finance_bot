//! Ham user-data stream payload'ını ayrıştırır.
//!
//! Binance futures user-data akışı gzip sıkıştırılmış binary frame'ler gönderir.
//! Gzip çözümü başarısız olursa düz metin JSON olarak denenir (savunmacı).

use crate::error::{ExecError, Result};
use flate2::read::GzDecoder;
use serde_json::Value;
use std::io::Read;

pub fn decode_binary(bytes: &[u8]) -> Result<Value> {
    // Önce gzip.
    let mut decoder = GzDecoder::new(bytes);
    let mut out = Vec::new();
    match decoder.read_to_end(&mut out) {
        Ok(_) if !out.is_empty() => {
            if let Ok(v) = serde_json::from_slice(&out) {
                return Ok(v);
            }
        }
        _ => {}
    }
    // Gzip değilse ham JSON (binary frame olarak gelmiş olabilir).
    serde_json::from_slice(bytes).map_err(ExecError::Json)
}

pub fn decode_text(text: &str) -> Result<Value> {
    serde_json::from_str(text).map_err(ExecError::Json)
}

/// Paylaşımlı ayrıştırma: ister binary ister text olsun.
pub fn decode_message(bytes: &[u8], is_text: bool) -> Result<Value> {
    if is_text {
        decode_text(std::str::from_utf8(bytes).unwrap_or(""))
    } else {
        decode_binary(bytes)
    }
}

pub fn as_event(value: &Value) -> crate::types::user_event::UserDataEvent {
    crate::types::user_event::UserDataEvent::parse(value)
}

/// Olay tipi etiketi ("e" alanı).
pub fn user_event_type(value: &Value) -> String {
    value
        .get("e")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn gzip_binary_decodes() {
        let json = r#"{"e":"ORDER_TRADE_UPDATE","E":1}"#;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(json.as_bytes()).unwrap();
        let bytes = enc.finish().unwrap();
        let v = decode_binary(&bytes).unwrap();
        assert_eq!(v["e"], "ORDER_TRADE_UPDATE");
    }

    #[test]
    fn plain_json_binary_fallback() {
        let json = r#"{"e":"listenKeyExpired","E":2}"#;
        let v = decode_binary(json.as_bytes()).unwrap();
        assert_eq!(v["e"], "listenKeyExpired");
    }

    #[test]
    fn text_decodes() {
        let v = decode_text(r#"{"e":"ACCOUNT_UPDATE"}"#).unwrap();
        assert_eq!(v["e"], "ACCOUNT_UPDATE");
    }
}
