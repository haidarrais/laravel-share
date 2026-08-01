---
title: Drivers
layout: default
permalink: /docs/drivers
---

Every driver terminates on infrastructure you already own. Pick the default in
`config/share.php` (or `SHARE_DRIVER`).

| Driver | Backend | Account | Cost |
|---|---|---|---|
| `relay` | The project's reference `relay-server`, which you deploy yourself. | Whatever host you already use. | Free tier of your host. |
| `cloudflare` | Wraps your own `cloudflared` binary and Cloudflare account. | Your Cloudflare account. | Free. |
| `ssh` | Classic `ssh -R` reverse tunnel. | Any SSH host you can access. | Free if you already have one. |

## relay

```php
// config/share.php
'drivers' => [
    'relay' => [
        'endpoint' => env('SHARE_RELAY_URL'),    // e.g. wss://tunnel.example.dev
        'token'    => env('SHARE_RELAY_TOKEN'),  // your instance's token, if set
    ],
],
```

Deploy the reference server anywhere with the templates in [`deploy/`](../deploy):
Fly.io, Railway, or Docker Compose.

## cloudflare

```php
'drivers' => [
    'cloudflare' => [
        'binary' => env('SHARE_CLOUDFLARED_PATH', 'cloudflared'),
    ],
],
```

`cloudflared` must already be installed and logged in (`cloudflared login`).
Artisan Share only shells out to the session you already established.

## ssh

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
