//! User-data stream istemcisi.
//!
//! listenKey yaşam döngüsü (üret/keepalive/sil), WS bağlantısı, üstel geri
//! çekilme ile yeniden bağlanma, gzip çözümü ve olayların actor'e iletimi.
//! Her (yeniden) bağlantıda `StreamConnected` gönderilir → actor tam resync yapar.

use crate::client::BinanceClient;
use crate::config::ExecConfig;
use crate::execution::actor::UserEvent;
use crate::user_data::decoder::{as_event, decode_message, user_event_type};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

const BASE_BACKOFF_SEC: u64 = 1;
const MAX_BACKOFF_SEC: u64 = 60;

pub struct UserDataStream {
    client: Arc<BinanceClient>,
    config: ExecConfig,
    user_tx: mpsc::UnboundedSender<UserEvent>,
}

impl UserDataStream {
    pub fn new(
        client: Arc<BinanceClient>,
        config: ExecConfig,
        user_tx: mpsc::UnboundedSender<UserEvent>,
    ) -> Self {
        Self {
            client,
            config,
            user_tx,
        }
    }

    pub async fn run(self) {
        let mut backoff_sec = BASE_BACKOFF_SEC;

        loop {
            let listen_key = match self.client.create_listen_key().await {
                Ok(k) => k,
                Err(e) => {
                    warn!("listenKey üretilemedi: {e} — {}s sonra", backoff_sec);
                    tokio::time::sleep(Duration::from_secs(backoff_sec)).await;
                    backoff_sec = (backoff_sec * 2).min(MAX_BACKOFF_SEC);
                    continue;
                }
            };
            backoff_sec = BASE_BACKOFF_SEC;

            let url = format!("{}/ws/{}", self.config.ws_url.trim_end_matches('/'), listen_key);
            info!("User-data stream bağlanıyor: {url}");

            match connect_async(&url).await {
                Ok((ws, _)) => {
                    info!("User-data stream bağlandı");
                    // Bağlantı kuruldu: actor tam yeniden eşitleme yapsın.
                    let _ = self.user_tx.send(UserEvent::StreamConnected);
                    self.run_connection(ws, listen_key.clone()).await;
                    // Bağlantı kapandı.
                }
                Err(e) => {
                    warn!("User-data stream bağlantı hatası: {e} — {}s sonra", backoff_sec);
                    tokio::time::sleep(Duration::from_secs(backoff_sec)).await;
                    backoff_sec = (backoff_sec * 2).min(MAX_BACKOFF_SEC);
                    continue;
                }
            }

            // Bağlantı kapandı: bir sonraki döngüde yeni listenKey.
            info!("User-data stream kapandı; yeniden bağlanılıyor");
        }
    }

    async fn run_connection(&self, ws: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, listen_key: String) {
        let (mut write, mut read) = ws.split();

        let keepalive_sec = self.config.listen_key_keepalive_sec.max(60);
        let mut keepalive = tokio::time::interval(Duration::from_secs(keepalive_sec));
        keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = keepalive.tick() => {
                    if let Err(e) = self.client.refresh_listen_key(&listen_key).await {
                        warn!("listenKey keepalive hatası: {e}");
                    }
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(message)) => {
                            match message {
                                Message::Binary(data) => {
                                    if let Some(ev) = self.handle_payload(&data, false) {
                                        let _ = self.user_tx.send(ev);
                                    }
                                }
                                Message::Text(text) => {
                                    if let Some(ev) = self.handle_payload(text.as_bytes(), true) {
                                        let _ = self.user_tx.send(ev);
                                    }
                                }
                                Message::Ping(data) => {
                                    let _ = write.send(Message::Pong(data)).await;
                                }
                                Message::Close(_) => {
                                    info!("User-data stream sunucu tarafından kapatıldı");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        Some(Err(e)) => {
                            warn!("User-data stream okuma hatası: {e}");
                            break;
                        }
                        None => {
                            info!("User-data stream bitti");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Payload'ı çözer, olayı üretir. listenKeyExpired → bağlantıyı bitir (yeni key).
    fn handle_payload(&self, bytes: &[u8], is_text: bool) -> Option<UserEvent> {
        match decode_message(bytes, is_text) {
            Ok(value) => {
                let etype = user_event_type(&value);
                let ev = as_event(&value);
                if etype == "listenKeyExpired" {
                    warn!("listenKeyExpired — listenKey yenilenecek");
                    // Yeni key için bağlantıyı kapat (run döngüsü yeni key üretir).
                    let _ = self.user_tx.send(UserEvent::Data(ev));
                    None
                } else {
                    Some(UserEvent::Data(ev))
                }
            }
            Err(e) => {
                warn!("User-data payload ayrıştırılamadı: {e}");
                None
            }
        }
    }
}
