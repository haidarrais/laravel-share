//! The `relay` driver: connects to the user's self-hosted relay-server over
//! WSS, prints the assigned public URL, forwards inbound requests to
//! `localhost:<port>`, and streams the log. Reconnects with backoff and keeps
//! the session idempotent so the URL survives short drops.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::cli::Config;
use crate::forward;
use crate::log::{print_exchange, print_forward_error, CapturedExchange, ExchangeStore};
use crate::protocol::{CapturedRequest, ClientMessage, ServerMessage};

const MAX_BACKOFF: Duration = Duration::from_secs(30);

pub async fn run(config: Config, store: Arc<ExchangeStore>) -> anyhow::Result<()> {
    // Resolve the WS endpoint (accept http/https scheme too, upgrading to ws/wss).
    let endpoint = normalize_endpoint(&config.relay.endpoint);

    let mut attempt: u32 = 0;
    loop {
        let url = build_url(&endpoint, &config, attempt == 0);
        match connect_async(url.as_str()).await {
            Ok((ws_stream, _resp)) => {
                attempt = 0;
                if let Err(e) = handle_session(ws_stream, &config, &store).await {
                    eprintln!("relay session ended: {e}");
                } else {
                    println!("\x1b[2mrelay session closed by server\x1b[0m");
                }
            }
            Err(e) => {
                eprintln!("relay connect failed: {e}");
            }
        }
        // Backoff before retry.
        let delay = backoff_for(attempt);
        attempt = attempt.saturating_add(1);
        tokio::time::sleep(delay).await;
    }
}

fn normalize_endpoint(endpoint: &str) -> String {
    let mut normalized = if endpoint.starts_with("http://") {
        endpoint.replacen("http://", "ws://", 1)
    } else if endpoint.starts_with("https://") {
        endpoint.replacen("https://", "wss://", 1)
    } else {
        endpoint.to_string()
    };

    // Route to the tunnel handshake endpoint when the configured endpoint is a
    // bare origin (no path).
    let has_path = url::Url::parse(&normalized)
        .map(|u| u.path() != "/" && !u.path().is_empty())
        .unwrap_or(false);
    if !has_path {
        normalized.push_str("/tunnel");
    }
    normalized
}

fn build_url(endpoint: &str, config: &Config, include_subdomain: bool) -> url::Url {
    let mut url = url::Url::parse(endpoint).expect("relay endpoint must be a valid URL");
    // Only request a subdomain on the first attempt; reconnect uses a fresh
    // random one so a stale session never hijacks a reserved name.
    if include_subdomain {
        if let Some(s) = &config.subdomain {
            url.query_pairs_mut().append_pair("subdomain", s);
        }
    }
    if let Some(t) = &config.relay.token {
        url.query_pairs_mut().append_pair("token", t);
    }
    url
}

fn backoff_for(attempt: u32) -> Duration {
    let base_ms = 500u64.saturating_mul(2u64.saturating_pow(attempt.min(6)));
    Duration::from_millis(base_ms.min(MAX_BACKOFF.as_millis() as u64))
}

async fn handle_session<S>(
    ws_stream: tokio_tungstenite::WebSocketStream<S>,
    config: &Config,
    store: &Arc<ExchangeStore>,
) -> anyhow::Result<()>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Await the hello handshake.
    let public_url = match ws_rx.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<ServerMessage>(&text)? {
            ServerMessage::Hello { url, .. } => url,
            ServerMessage::Error { message } => anyhow::bail!("relay handshake error: {message}"),
            _ => anyhow::bail!("unexpected first message from relay"),
        },
        Some(Ok(_)) => anyhow::bail!("unexpected non-text message from relay"),
        Some(Err(e)) => return Err(anyhow::anyhow!("relay socket error: {e}")),
        None => anyhow::bail!("relay closed during handshake"),
    };

    println!(
        "\x1b[1mForwarding\x1b[0m   {public_url} -> http://localhost:{}",
        config.local_port
    );
    println!("\x1b[2mPress Ctrl+C to stop\x1b[0m");

    // Read loop: handle inbound requests, forwarding to localhost.
    while let Some(Ok(msg)) = ws_rx.next().await {
        match msg {
            Message::Text(text) => {
                let server_msg: ServerMessage = serde_json::from_str(&text)?;
                match server_msg {
                    ServerMessage::Request {
                        id,
                        method,
                        path,
                        query,
                        headers,
                        body,
                    } => {
                        let body_bytes = base64::engine::general_purpose::STANDARD
                            .decode(&body)
                            .unwrap_or_default();
                        let captured = CapturedRequest {
                            id: id.clone(),
                            method: method.clone(),
                            path: path.clone(),
                            query: query.clone(),
                            headers: headers.clone(),
                            body: body_bytes.clone(),
                            received_at: chrono::Local::now(),
                        };

                        match forward::forward(
                            config.local_port,
                            &method,
                            &path,
                            &query,
                            &headers,
                            body_bytes,
                            config.basic_auth.as_deref(),
                        )
                        .await
                        {
                            Ok(resp) => {
                                let exchange = CapturedExchange {
                                    request: captured,
                                    status: resp.status,
                                    latency_ms: resp.latency_ms,
                                };
                                print_exchange(store, &exchange);
                                store.push(exchange);

                                let reply = ClientMessage::Response {
                                    id,
                                    status: resp.status,
                                    headers: resp.headers,
                                    body: base64::engine::general_purpose::STANDARD
                                        .encode(&resp.body),
                                };
                                ws_tx
                                    .send(Message::Text(serde_json::to_string(&reply)?))
                                    .await?;
                            }
                            Err(e) => {
                                print_forward_error(&captured, &e.to_string());
                                // Return a 502 so the provider gets a clear signal.
                                let reply = ClientMessage::Response {
                                    id,
                                    status: 502,
                                    headers: BTreeMap::new(),
                                    body: String::new(),
                                };
                                ws_tx
                                    .send(Message::Text(serde_json::to_string(&reply)?))
                                    .await?;
                            }
                        }
                    }
                    ServerMessage::Error { message } => {
                        eprintln!("relay error: {message}");
                    }
                    ServerMessage::Hello { .. } => {
                        // Ignore duplicate hellos.
                    }
                }
            }
            Message::Close(_) => break,
            Message::Ping(p) => {
                ws_tx.send(Message::Pong(p)).await?;
            }
            Message::Pong(_) => {}
            Message::Binary(_) => {}
            Message::Frame(_) => {}
        }
    }

    // Gracefully signal the relay to release the subdomain.
    let _ = ws_tx
        .send(Message::Text(serde_json::to_string(&ClientMessage::Close)?))
        .await;

    Ok(())
}
