# Multi-stage build for the Artisan Share reference relay server.
# Produces a small distroless image containing only the relay-server binary.
#
# Build locally with:
#   docker build -f deploy/docker/relay-server.Dockerfile -t relay-server .

# ---- build stage -----------------------------------------------------------
FROM rust:1.85 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p relay-server

# ---- runtime stage ---------------------------------------------------------
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/relay-server /usr/local/bin/relay-server
# The relay keeps only ephemeral in-memory state and writes nothing to disk.
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/relay-server"]
