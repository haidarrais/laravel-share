---
title: Relay wire protocol
layout: default
permalink: /docs/protocol
---

The `relay` driver tunnels HTTP requests over a single WebSocket connection
using newline-delimited JSON messages. Both `crates/relay-server` and
`crates/tunnel-client` implement this protocol; the JSON config contract between
the Laravel package and the client is versioned separately (see the README
compatibility matrix).

The canonical spec lives in
[`docs/protocol.md`](https://github.com/haidarrais/laravel-share/blob/main/docs/protocol.md).
This page summarizes the v1 contract.

## Handshake

The client connects to `{endpoint}/tunnel` over `wss://`. Optional query
parameters:

- `token` — required when the operator's relay instance is configured with a
  static token.
- `subdomain` — a requested subdomain, sent only on the first connection attempt
  of a session.

On connect the server immediately sends a `hello`:

```json
{"type":"hello","url":"https://swift-otter-42.relay.example.dev","session_id":"..."}
```

The client derives its subdomain from the `url`'s host for any subsequent
reconnect in the same session.

## Messages

All messages are JSON with a `type` discriminator. Fields are `snake_case`.
Single JSON object per newline.

### Server → client

**`hello`** (sent once at session start)

```json
{"type":"hello","url":"...","session_id":"..."}
```

**`request`** (an inbound HTTP request to forward to localhost)

```json
{
  "type": "request",
  "id": "9f1c...",
  "method": "POST",
  "path": "/webhooks/stripe",
  "query": "?event=customer.created",
  "headers": {"content-type": ["application/json"], ...},
  "body": "<base64>"
}
```

`body` is the raw request body base64-encoded. The `id` must be echoed back in
the matching `response`.

**`error`** (the server rejected or failed a request)

```json
{"type":"error","id":"...","message":"..."}
```

### Client → server

**`response`** (the result of forwarding `request.id` to localhost)

```json
{"type":"response","id":"...","status":200,"headers":{...},"body":"<base64>"}
```

**`close`** (graceful teardown; the server tears down the session and
invalidates the public URL)

```json
{"type":"close"}
```

## Routing

The server maps inbound HTTP requests to a tunnel by the first label of the
`Host` header against the session's subdomain. A request for a host with no
active tunnel returns `404`.

## Limits

- Maximum message size: **16 MiB**.
- A dropped client connection while a request is in flight yields `502` to the
  public caller; a timeout awaiting a response yields `504`.

## Versioning

The `type` and field names above are the v1 contract. A breaking change to this
protocol increments the version and is coordinated across the client, server,
and config compatibility matrix in the README.
