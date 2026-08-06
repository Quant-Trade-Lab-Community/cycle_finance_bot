use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use crate::rpc::metrics_collector::SharedMetrics;
use crossbeam_channel::Sender;
use serde::Deserialize;

#[derive(RustEmbed)]
#[folder = "../admin-ui/dist/"]
struct Assets;

pub struct AppState {
    pub metrics: Arc<SharedMetrics>,
    pub cmd_tx: Sender<AdminCommand>,
}

#[derive(Deserialize, Debug)]
pub struct AdminCommand {
    pub cmd: String,
}

pub async fn start_rpc_server(metrics: Arc<SharedMetrics>, cmd_tx: Sender<AdminCommand>) {
    let state = Arc::new(AppState { metrics, cmd_tx });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/assets/*file", get(static_handler))
        .route("/ws", get(ws_handler))
        .with_state(state);

    println!("Admin RPC Server listening on 0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> impl IntoResponse {
    let index = Assets::get("index.html").unwrap();
    Html(index.data)
}

async fn static_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => (axum::http::StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // We send metrics continuously every 100ms via Postcard binary format
    let mut metrics_interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = metrics_interval.tick() => {
                let snapshot = state.metrics.snapshot();
                if let Ok(bytes) = postcard::to_allocvec(&snapshot) {
                    if socket.send(Message::Binary(bytes.into())).await.is_err() {
                        break;
                    }
                }
            }
            msg = socket.recv() => {
                if let Some(Ok(Message::Binary(bytes))) = msg {
                    if let Ok(cmd) = postcard::from_bytes::<AdminCommand>(&bytes) {
                        let _ = state.cmd_tx.try_send(cmd);
                    }
                } else if let Some(Ok(Message::Text(txt))) = msg {
                     if let Ok(cmd) = serde_json::from_str::<AdminCommand>(&txt) {
                        let _ = state.cmd_tx.try_send(cmd);
                    }
                } else {
                    break;
                }
            }
        }
    }
}
