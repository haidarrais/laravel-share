## What changed

Briefly describe the change and why. Link the issue it resolves (e.g. `Closes #123`).

## Scope
- [ ] Laravel package (`src/`, root `composer.json`)
- [ ] Tunnel client (`crates/tunnel-client`)
- [ ] Relay server (`crates/relay-server`)
- [ ] Deploy templates (`deploy/`)
- [ ] Docs / CI

## Verification

Paste the real output for every check that applies. Do not summarize as "passes".

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `vendor/bin/pint --test` (if PHP changed)
- [ ] `vendor/bin/phpunit` (if PHP changed)
- [ ] Docker build (if `deploy/` changed)

## Notes for the reviewer
Anything the reviewer should double-check, assumptions made, or follow-ups.

## Checklist
- [ ] Smallest correct diff (no unrelated changes)
- [ ] Matches surrounding style and conventions
- [ ] No secrets, credentials, or environment-specific values added
- [ ] No new shared endpoint / project-operated service introduced
