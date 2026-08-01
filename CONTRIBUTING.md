# Contributing to Artisan Share

Thanks for your interest! This project is deliberately lean and well-scoped.
Please read this before opening a PR so the maintainer's review is easy and your
contribution merges quickly.

## Code of Conduct

By participating you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to contribute

1. **Open an issue** (or comment on an existing one) describing what you want to
   change and why, before writing code. This avoids wasted work on something the
   maintainer would scope differently.
2. **Fork** the repo and create a branch off `main`:
   `git checkout -b feat/your-change`.
3. Make a **small, focused** change that matches the surrounding style.
4. **Verify** (see below).
5. Open a **PR** with a clear title and a description that states the problem,
   the change, and how you verified it.

## Development environment

- **Rust:** stable toolchain. Verify with `rustc --version`.
- **PHP:** 8.2+ with Composer.
- **Docker** (optional) to build/test the relay image.

## The three artifacts

| Artifact | Path | Language | Checks |
|---|---|---|---|
| Laravel package | `packages/artisan-share` | PHP | PHPUnit, Pint |
| Tunnel client | `crates/tunnel-client` | Rust | cargo test, clippy, fmt |
| Relay server | `crates/relay-server` | Rust | cargo test, clippy, fmt |

A PR that touches more than one artifact is expected to pass the checks for
every artifact it touches.

## Verification

### Rust

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### PHP

```bash
cd packages/artisan-share
composer install
vendor/bin/pint --test
vendor/bin/phpunit
```

### Docker image (if you change `deploy/`)

```bash
docker build -f deploy/docker/relay-server.Dockerfile -t relay-server:test .
```

### End-to-end smoke test

The integration test in `crates/relay-server/tests/forwarding.rs` spins up the
relay, a tunnel client, and a local origin in-process and verifies the full
round trip. Run it with:

```bash
cargo test -p relay-server --test forwarding
```

## Conventions

- **Match the neighbors.** Imitate the style, naming, and error handling of the
  code you're editing. Do not introduce new patterns for the sake of it.
- **Smallest correct diff.** No drive-by refactors, renames, or formatting
  changes unrelated to your change.
- **Redaction stays client-side and driver-agnostic.** Sensitive data masking
  must never depend on a central (trusted) relay.
- **No new shared endpoint.** Never add a feature that routes traffic through
  infrastructure this project operates.
- **Binary/lockfile changes.** Because both crates are binaries, `Cargo.lock` is
  committed. Update it when you change dependencies (`cargo build` regenerates
  it).

## Good first issues

Look for issues labeled [`good first issue`](https://github.com/haidarrais/laravel-share/labels/good%20first%20issue).
These are scoped, low-risk, and perfect for a first contribution.

## Docs

Update `README.md` and any affected `deploy/` templates when behavior changes
user-visible behavior (flags, config keys, env vars, protocol).

## Commit messages

We use conventional, imperative commit messages:

```
feat: add X
fix: correct Y
docs: explain Z
test: cover W
```
