---
title: Artisan Share
layout: default
permalink: /
---

<section class="hero">
  <h1>Zero-config webhook tunnels for Laravel</h1>
  <p class="lead">
    Share a local endpoint with a public HTTPS URL via a <strong>driver you own</strong> —
    a self-hosted relay, your own Cloudflare account, or any SSH host you already have.
  </p>
  <p class="hero-note">
    <strong>This project is software only.</strong> It operates no shared relay, no backend,
    and no "artisan-share.com" service. Every public endpoint is deployed and owned by the
    person using it. There is nothing to sign up for.
  </p>
  <div class="hero-actions">
    <a class="btn primary" href="{{ '/docs/php-package' | relative_url }}">Get started</a>
    <a class="btn" href="{{ '/docs/drivers' | relative_url }}">Browse the docs</a>
  </div>
</section>

```
$ php artisan share
Artisan Share
Forwarding   https://swift-otter-42.relay.example.dev -> http://localhost:8000
Inspector    http://127.0.0.1:4040
Press Ctrl+C to stop

12:10:03  POST /webhooks/stripe   200  74ms  [stripe]  event=customer.created
12:10:11  POST /webhooks/github   200  42ms  [github]  event=push
```

## Features

- **One command, zero config.** `php artisan share` reads your app's port, starts the
  tunnel, and prints a public HTTPS URL.
- **Bring-your-own drivers.** `relay` (self-host the project's reference server),
  `cloudflare` (wraps your own `cloudflared`), and `ssh` (classic reverse tunnel). All
  terminate on infrastructure you own.
- **Webhook-aware logging.** Pretty-printed bodies, provider detection from signing
  headers, and compact one-line summaries with `--verbose` for full headers.
- **Local web inspector.** A localhost-only dashboard (`127.0.0.1:4040`) that mirrors the
  terminal log and supports request replay.
- **Secure by default.** TLS end-to-end, per-session tokens, client-side header
  redaction, and no payload persistence on any shipped driver.

## What's in this monorepo

| Artifact | Path | Description |
|---|---|---|
| Laravel package | `packages/artisan-share` | The `php artisan share` command and config. |
| Tunnel client | `crates/tunnel-client` | A single static Rust binary spawned by the command. |
| Relay server | `crates/relay-server` | A reference self-hosted backend for the `relay` driver. |

## Explore the docs

<div class="cards">
  <div class="card">
    <h3><a href="{{ '/docs/php-package' | relative_url }}">PHP package</a></h3>
    <p>Install, config, flags, and the local web inspector.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/docs/drivers' | relative_url }}">Drivers</a></h3>
    <p><code>relay</code>, <code>cloudflare</code>, and <code>ssh</code>.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/docs/deploy' | relative_url }}">Deploy</a></h3>
    <p>Run the reference relay server on Fly, Railway, or Docker.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/docs/protocol' | relative_url }}">Relay protocol</a></h3>
    <p>The wire protocol v1 spec between client and server.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/docs/rust' | relative_url }}">Rust crates</a></h3>
    <p>Build, test, and cross-compile the tunnel client and server.</p>
  </div>
  <div class="card">
    <h3><a href="{{ '/docs/contributing' | relative_url }}">Contributing</a></h3>
    <p>Development workflow, testing, and security reporting.</p>
  </div>
</div>
