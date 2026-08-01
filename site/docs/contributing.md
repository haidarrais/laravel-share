---
title: Contributing
layout: default
permalink: /docs/contributing
---

Thanks for considering contributing to Artisan Share.

## Code of conduct

All interactions are governed by the project's
[Code of Conduct](https://github.com/haidarrais/laravel-share/blob/main/CODE_OF_CONDUCT.md).

## Development

See the full
[CONTRIBUTING.md](https://github.com/haidarrais/laravel-share/blob/main/CONTRIBUTING.md)
for setup and workflow details.

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

## Security

Opening a public ingress point to your machine is powerful. Before reporting an
issue, read the threat model and reporting policy in
[SECURITY.md](https://github.com/haidarrais/laravel-share/blob/main/SECURITY.md).
