---
title: Artisan Share
layout: default
permalink: /
---

Zero-config webhook tunnels for Laravel. Share a local endpoint with a public
URL via a **driver you own** — a self-hosted relay, your own Cloudflare
account, or any SSH host you already have.

> **This project is software only.** It operates no shared relay, no backend,
> and no "artisan-share.com" service. Every public endpoint is deployed and
> owned by the person using it. There is nothing to sign up for.

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

- **One command, zero config.** `php artisan share` reads your app's port,
  starts the tunnel, and prints a public HTTPS URL.
- **Bring-your-own drivers.** `relay` (self-host the project's reference
  server), `cloudflare` (wraps your own `cloudflared`), and `ssh` (classic
  reverse tunnel). All terminate on infrastructure you own.
- **Webhook-aware logging.** Pretty-printed bodies, provider detection from
  signing headers, and compact one-line summaries with `--verbose` for full
  headers.
- **Local web inspector.** A localhost-only dashboard (`127.0.0.1:4040`) that
  mirrors the terminal log and supports request replay.
- **Secure by default.** TLS end-to-end, per-session tokens, client-side header
  redaction, and no payload persistence on any shipped driver.

## Contents

This monorepo ships three artifacts:

| Artifact | Path | Description |
|---|---|---|
| Laravel package | `packages/artisan-share` | The `php artisan share` command and config. |
| Tunnel client | `crates/tunnel-client` | A single static Rust binary spawned by the command. |
| Relay server | `crates/relay-server` | A reference self-hosted backend for the `relay` driver. |

## Getting started

```bash
composer require --dev artisan-share/laravel
php artisan share
```

See the [PHP package guide](/docs/php-package) for installation, config, and
the full flag reference.

## Explore the docs

- [Drivers](/docs/drivers) — `relay`, `cloudflare`, and `ssh`.
- [Deploy](/docs/deploy) — deploy the reference relay server (Fly, Railway,
  Docker Compose).
- [Relay protocol](/docs/protocol) — the wire protocol v1 spec.
- [Rust](/docs/rust) — the tunnel client and relay server crates.
- [Contributing](/docs/contributing) — build, test, and report security issues.

The full source of this page is the repository
[README]({{ site.github.repository_url }}/blob/main/README.md).
