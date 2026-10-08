FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install stable --profile minimal \
    --component clippy --component rustfmt --component rust-src \
    && rustup default stable

RUN cargo install cargo-watch --locked

RUN mkdir -p /usr/local/cargo/registry /usr/local/cargo/git /cargo-target \
    && chown -R vscode:vscode /usr/local/cargo /cargo-target
