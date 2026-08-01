//! Webhook-aware terminal logger and in-memory request/response store shared
//! with the local web inspector.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::protocol::CapturedRequest;
use crate::redact::{redact_body, redact_headers};

/// Provider detection: recognize well-known webhook signing headers so the log
/// line can be labelled without altering or validating the payload.
pub fn detect_provider(headers: &BTreeMap<String, String>) -> Option<&'static str> {
    for k in headers.keys() {
        match k.to_ascii_lowercase().as_str() {
            "stripe-signature" => return Some("stripe"),
            "x-hub-signature-256" => return Some("github"),
            "x-slack-signature" => return Some("slack"),
            "x-shopify-hmac-sha256" => return Some("shopify"),
            "x-twilio-signature" => return Some("twilio"),
            "x-paypal-transmission-id" => return Some("paypal"),
            _ => {}
        }
    }
    None
}

/// A single captured exchange, stored for the inspector and for replay.
#[derive(Debug, Clone)]
pub struct CapturedExchange {
    pub request: CapturedRequest,
    pub status: u16,
    pub latency_ms: u128,
}

/// Thread-safe store of recent exchanges, bounded to avoid unbounded growth.
#[derive(Clone, Default)]
pub struct ExchangeStore {
    inner: Arc<Mutex<Vec<CapturedExchange>>>,
    pub verbose: Arc<Mutex<bool>>,
}

impl ExchangeStore {
    pub fn new(verbose: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            verbose: Arc::new(Mutex::new(verbose)),
        }
    }

    pub fn push(&self, exchange: CapturedExchange) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(exchange);
        if guard.len() > 1000 {
            let excess = guard.len() - 1000;
            guard.drain(..excess);
        }
    }

    pub fn all(&self) -> Vec<CapturedExchange> {
        self.inner.lock().unwrap().clone()
    }

    pub fn by_id(&self, id: &str) -> Option<CapturedExchange> {
        self.inner
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.request.id == id)
            .cloned()
    }
}

/// Pretty-print a captured request to the terminal, provider-aware and redacted.
pub fn print_exchange(store: &ExchangeStore, exchange: &CapturedExchange) {
    let req = &exchange.request;
    let provider = detect_provider(&req.headers).unwrap_or("http");
    let verbose = *store.verbose.lock().unwrap();

    // Compact one-line summary with drill-down.
    println!(
        "\x1b[2m{:<12}\x1b[0m \x1b[36m{:<7}\x1b[0m {} \x1b[2m{}\x1b[0m \x1b[33m{}\x1b[0m \x1b[2m{}ms\x1b[0m \x1b[35m[{provider}]\x1b[0m",
        format_time(req.received_at),
        req.method,
        req.path,
        if req.query.is_empty() { String::new() } else { format!("?{}", req.query) },
        exchange.status,
        exchange.latency_ms,
    );

    if verbose {
        let headers = redact_headers(&req.headers, true);
        for (k, v) in headers {
            println!("  \x1b[2m{}\x1b[0m: {v}", k.to_ascii_lowercase());
        }
    }

    // Print the redacted body (full headers only with --verbose; body always
    // redacted for secret shapes).
    if !req.body.is_empty() {
        let body_str = String::from_utf8_lossy(&req.body);
        let redacted = redact_body(&body_str);
        if let Ok(pretty) = serde_json::from_str::<serde_json::Value>(&redacted) {
            println!(
                "  {}",
                serde_json::to_string_pretty(&pretty).unwrap_or(redacted)
            );
        } else {
            println!("  {redacted}");
        }
    }
}

/// Line used when a forward to the local app failed (e.g. app not running).
pub fn print_forward_error(exchange: &CapturedRequest, err: &str) {
    println!(
        "\x1b[31m✗\x1b[0m {} {} \x1b[31m{err}\x1b[0m",
        exchange.method, exchange.path
    );
}

fn format_time(t: chrono::DateTime<chrono::Local>) -> String {
    t.format("%H:%M:%S").to_string()
}
