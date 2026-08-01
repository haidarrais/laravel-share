//! Client configuration, deserialized from the JSON file written by the PHP
//! package. This keeps the Rust binary framework-agnostic: any host language
//! can emit this shape.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Which driver to use: `relay`, `cloudflare`, or `ssh`.
    pub driver: String,
    /// Local port to forward inbound requests to.
    pub local_port: u16,
    /// Optional reserved subdomain (relay driver).
    pub subdomain: Option<String>,
    /// Optional `user:pass` HTTP basic auth applied to the public endpoint.
    pub basic_auth: Option<String>,
    /// Show full headers in the terminal log.
    pub verbose: bool,
    /// Port for the localhost web inspector; `0` disables it.
    pub inspector_port: u16,
    pub relay: RelayConfig,
    pub cloudflare: CloudflareConfig,
    pub ssh: SshConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelayConfig {
    /// WebSocket endpoint, e.g. `wss://tunnel.example.dev`.
    pub endpoint: String,
    /// Optional per-session token required by the operator's relay instance.
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CloudflareConfig {
    /// Path to (or name of) the `cloudflared` binary.
    pub binary: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SshConfig {
    pub host: String,
    pub user: String,
    pub remote_port: u16,
}
