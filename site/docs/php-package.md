---
title: PHP package
layout: default
permalink: /docs/php-package
---

The Laravel package at `packages/artisan-share` provides the `php artisan
share` command, its config, and the platform-appropriate tunnel-client binary
management.

## Requirements

- PHP **8.2+** and Composer
- Laravel **10, 11, or 12**
- A tunnel driver you already control (see [Drivers](/docs/drivers))

## Installation

```bash
composer require --dev artisan-share/laravel
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

## Local web inspector

By default a localhost-only dashboard runs at **http://127.0.0.1:4040**. It
mirrors the terminal log and lets you list captured requests, inspect full
headers and bodies, and **replay** a captured webhook against your local app.
Set `--inspector-port=0` or `inspector_port` to disable it.
