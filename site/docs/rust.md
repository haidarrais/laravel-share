---
title: Rust crates
layout: default
permalink: /docs/rust
---

The repository is a Cargo workspace with two crates that talk the
[relay protocol](/docs/protocol).

| Crate | Path | Purpose |
|---|---|---|
| `tunnel-client` | `crates/tunnel-client` | A single static binary spawned by `php artisan share`; opens the tunnel. |
| `relay-server` | `crates/relay-server` | A reference self-hosted backend for the `relay` driver. |

## Build

```bash
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## Lint and format

```bash
cargo clippy --workspace --all-targets
cargo fmt --all
```

Both commands are enforced in CI by the `rust` job (fmt / clippy / test) on the
`ubuntu-latest` runner.

## Cross-compilation

Release binaries for multiple platforms (Linux x86_64/aarch64, macOS
x86_64/aarch64, Windows x86_64) are built by the release workflow. The aarch64
Linux target uses `cargo-zigbuild` (with Zig) as the cross-linker on an Ubuntu
runner.
