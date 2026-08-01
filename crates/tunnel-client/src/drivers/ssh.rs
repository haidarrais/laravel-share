//! The `ssh` driver: a classic reverse tunnel (`ssh -R`) against any server the
//! user already has SSH access to.

use std::sync::Arc;

use crate::cli::Config;
use crate::log::ExchangeStore;

pub async fn run(config: Config, store: Arc<ExchangeStore>) -> anyhow::Result<()> {
    let _ = store;
    let ssh = &config.ssh;
    let local_port = config.local_port.to_string();
    let remote_port = ssh.remote_port.to_string();

    println!(
        "\x1b[1mDriver\x1b[0m       ssh ({}@{}:{remote_port})",
        ssh.user, ssh.host
    );

    // `ssh -R <remote_port>:localhost:<local_port> -N user@host` opens a reverse
    // tunnel; the public URL is <remote_port> on the remote host.
    let status = tokio::process::Command::new("ssh")
        .args([
            "-N",
            "-o",
            "ServerAliveInterval=30",
            "-o",
            "ExitOnForwardFailure=yes",
            "-R",
        ])
        .arg(format!("{remote_port}:localhost:{local_port}"))
        .arg(format!("{}@{}", ssh.user, ssh.host))
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("failed to start `ssh`: {e}"))?;

    if !status.success() {
        anyhow::bail!("ssh exited with status {status}");
    }
    Ok(())
}
