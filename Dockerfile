# ── Stage 1: Build static binary ──
FROM rust:1-slim AS builder
WORKDIR /fastlane
COPY crates/ crates/
COPY Cargo.toml Cargo.toml
RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN cp target/x86_64-unknown-linux-musl/release/fastlane /usr/local/bin/fastlane

# ── Stage 2: Minimal rockylinux:10 runtime ──
FROM rockylinux/rockylinux:10
RUN dnf install -y iproute ethtool && dnf clean all
WORKDIR /fastlane
COPY --from=builder /usr/local/bin/fastlane /usr/local/bin/fastlane
RUN mkdir -p /fastlane/examples
ENTRYPOINT ["fastlane"]
CMD ["--help"]
