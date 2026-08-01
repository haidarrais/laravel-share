//! Pluggable relay drivers.
//!
//! Every driver terminates on infrastructure the *user* owns; the client never
//! depends on a project-operated endpoint. The `relay` driver speaks the native
//! WSS protocol; `cloudflare` and `ssh` wrap the user's own binaries/sessions.

pub mod cloudflare;
pub mod relay;
pub mod ssh;

use std::sync::Arc;

use crate::cli::Config;
use crate::log::ExchangeStore;

/// Run the configured driver to completion.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let store = Arc::new(ExchangeStore::new(config.verbose));

    // Print the standard header, matching the UX regardless of driver.
    println!("\x1b[1mArtisan Share\x1b[0m");

    // Start the inspector unless disabled.
    if config.inspector_port != 0 {
        let store_insp = store.clone();
        let local_port = config.local_port;
        let basic_auth = config.basic_auth.clone();
        let port = config.inspector_port;
        tokio::spawn(async move {
            if let Err(e) = crate::inspector::spawn(store_insp, local_port, basic_auth, port).await
            {
                eprintln!("inspector failed to start: {e}");
            }
        });
    }

    match config.driver.as_str() {
        "relay" => relay::run(config, store).await,
        "cloudflare" => cloudflare::run(config, store).await,
        "ssh" => ssh::run(config, store).await,
        other => anyhow::bail!("unknown driver: {other}"),
    }
}
