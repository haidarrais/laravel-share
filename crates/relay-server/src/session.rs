//! In-memory session store and pending-request registry.
//!
//! By default the relay keeps only ephemeral in-memory session state. There is
//! no persistence backend in v0.1; the optional `request_logs`/`auth_tokens`
//! tables described in the PRD are documented future work for operators who
//! opt into persistence.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Mutex};

/// A live tunnel session, owned by a single WebSocket connection.
pub struct Session {
    /// Sends requests to the connected client.
    pub sender: mpsc::Sender<RequestToClient>,
}

/// A request to be delivered to the connected client. The corresponding
/// `oneshot::Sender<ResponseResult>` lives in the `pending` registry (keyed by
/// request id) so the reader task can resolve it when the client replies.
pub struct RequestToClient {
    pub message: crate::protocol::ServerMessage,
}

/// The outcome of forwarding a request to a client.
pub enum ResponseResult {
    /// The client responded successfully.
    Responded {
        status: u16,
        headers: std::collections::BTreeMap<String, String>,
        body: Vec<u8>,
    },
    /// The client connection was gone or the forward failed.
    Failed(String),
}

#[derive(Clone, Default)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    pending: Arc<Mutex<HashMap<String, oneshot::Sender<ResponseResult>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reserve a subdomain for a session.
    ///
    /// Fails if the subdomain is already reserved, or if the instance requires
    /// a token and the presented token does not match.
    pub async fn reserve(
        &self,
        token_required: Option<&str>,
        requested_subdomain: Option<&str>,
        presented_token: Option<&str>,
    ) -> anyhow::Result<String> {
        if let Some(required) = token_required {
            if presented_token != Some(required) {
                anyhow::bail!("invalid or missing token");
            }
        }

        let subdomain = match requested_subdomain {
            Some(s) if !s.is_empty() => s.to_string(),
            _ => new_subdomain(),
        };

        let mut guard = self.sessions.lock().await;
        if guard.contains_key(&subdomain) {
            anyhow::bail!("subdomain already in use: {subdomain}");
        }
        guard.insert(
            subdomain.clone(),
            Session {
                sender: mpsc::channel::<RequestToClient>(64).0,
            },
        );
        Ok(subdomain)
    }

    /// Replace the request sender for a live session with a freshly built one,
    /// returning the matching receiver. Called by the WS handler when it is
    /// ready to accept requests.
    pub async fn attach_sender(&self, subdomain: &str) -> Option<mpsc::Receiver<RequestToClient>> {
        let (tx, rx) = mpsc::channel::<RequestToClient>(64);
        let mut guard = self.sessions.lock().await;
        guard.get_mut(subdomain)?.sender = tx;
        Some(rx)
    }

    pub async fn remove(&self, subdomain: &str) {
        self.sessions.lock().await.remove(subdomain);
    }

    /// Fetch a clone of the sender for a subdomain so the HTTP handler can
    /// forward without holding the store lock across the await.
    pub async fn sender_for(&self, subdomain: &str) -> Option<mpsc::Sender<RequestToClient>> {
        let guard = self.sessions.lock().await;
        guard.get(subdomain).map(|s| s.sender.clone())
    }

    /// Register a pending request and return the receiver for its response.
    pub async fn register_pending(&self, id: &str) -> oneshot::Receiver<ResponseResult> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id.to_string(), tx);
        rx
    }

    /// Resolve a pending request by id (called when the client replies).
    pub async fn resolve_pending(&self, id: &str) -> Option<oneshot::Sender<ResponseResult>> {
        self.pending.lock().await.remove(id)
    }
}

fn new_subdomain() -> String {
    use rand_word::rand_word;
    format!(
        "{}-{}-{}",
        rand_word(0),
        rand_word(7),
        &uuid::Uuid::new_v4().simple().to_string()[..6]
    )
}

/// Small word source for human-friendly subdomains (no extra dependency).
mod rand_word {
    const WORDS: &[&str] = &[
        "swift", "lively", "quiet", "brave", "gentle", "royal", "clever", "lucky", "golden",
        "sunny", "coral", "amber", "misty", "navy", "mint", "oak", "fern", "willow", "cypress",
    ];

    pub fn rand_word(seed: usize) -> &'static str {
        let tick = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let idx = (tick.wrapping_add(seed as u128) % WORDS.len() as u128) as usize;
        WORDS[idx]
    }
}
