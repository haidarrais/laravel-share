//! Artisan Share reference relay server (library facade).
//!
//! The crate is structured as a library plus a thin `main` binary so that
//! integration tests can drive the full axum application in-process. See
//! [`crate::main`] for the binary entrypoint.

pub mod config;
pub mod protocol;
pub mod session;
pub mod tunnel;

pub use config::Config;
pub use session::{RequestToClient, ResponseResult, Session, SessionStore};
pub use tunnel::{http_app, ws_handler, AppState};

use std::net::SocketAddr;

use axum::routing::get;
use axum::Router;
use tracing::info;

/// Builds the axum router for a given application state.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/tunnel", get(ws_handler))
        .fallback(get(http_app))
        .with_state(state)
}

/// Serves the application; used by both the binary and integration tests.
pub async fn serve(state: AppState, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "listening");
    axum::serve(listener, app(state)).await?;
    Ok(())
}
