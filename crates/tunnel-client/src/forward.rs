//! Forwards an inbound webhook request to the local app and captures the
//! response, measuring latency for the log line.

use std::collections::BTreeMap;
use std::time::Instant;

use base64::Engine;
use serde::{Deserialize, Serialize};

/// The result of forwarding a request to the local app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u128,
}

/// Send an inbound webhook to `http://localhost:<port><path>?<query>`.
///
/// `basic_auth` is an optional `user:pass` applied to the origin request when
/// the tunnel was started with `--auth` (used by hosts that gate their webhook
/// endpoint on HTTP basic auth).
pub async fn forward(
    local_port: u16,
    method: &str,
    path: &str,
    query: &str,
    headers: &BTreeMap<String, String>,
    body: Vec<u8>,
    basic_auth: Option<&str>,
) -> anyhow::Result<ForwardedResponse> {
    let started = Instant::now();
    let client = reqwest::Client::new();

    let url = format!(
        "http://localhost:{local_port}{}{}",
        path,
        if query.is_empty() {
            String::new()
        } else {
            format!("?{query}")
        }
    );

    let mut req = client.request(
        reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
        &url,
    );

    for (k, v) in headers {
        // Skip hop-by-hop / proxy headers we must not echo to the origin.
        if matches!(
            k.to_ascii_lowercase().as_str(),
            "host" | "connection" | "upgrade" | "x-forwarded-for" | "x-forwarded-proto"
        ) {
            continue;
        }
        req = req.header(k, v);
    }
    req = req.header("X-Forwarded-Proto", "https");
    req = req.header("X-Artisan-Share", "1");

    if let Some(cred) = basic_auth {
        if !headers
            .keys()
            .any(|k| k.eq_ignore_ascii_case("authorization"))
        {
            let encoded = base64::engine::general_purpose::STANDARD.encode(cred.as_bytes());
            req = req.header("Authorization", format!("Basic {encoded}"));
        }
    }

    if !body.is_empty() {
        req = req.body(body);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("forward failed: {e}"))?;
    let status = resp.status().as_u16();
    let headers_out = resp
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("<non-utf8>").to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let bytes = resp.bytes().await.unwrap_or_default().to_vec();

    Ok(ForwardedResponse {
        status,
        headers: headers_out,
        body: bytes,
        latency_ms: started.elapsed().as_millis(),
    })
}
