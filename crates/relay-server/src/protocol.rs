//! Wire protocol for the `relay` driver.
//!
//! All messages are newline-delimited JSON over the WebSocket.
//!
//! Client → Server handshake (query params on the WSS URL):
//!   - `token`      optional per-session auth token configured on the operator's instance
//!   - `subdomain`  optional reserved subdomain (e.g. `swift-otter-42`)
//!
//! Server → Client:
//!   - `{"type":"hello","url":"https://<subdomain>.<host>","session_id":"..."}`
//!   - `{"type":"request","id":"...","method":"GET","path":"/...","headers":{...},"body":"<b64>","query":"..."}`
//!   - `{"type":"error","message":"..."}`
//!
//! Client → Server:
//!   - `{"type":"response","id":"...","status":200,"headers":{...},"body":"<b64>"}`
//!   - `{"type":"close"}`

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MESSAGE_MAX_BYTES: usize = 16 * 1024 * 1024; // 16 MiB payload ceiling

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        url: String,
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
        body: String, // base64, empty when no body
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_roundtrips() {
        let msg = ServerMessage::Hello {
            url: "https://swift-otter-42.example.dev".into(),
            session_id: "sess-1".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"hello""#));
        let back: ServerMessage = serde_json::from_str(&json).unwrap();
        match back {
            ServerMessage::Hello { url, .. } => {
                assert_eq!(url, "https://swift-otter-42.example.dev")
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn request_roundtrips_with_defaults() {
        let json = r#"{"type":"request","id":"r1","method":"POST","path":"/hooks/stripe","headers":{},"body":""}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::Request { query, body, .. } => {
                assert_eq!(query, "");
                assert_eq!(body, "");
            }
            _ => panic!("wrong variant"),
        }
    }
}
