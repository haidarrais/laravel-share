//! The `cloudflare` driver: wraps the user's own `cloudflared` binary and
//! Cloudflare account. Artisan Share only shells out to a session the user
//! already logged into; it never holds a Cloudflare credential.

use std::sync::Arc;

use crate::cli::Config;
use crate::log::ExchangeStore;

pub async fn run(config: Config, store: Arc<ExchangeStore>) -> anyhow::Result<()> {
    let _ = store;
    let binary = &config.cloudflare.binary;
    let port = config.local_port.to_string();

    println!("\x1b[1mDriver\x1b[0m       cloudflare (via `{binary}`)");

    // `cloudflared tunnel --url <local>` opens a quick tunnel using the user's
    // existing cloudflared session and prints a public trycloudflare hostname.
    // The assigned hostname is random; a fixed subdomain requires a named
    // tunnel configured on the user's own Cloudflare account.
    let status = tokio::process::Command::new(binary)
        .args(["tunnel", "--no-autoupdate", "--url"])
        .arg(format!("http://localhost:{port}"))
        .status()
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "failed to start `{binary}`: {e}\nEnsure cloudflared is installed and logged in via `cloudflared tunnel login`."
            )
        })?;

    if !status.success() {
        anyhow::bail!("`{binary}` exited with status {status}");
    }
    Ok(())
}
