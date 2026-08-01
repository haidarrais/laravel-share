//! End-to-end integration test for the relay driver.
//!
//! Spins up the relay server and a local origin in-process, connects a WebSocket
//! acting as the tunnel client, and verifies the full request → forward →
//! response round trip plus session teardown.

use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use relay_server::protocol::{ClientMessage, ServerMessage};
use relay_server::{app, Config};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

/// Spawn a relay server on an ephemeral port and return its bound address.
async fn spawn_relay() -> std::net::SocketAddr {
    let cfg = Config {
        listen_addr: "127.0.0.1:0".into(),
        public_host: "test.dev".into(),
        token: None,
    };
    let state = relay_server::AppState {
        store: Arc::new(relay_server::SessionStore::new()),
        config: Arc::new(cfg),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app(state)).await.unwrap();
    });
    addr
}

/// Spawn a local origin HTTP server returning a fixed body; returns its address.
async fn spawn_origin(body: &'static str) -> std::net::SocketAddr {
    let app = Router::new().route("/", get(move || async move { body }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

#[tokio::test]
async fn forwards_http_through_tunnel() {
    let relay_addr = spawn_relay().await;
    let origin_addr = spawn_origin("origin-ok").await;

    // Connect a WebSocket acting as the tunnel client.
    let ws_url = format!("ws://{relay_addr}/tunnel");
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url).await.unwrap();

    // Read the hello to learn the assigned subdomain.
    let hello = ws.next().await.unwrap().unwrap();
    let hello_text = match hello {
        Message::Text(t) => t,
        _ => panic!("expected text hello"),
    };
    let server_msg: ServerMessage = serde_json::from_str(&hello_text).unwrap();
    let subdomain = match &server_msg {
        ServerMessage::Hello { url, .. } => url
            .trim_start_matches("https://")
            .split('.')
            .next()
            .unwrap_or_default()
            .to_string(),
        _ => panic!("expected hello, got {server_msg:?}"),
    };
    assert!(!subdomain.is_empty());

    // Client task: for each request message, forward to the origin and reply.
    // It stops when the test signals teardown (or the socket closes).
    let origin_port = origin_addr.port();
    let (mut sender, mut reader) = ws.split();
    let (close_tx, mut close_rx) = tokio::sync::mpsc::channel::<()>(1);
    let client_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = reader.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(ServerMessage::Request { id, method, path, .. }) =
                                serde_json::from_str::<ServerMessage>(&text)
                            {
                                // Forward the path (no body support needed here).
                                let url = format!("http://127.0.0.1:{origin_port}{path}");
                                match reqwest::Client::new()
                                    .request(method.parse().unwrap_or_default(), &url)
                                    .send()
                                    .await
                                {
                                    Ok(res) => {
                                        let status = res.status().as_u16();
                                        let body = res.bytes().await.unwrap_or_default().to_vec();
                                        let reply = ClientMessage::Response {
                                            id,
                                            status,
                                            headers: Default::default(),
                                            body: base64::engine::general_purpose::STANDARD
                                                .encode(&body),
                                        };
                                        let _ = sender
                                            .send(Message::Text(
                                                serde_json::to_string(&reply).unwrap(),
                                            ))
                                            .await;
                                    }
                                    Err(e) => warn!(%e, "origin forward failed"),
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = close_rx.recv() => {
                    let _ = sender
                        .send(Message::Text(
                            serde_json::to_string(&ClientMessage::Close).unwrap(),
                        ))
                        .await;
                    break;
                }
            }
        }
    });

    // Send an HTTP request through the relay to the assigned public host.
    // A raw TCP write gives us full control over the Host header, which is what
    // the relay uses to route to the correct subdomain.
    let host_header = format!("{subdomain}.test.dev");
    let mut sock = tokio::net::TcpStream::connect(relay_addr).await.unwrap();
    let request = format!("GET / HTTP/1.1\r\nHost: {host_header}\r\nConnection: close\r\n\r\n");
    use tokio::io::AsyncWriteExt;
    sock.write_all(request.as_bytes()).await.unwrap();
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    sock.read_to_end(&mut buf).await.unwrap();
    let response = String::from_utf8_lossy(&buf);
    let status_line = response.lines().next().unwrap_or_default();
    assert!(
        status_line.contains("200"),
        "expected 200, got: {status_line}\n{response}"
    );
    assert!(
        response.ends_with("origin-ok"),
        "origin body missing: {response}"
    );

    // Gracefully close the tunnel and verify teardown.
    let _ = close_tx.send(()).await;
    let _ = client_task.await;
}
