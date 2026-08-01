//! Artisan Share reference relay server.
//!
//! This is the project's *only* server-side component, and it is never run by
//! the project on anyone's behalf. It ships purely as software for users to
//! deploy under their own account. It keeps only ephemeral in-memory session
//! state by default; nothing is written to disk unless the operator opts in.
//!
//! The implementation lives in the library (`lib.rs`); this binary is a thin
//! wrapper that parses configuration and starts the axum server.

use std::net::SocketAddr;

use clap::Parser;
use relay_server::Config;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("relay_server=info")),
        )
        .init();

    let addr: SocketAddr = cfg.listen_addr.parse()?;
    info!(host = %cfg.public_host, %addr, "relay server starting");

    let state = relay_server::AppState {
        store: std::sync::Arc::new(relay_server::SessionStore::new()),
        config: std::sync::Arc::new(cfg),
    };
    relay_server::serve(state, addr).await
}
