# syntax=docker/dockerfile:1

# Multi-stage image: compilation happens in `builder`, the final image only
# ships the binary and the minimal system libraries.
#
# Build context: repository root (see docker-compose.yml).
#   docker build -t oxidax:latest -f docker/prod/Dockerfile .

# ---------------------------------------------------------------- builder
FROM rust:1-slim AS builder
WORKDIR /app

# Cargo lock: reproducible build (same versions on every build).
COPY Cargo.toml Cargo.lock ./
# Local workspace crates: oxidax-core and oxidax-macro are `path`
# dependencies, oxidax-cli is a workspace member (needed to resolve the
# workspace).
COPY oxidax-core ./oxidax-core
COPY oxidax-macro ./oxidax-macro
COPY oxidax-cli ./oxidax-cli

# Dummy build: compiles and caches ALL the dependencies. Rebuilding the real
# src reuses the libraries, so the actual build is fast.
RUN mkdir src && echo 'fn main(){}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Real build of the API binary.
COPY src ./src
RUN cargo build --release

# ---------------------------------------------------------------- runtime
FROM debian:bookworm-slim AS runtime

# ca-certificates: needed if SURREAL_URL uses wss:// (TLS).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/oxidax /usr/local/bin/oxidax

# Non-root user with dedicated uid/gid.
RUN useradd --uid 10001 --create-home app
USER app

# Almost every value comes from the environment (see docker/prod/.env.example).
# The only default here is listening on all interfaces, which is always correct
# inside a container.
ENV SERVER_ADDR=0.0.0.0:3000

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/oxidax"]
