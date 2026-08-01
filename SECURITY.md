# Security Policy

Artisan Share intentionally opens a public ingress point to a developer's
machine. We take security seriously and expect the same from the community.

## Reporting a vulnerability

**Please do not open a public issue for security reports.** Report
vulnerabilities privately:

- **GitHub:** open a [private security advisory][gh-advisory] on this
  repository, **or**
- **Email:** `security@artisan-share.dev` (PGP key available on request)

We ask you to include:

- the affected artifact and version,
- a minimal, reproducible description,
- the impact you believe the issue has,
- a suggested fix, if you have one.

We acknowledge reports within **72 hours** and work toward a coordinated
disclosure. Once a fix is released we credit reporters (unless they ask to
remain anonymous).

## Scope

In scope:

- The `tunnel-client` binary (forwarding, redaction, inspector, driver logic).
- The `relay-server` reference implementation.
- The Laravel package (the repository root `composer.json`).

Out of scope (the project operates none of these; they are owned by the user):

- Infrastructure the user deploys for the `relay`/`cloudflare`/`ssh` drivers.
- Third-party providers (Cloudflare, GitHub, the user's SSH host).
- Laravel itself or the Rust toolchain.

## Threat model

The core risk: a public URL points into a developer's local machine. Anyone who
can reach that URL can send HTTP requests to the developer's local server.

Mitigations shipped by default:

- **TLS end-to-end** between the webhook provider, the public endpoint, and the
  client.
- **Per-session tokens** on the `relay` driver; single-use, expire on process
  exit.
- **Client-side header redaction.** Sensitive headers (`authorization`,
  `cookie`, `stripe-signature`, …) and secret-shaped body patterns are masked in
  logs and the inspector by default.
- **No payload persistence.** No shipped driver writes request bodies to disk or
  a database by default.
- **Inspector is loopback-only.** The web dashboard binds `127.0.0.1` and is not
  exposed through the tunnel.
- **Optional basic auth** on the public endpoint via `--basic-auth`.

### Assumptions and residual risk

- The `relay` driver is a *reference* implementation; operators deploying it are
  responsible for securing their own instance (TLS termination, token
  provisioning, rate limiting, network isolation).
- The `cloudflare` and `ssh` drivers inherit the security posture of the user's
  own account/host.
- The terminal logger deliberately **does not validate** webhook signatures; it
  only labels requests. Do not treat the log as proof of authenticity.
- A `--basic-auth` credential protects the public endpoint only to the degree
  your chosen driver honors it.

## Supported versions

| Version | Supported |
|---|---|
| latest release | ✅ |
| older releases | ❌ |

We patch the current release and, where feasible, backport security fixes one
minor version back.

## Disclosure timeline

We follow a **90-day** coordinated disclosure window, shortened only by prior
public exploitation or mutual agreement.

[gh-advisory]: https://github.com/haidarrais/laravel-share/security/advisories/new
