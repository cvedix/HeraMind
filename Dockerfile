# =============================================================================
# HeraMind Dockerfile — Multi-stage build
# =============================================================================
# Usage:
#   docker build -t heramind:latest .
#   docker compose up -d
#
# Platforms: linux/amd64, linux/arm64
# =============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Build frontend
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
# Stage 2: Build backend
# ---------------------------------------------------------------------------
FROM --platform=$TARGETPLATFORM rust:1.85-alpine AS backend

RUN apk add --no-cache musl-dev

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

RUN cargo build --release -p heramind-cli -p heramind-extension-runner 2>/dev/null || true

# Copy real source code and build
COPY crates/ crates/
RUN cargo build --release -p heramind-cli -p heramind-extension-runner

# ---------------------------------------------------------------------------
# Stage 3: Runtime
# ---------------------------------------------------------------------------
FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates curl tzdata && \
    addgroup -S heramind && adduser -S heramind -G heramind

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
