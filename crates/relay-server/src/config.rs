//! Relay server configuration (environment variables or CLI flags).

use clap::Parser;
use serde::Serialize;

/// Artisan Share relay server — the self-hosted backend for the `relay` driver.
#[derive(Debug, Clone, Parser, Serialize)]
#[command(name = "relay-server", version)]
pub struct Config {
    /// Listen address, e.g. `0.0.0.0:8080`.
    #[arg(long, env = "SHARE_RELAY_LISTEN", default_value = "0.0.0.0:8080")]
    pub listen_addr: String,

    /// Public hostname of this instance as clients reach it, e.g. `tunnel.example.dev`.
    #[arg(long, env = "SHARE_RELAY_HOST", default_value = "localhost")]
    pub public_host: String,

    /// Optional static token. If set, the `/tunnel` handshake requires `?token=`.
    #[arg(long, env = "SHARE_RELAY_TOKEN")]
    pub token: Option<String>,
}
