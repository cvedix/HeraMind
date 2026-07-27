# =============================================================================
# HeraMind Dockerfile — Multi-stage build (Ubuntu 22.04 / glibc 2.35)
# =============================================================================
# Usage:
#   docker build -t heramind:latest .
#   docker compose up -d
#
# Platforms: linux/amd64, linux/arm64
#
# Why glibc (not Alpine/musl): extensions ship native binaries (`extension.so`)
# built against glibc on Ubuntu — a musl container cannot dlopen a glibc-linked
# shared library (different libc + dynamic linker), so the extension marketplace
# is unusable in an Alpine image. Ubuntu 22.04 (glibc 2.35) matches the
# bare-metal release baseline (see release-build-glibc22.04 memory), so Docker
# and bare-metal load the exact same extension binaries. Both the build and
# runtime stages use ubuntu:22.04 so the produced binary + loaded extensions
# share one glibc version (2.35).
# ============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Build frontend (static output — libc-irrelevant, alpine is fine)
# ---------------------------------------------------------------------------
FROM --platform=$BUILDPLATFORM node:20-alpine AS frontend

WORKDIR /build/web

# Install dependencies first (layer cache)
COPY web/package.json web/package-lock.json ./
RUN npm ci --ignore-scripts

# Copy source and build
COPY web/ ./
RUN npm run build

# ---------------------------------------------------------------------------
# Stage 2: Build backend (ubuntu:22.04 = glibc 2.35, matches bare-metal)
# ---------------------------------------------------------------------------
FROM --platform=$TARGETPLATFORM ubuntu:22.04 AS backend

ARG TARGETARCH
ENV DEBIAN_FRONTEND=noninteractive

# build-essential = gcc + g++ + make (make is required by tikv-jemalloc-sys's
# C build). curl+ca-certificates for rustup. pkg-config for build scripts.
# No libssl-dev: reqwest/lettre are rustls-only; the only "openssl" in the tree
# is openssl-probe (pure-Rust cert-path lookup, no link).
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        curl \
        ca-certificates \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust (pin to match rust-toolchain.toml). ubuntu:22.04 has no official
# rust:*-jammy image, so rustup is the path to a glibc-2.35 toolchain.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain 1.92.0 --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build

# Cache dependencies by creating a dummy build first
COPY Cargo.toml Cargo.lock ./
COPY crates/heramind-core/Cargo.toml crates/heramind-core/Cargo.toml
COPY crates/heramind-api/Cargo.toml crates/heramind-api/Cargo.toml
COPY crates/heramind-agent/Cargo.toml crates/heramind-agent/Cargo.toml
COPY crates/heramind-cli/Cargo.toml crates/heramind-cli/Cargo.toml
COPY crates/heramind-cli-ops/Cargo.toml crates/heramind-cli-ops/Cargo.toml
COPY crates/heramind-storage/Cargo.toml crates/heramind-storage/Cargo.toml
COPY crates/heramind-devices/Cargo.toml crates/heramind-devices/Cargo.toml
COPY crates/heramind-rules/Cargo.toml crates/heramind-rules/Cargo.toml
COPY crates/heramind-messages/Cargo.toml crates/heramind-messages/Cargo.toml
COPY crates/heramind-extension-sdk/Cargo.toml crates/heramind-extension-sdk/Cargo.toml
COPY crates/heramind-extension-runner/Cargo.toml crates/heramind-extension-runner/Cargo.toml
COPY crates/heramind-data-push/Cargo.toml crates/heramind-data-push/Cargo.toml

# Create dummy source files for dependency caching
RUN mkdir -p crates/heramind-core/src && echo "" > crates/heramind-core/src/lib.rs && \
    mkdir -p crates/heramind-api/src && echo "fn main(){}" > crates/heramind-api/src/lib.rs && \
    mkdir -p crates/heramind-agent/src && echo "" > crates/heramind-agent/src/lib.rs && \
    mkdir -p crates/heramind-cli/src && echo "fn main(){}" > crates/heramind-cli/src/main.rs && \
    mkdir -p crates/heramind-cli-ops/src && echo "" > crates/heramind-cli-ops/src/lib.rs && \
    mkdir -p crates/heramind-storage/src && echo "" > crates/heramind-storage/src/lib.rs && \
    mkdir -p crates/heramind-devices/src && echo "" > crates/heramind-devices/src/lib.rs && \
    mkdir -p crates/heramind-rules/src && echo "" > crates/heramind-rules/src/lib.rs && \
    mkdir -p crates/heramind-messages/src && echo "" > crates/heramind-messages/src/lib.rs && \
    mkdir -p crates/heramind-extension-sdk/src && echo "" > crates/heramind-extension-sdk/src/lib.rs && \
    mkdir -p crates/heramind-extension-runner/src && echo "" > crates/heramind-extension-runner/src/lib.rs && \
    mkdir -p crates/heramind-data-push/src && echo "" > crates/heramind-data-push/src/lib.rs

# jemalloc (heramind-cli global allocator) must assume 64KB pages on ARM, else it
# crashes on 64KB-page hosts like Raspberry Pi 5 / Jetson (the arm64 container
# runs on the host kernel, so a 64KB-page Pi5 host still sees 64KB pages inside
# the container). No-op on amd64 (4KB pages). See release-build-glibc22.04.
RUN if [ "$TARGETARCH" = "arm64" ] || [ "$TARGETARCH" = "aarch64" ]; then export JEMALLOC_SYS_WITH_LG_PAGE=16; fi && \
    cargo build --release -p heramind-cli -p heramind-extension-runner 2>/dev/null || true

# Copy real source code and build
COPY crates/ crates/
RUN if [ "$TARGETARCH" = "arm64" ] || [ "$TARGETARCH" = "aarch64" ]; then export JEMALLOC_SYS_WITH_LG_PAGE=16; fi && \
    cargo build --release -p heramind-cli -p heramind-extension-runner

# ---------------------------------------------------------------------------
# Stage 3: Runtime (ubuntu:22.04 = glibc 2.35, same as build)
# ---------------------------------------------------------------------------
FROM ubuntu:22.04 AS runtime

ENV DEBIAN_FRONTEND=noninteractive

# apt-get upgrade patches base-image packages between refreshes (the main
# source of "high" findings in image scans). Then add runtime deps.
RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y --no-install-recommends ca-certificates curl tzdata && \
    rm -rf /var/lib/apt/lists/* && \
    groupadd --system heramind && useradd --system --gid heramind --home-dir /app heramind

WORKDIR /app

# Copy backend binaries (heramind finds extension-runner in same directory or PATH)
COPY --from=backend /build/target/release/heramind /usr/local/bin/heramind
COPY --from=backend /build/target/release/heramind-extension-runner /usr/local/bin/heramind-extension-runner

# Copy frontend build output
COPY --from=frontend /build/web/dist /var/www/heramind

# Create data directory
RUN mkdir -p /app/data && chown -R heramind:heramind /app/data

# Environment defaults
ENV HERAMIND_WEB_DIR=/var/www/heramind
ENV RUST_LOG=heramind=info
ENV RUST_BACKTRACE=1

EXPOSE 9375 1883

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:9375/api/health || exit 1

USER heramind

VOLUME ["/app/data"]

ENTRYPOINT ["heramind"]
CMD ["serve"]
