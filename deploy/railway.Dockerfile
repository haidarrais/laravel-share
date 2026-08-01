# Deploy your own relay instance on Railway.
#
#   1. Push this repository to GitHub.
#   2. In Railway, create a new project from that repo.
#   3. Railway auto-detects this Dockerfile.
#   4. Add the service env vars:
#        SHARE_RELAY_HOST = <your-app>.up.railway.app
#        SHARE_RELAY_TOKEN = <a long random token>
#   5. Expose the default service port (8080).

# Multi-stage build for the reference relay server.
FROM rust:1.85 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p relay-server

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/relay-server /usr/local/bin/relay-server
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/relay-server"]
