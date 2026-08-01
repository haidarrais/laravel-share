//! Artisan Share tunnel client.
//!
//! A framework-agnostic local binary spawned by `php artisan share`. It reads a
//! JSON config (written by the PHP package), opens a tunnel through the active
//! driver, forwards inbound HTTP to `localhost:<local_port>`, and streams a
//! pretty-printed webhook-aware log to stdout.

mod cli;
mod drivers;
mod forward;
mod inspector;
mod log;
mod protocol;
mod redact;

use clap::Parser;

use crate::cli::Config;

#[derive(Parser, Debug)]
#[command(name = "tunnel-client", version)]
struct Args {
    /// Path to the JSON config file written by the PHP package.
    #[arg(long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let raw = std::fs::read_to_string(&args.config)
        .map_err(|e| anyhow::anyhow!("cannot read config file {}: {e}", args.config))?;
    let config: Config =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("invalid config: {e}"))?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("tunnel_client=info")),
        )
        .init();

    // The PHP package owns the subprocess and forwards SIGINT/SIGTERM, so we
    // simply run until the driver loop ends.
    drivers::run(config).await
}
