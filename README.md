# Artisan Share

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

---

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

## Requirements

- PHP **8.2+** and Composer
- Laravel **10, 11, or 12**
- A tunnel driver you already control (see [Drivers](#drivers))

## Installation

> **Note on Packagist.** This monorepo publishes the umbrella
> `artisan-share/monorepo` package on Packagist, but public Packagist does not
> expose packages living in a monorepo subdirectory. So `artisan-share/laravel`
> is not a standalone Packagist package — install it straight from this GitHub
> repository as a VCS repository.

```bash
composer config repositories.artisan-share vcs https://github.com/haidarrais/laravel-share.git
composer require --dev artisan-share/laravel:dev-main
```

The package registers its service provider automatically. On first `php artisan
share`, the platform-appropriate Rust binary is downloaded from the GitHub
release, its SHA-256 checksum is verified, and it is cached under
`~/.artisan-share/bin`.

To publish the config:

```bash
php artisan vendor:publish --tag=share-config
```

## Usage

```bash
php artisan share                              # uses the configured default driver
php artisan share --driver=relay               # override the driver for this run
php artisan share --port=9000                  # forward a different local port
php artisan share --subdomain=my-app           # request a subdomain (relay)
php artisan share --basic-auth="user:pass"     # protect the public endpoint
php artisan share --inspector-port=0           # disable the web inspector
php artisan share --verbose                    # show full request headers
```

Press `Ctrl+C` to shut down cleanly. The tunnel session is torn down so the
public URL is immediately invalidated.

### Flags

| Flag | Description |
|---|---|
| `--driver` | Tunnel backend: `relay`, `cloudflare`, or `ssh`. |
| `--port` | Local port to forward to (default: your `local_port` config). |
| `--subdomain` | Requested subdomain on the relay driver. |
| `--basic-auth` | `user:pass` HTTP basic auth for the public endpoint. |
| `--inspector-port` | Port for the local web inspector (`0` disables). |
| `--verbose` | Print full request headers in the terminal log. |
| `--binary` | Path to an already-installed tunnel client binary. |

## Drivers

Every driver terminates on infrastructure you already own. Pick the default in
`config/share.php` (or `SHARE_DRIVER`).

| Driver | Backend | Account | Cost |
|---|---|---|---|
| `relay` | The project's reference `relay-server`, which you deploy yourself. | Whatever host you already use. | Free tier of your host. |
| `cloudflare` | Wraps your own `cloudflared` binary and Cloudflare account. | Your Cloudflare account. | Free. |
| `ssh` | Classic `ssh -R` reverse tunnel. | Any SSH host you can access. | Free if you already have one. |

### relay

```php
// config/share.php
'drivers' => [
    'relay' => [
        'endpoint' => env('SHARE_RELAY_URL'),    // e.g. wss://tunnel.example.dev
        'token'    => env('SHARE_RELAY_TOKEN'),  // your instance's token, if set
    ],
],
```

Deploy the reference server anywhere with the templates in [`deploy/`](deploy/):

- [Fly.io](deploy/fly.toml)
- [Railway](deploy/railway.Dockerfile)
- [Docker Compose](deploy/docker/compose.yml)

### cloudflare

```php
'drivers' => [
    'cloudflare' => [
        'binary' => env('SHARE_CLOUDFLARED_PATH', 'cloudflared'),
    ],
],
```

`cloudflared` must already be installed and logged in (`cloudflared login`).
Artisan Share only shells out to the session you already established.

### ssh

```php
'drivers' => [
    'ssh' => [
        'host'        => env('SHARE_SSH_HOST'),
        'user'        => env('SHARE_SSH_USER'),
        'remote_port' => env('SHARE_SSH_REMOTE_PORT', 8080),
    ],
],
```

Opens `ssh -R <remote_port>:localhost:<port>` to your host.

## Local Web Inspector

By default a localhost-only dashboard runs at **http://127.0.0.1:4040**. It
mirrors the terminal log and lets you:

- list captured requests,
- inspect full headers and bodies,
- **replay** a captured webhook against your local app without re-triggering it
  from the provider.

Set `--inspector-port=0` or `inspector_port` to disable it.

## Webhook provider detection

The logger labels requests based on common signing headers (without validating
payloads):

- `Stripe-Signature` → `stripe`
- `X-Hub-Signature-256` → `github`
- `X-Slack-Signature` → `slack`
- and more

## Redaction

Sensitive request headers (`authorization`, `cookie`, `stripe-signature`, …)
and secret-shaped body patterns are masked in the terminal log and inspector by
default. Use `--verbose` to reveal headers.

## Security

Opening a public ingress point to your machine is powerful. Read the full
[`SECURITY.md`](SECURITY.md) for the threat model and reporting policy.

## Development

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

### Rust

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --all
```

### PHP

```bash
cd packages/artisan-share
composer install
vendor/bin/phpunit
vendor/bin/pint --test
```

## Versioning & compatibility

All three published artifacts follow [Semantic Versioning](https://semver.org).
The Rust client and relay server are compatible with each other at the wire
protocol level; the PHP package and the Rust client share a JSON config
contract. See the compatibility matrix below.

| artifact version | protocol | config contract |
|---|---|---|
| v0.1 | `1` | `1` |

## License

[MIT](LICENSE)
