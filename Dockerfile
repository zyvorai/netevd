# Copyright 2026 Zyvor AI Labs · https://zyvor.dev
# SPDX-License-Identifier: Apache-2.0
# Multi-stage build for netevd

# Builder stage
FROM rust:1-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY benches ./benches
COPY systemd ./systemd
COPY examples ./examples

# Build release binary
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    iproute2 \
    dbus \
    && rm -rf /var/lib/apt/lists/*

# Create netevd user
RUN useradd --system --no-create-home --shell /usr/sbin/nologin netevd

# Copy binary from builder
COPY --from=builder /build/target/release/netevd /usr/bin/netevd

# Copy default configuration
COPY config/netevd.example.yaml /etc/netevd/netevd.yaml

# Create script directories
RUN mkdir -p /etc/netevd/{carrier.d,no-carrier.d,configured.d,degraded.d,routable.d,activated.d,disconnected.d,manager.d,routes.d}

# Create log directory
RUN mkdir -p /var/log/netevd && chown netevd:netevd /var/log/netevd

# Expose API port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/bin/netevd", "status"] || exit 1

# Run as root initially (will drop privileges)
USER root

# Start netevd
CMD ["/usr/bin/netevd", "start", "--foreground"]
