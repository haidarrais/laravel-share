---
title: Deploying the relay server
layout: default
permalink: /docs/deploy
---

The reference `relay-server` is a self-hosted backend for the `relay` driver.
Deploy it on infrastructure you control using the templates in the repository
[`deploy/`](https://github.com/haidarrais/laravel-share/tree/main/deploy). The
server binds `0.0.0.0:8080` and keeps only ephemeral in-memory state — no
database or volumes are required.

## Docker Compose

`deploy/docker/compose.yml` runs the server image with the bundled
`deploy/docker/relay-server.Dockerfile`:

```bash
docker compose -f deploy/docker/compose.yml up -d
```

## Fly.io

`deploy/fly.toml` is a drop-in Fly.io app config that uses the same Dockerfile:

```bash
flyctl launch --config deploy/fly.toml --dockerfile deploy/docker/relay-server.Dockerfile
flyctl secrets set SHARE_RELAY_HOST=tunnel.your-app.fly.dev SHARE_RELAY_TOKEN=change-me
flyctl deploy --config deploy/fly.toml
```

## Railway

`deploy/railway.Dockerfile` is a multi-stage Dockerfile you point a Railway
service at directly, with the repository root as the build context.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `SHARE_RELAY_HOST` | `localhost` | Public hostname clients reach the relay at (used to derive tunnel URLs). |
| `SHARE_RELAY_TOKEN` | *(empty)* | Optional static token clients must present on connect. |

Because clients connect over `wss://`, terminate TLS at a reverse proxy (Caddy,
Nginx, or your platform's load balancer) in front of port 8080.
