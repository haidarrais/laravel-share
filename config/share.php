<?php

declare(strict_types=1);

return [

    /*
    |--------------------------------------------------------------------------
    | Default driver
    |--------------------------------------------------------------------------
    |
    | This option controls the tunnel backend used when you run `php artisan
    | share` without an explicit --driver flag. Every driver terminates on
    | infrastructure you own (your own relay instance, your own Cloudflare
    | account, or an SSH host you have access to).
    |
    */

    'default' => env('SHARE_DRIVER', 'cloudflare'),

    /*
    |--------------------------------------------------------------------------
    | Driver presets
    |--------------------------------------------------------------------------
    |
    | Each driver has its own settings. A driver is selected by name in the
    | 'default' value above or via the --driver= flag.
    |
    */

    'drivers' => [

        /*
        | The project's own OSS relay-server, deployed by you to any host you
        | already have (Fly.io/Railway free tier, a home server, a VPS). See the
        | deploy templates in the `deploy/` directory.
        */
        'relay' => [
            // WebSocket endpoint of your relay instance, e.g. wss://tunnel.example.dev
            'endpoint' => env('SHARE_RELAY_URL'),
            // Optional per-session token required by your relay instance (if configured).
            'token' => env('SHARE_RELAY_TOKEN'),
        ],

        /*
        | Wraps your own `cloudflared` binary and Cloudflare account. Reuses
        | whatever `cloudflared login` session already exists on the machine.
        */
        'cloudflare' => [
            // Path to (or name of) the cloudflared binary.
            'binary' => env('SHARE_CLOUDFLARED_PATH', 'cloudflared'),
        ],

        /*
        | Classic SSH reverse tunnel (`ssh -R`) against any host you already
        | have SSH access to.
        */
        'ssh' => [
            'host' => env('SHARE_SSH_HOST'),
            'user' => env('SHARE_SSH_USER'),
            'remote_port' => env('SHARE_SSH_REMOTE_PORT', 8080),
        ],
    ],

    /*
    |--------------------------------------------------------------------------
    | Local server
    |--------------------------------------------------------------------------
    |
    | The local port Artisan Share forwards inbound requests to. Defaults to
    | `php artisan serve`'s 8000; override with the --port= flag.
    |
    */

    'local_port' => env('SHARE_LOCAL_PORT', 8000),

    /*
    |--------------------------------------------------------------------------
    | Local web inspector
    |--------------------------------------------------------------------------
    |
    | A localhost-only dashboard mirroring the terminal log, with request
    | replay. Port 0 disables it. Override with the --inspector-port= flag.
    |
    */

    'inspector_port' => env('SHARE_INSPECTOR_PORT', 4040),

    /*
    |--------------------------------------------------------------------------
    | Binary installation
    |--------------------------------------------------------------------------
    |
    | Where to cache the downloaded Rust tunnel client binary and from which
    | GitHub release (owner/repo) to fetch it.
    |
    */

    'binary' => [
        // Cache directory for the downloaded binary. `~/.artisan-share/bin` by default.
        'cache_dir' => env('SHARE_BINARY_CACHE_DIR', null),

        // GitHub repository providing the release assets.
        'repo' => env('SHARE_BINARY_REPO', 'haidarrais/laravel-share'),

        // Release tag/asset prefix. When null, defaults to the package version.
        'tag' => env('SHARE_BINARY_TAG', null),
    ],
];
