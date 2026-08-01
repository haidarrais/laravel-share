//! WebSocket tunnel endpoint and the HTTP forwarding front-door.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tracing::{info, warn};

use crate::config::Config;
use crate::protocol::{ClientMessage, ServerMessage};
use crate::session::{RequestToClient, ResponseResult, SessionStore};

/// Shared application state for both the WS and HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<SessionStore>,
    pub config: Arc<Config>,
}

/// WebSocket upgrade handler at `/tunnel`.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<BTreeMap<String, String>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params))
}

async fn handle_socket(socket: WebSocket, state: AppState, params: BTreeMap<String, String>) {
    let subdomain_req = params.get("subdomain").map(String::as_str);
    let token = params.get("token").map(String::as_str);

    let subdomain = match state
        .store
        .reserve(state.config.token.as_deref(), subdomain_req, token)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            warn!(%e, "handshake rejected");
            return;
        }
    };

    let public_url = format!("https://{subdomain}.{}", state.config.public_host);
    let (mut ws_tx, mut ws_rx) = socket.split();

    let hello = ServerMessage::Hello {
        url: public_url.clone(),
        session_id: format!("sess-{subdomain}"),
    };
    if let Err(e) = ws_tx
        .send(Message::Text(serde_json::to_string(&hello).unwrap()))
        .await
    {
        warn!(%e, "failed to send hello");
        state.store.remove(&subdomain).await;
        return;
    }

    info!(%subdomain, %public_url, "tunnel session open");

    let store = state.store.clone();
    let mut req_rx = match store.attach_sender(&subdomain).await {
        Some(rx) => rx,
        None => {
            store.remove(&subdomain).await;
            return;
        }
    };

    // Single event loop: drive both the inbound request channel and the
    // WebSocket read stream so `ws_tx` can serve both forwarding and ponging.
    loop {
        tokio::select! {
            req = req_rx.recv() => {
                match req {
                    Some(req) => {
                        let payload = serde_json::to_string(&req.message).unwrap();
                        if ws_tx.send(Message::Text(payload)).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            msg = ws_rx.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(text) => {
                                match serde_json::from_str::<ClientMessage>(&text) {
                                    Ok(ClientMessage::Close) => break,
                                    Ok(ClientMessage::Response { id, status, headers, body }) => {
                                        if let Some(tx) = store.resolve_pending(&id).await {
                                            let body = base64::engine::general_purpose::STANDARD
                                                .decode(&body)
                                                .unwrap_or_default();
                                            let _ = tx.send(ResponseResult::Responded { status, headers, body });
                                        }
                                    }
                                    Err(e) => warn!(%e, "malformed client message"),
                                }
                            }
                            Message::Close(_) => break,
                            Message::Ping(p) => {
                                if ws_tx.send(Message::Pong(p)).await.is_err() {
                                    break;
                                }
                            }
                            Message::Binary(_) => {}
                            Message::Pong(_) => {}
                        }
                    }
                    Some(Err(e)) => {
                        warn!(%e, "websocket read error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    store.remove(&subdomain).await;
    info!(%subdomain, "tunnel session closed, url released");
}

/// HTTP front-door: forwards requests to the owning tunnel.
pub async fn http_app(State(state): State<AppState>, req: axum::extract::Request) -> Response {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let subdomain = host.split('.').next().unwrap_or_default().to_string();
    if subdomain.is_empty() {
        return (StatusCode::NOT_FOUND, "No active tunnel for this host.").into_response();
    }

    let sender = match state.store.sender_for(&subdomain).await {
        Some(s) => s,
        None => {
            return (StatusCode::NOT_FOUND, "No active tunnel for this host.").into_response();
        }
    };

    forward_request(state, sender, req).await
}

async fn forward_request(
    state: AppState,
    sender: tokio::sync::mpsc::Sender<RequestToClient>,
    req: axum::extract::Request,
) -> Response {
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, crate::protocol::MESSAGE_MAX_BYTES).await {
        Ok(b) => b.to_vec(),
        Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "payload too large").into_response(),
    };

    let id = uuid::Uuid::new_v4().simple().to_string();
    let path = parts.uri.path().to_string();
    let query = parts.uri.query().unwrap_or_default().to_string();
    let headers: BTreeMap<String, String> = parts
        .headers
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("<non-utf8>").to_string(),
            )
        })
        .collect();

    let message = ServerMessage::Request {
        id: id.clone(),
        method: parts.method.as_str().to_string(),
        path,
        query,
        headers,
        body: base64::engine::general_purpose::STANDARD.encode(&bytes),
    };

    let rx = state.store.register_pending(&id).await;
    if sender.send(RequestToClient { message }).await.is_err() {
        return (StatusCode::BAD_GATEWAY, "tunnel client disconnected").into_response();
    }

    match rx.await {
        Ok(ResponseResult::Responded {
            status,
            headers,
            body,
        }) => {
            let mut response = Response::new(Body::from(body));
            *response.status_mut() =
                StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
            for (k, v) in headers {
                if k.eq_ignore_ascii_case("host") || k.eq_ignore_ascii_case("content-length") {
                    continue;
                }
                if let (Ok(kv), Ok(val)) = (
                    HeaderName::from_bytes(k.as_bytes()),
                    v.parse::<HeaderValue>(),
                ) {
                    response.headers_mut().insert(kv, val);
                }
            }
            response
        }
        Ok(ResponseResult::Failed(e)) => {
            warn!(%e, "request forward failed");
            (StatusCode::BAD_GATEWAY, "tunnel client unavailable").into_response()
        }
        Err(e) => {
            warn!(%e, "request forward dropped");
            (StatusCode::GATEWAY_TIMEOUT, "tunnel client unavailable").into_response()
        }
    }
}
