//! Wire protocol for the `relay` driver (mirrors the relay-server crate).
//!
//! Newline-delimited JSON over the WebSocket. See
//! `crates/relay-server/src/protocol.rs` for the authoritative spec.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        url: String,
        #[allow(dead_code)]
        session_id: String,
    },
    Request {
        id: String,
        method: String,
        path: String,
        #[serde(default)]
        query: String,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        body: String, // base64
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Response {
        id: String,
        status: u16,
        #[serde(default)]
        headers: BTreeMap<String, String>,
        #[serde(default)]
        body: String, // base64
    },
    Close,
}

/// An inbound webhook request captured for the terminal log and inspector.
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub id: String,
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
    pub received_at: chrono::DateTime<chrono::Local>,
}
