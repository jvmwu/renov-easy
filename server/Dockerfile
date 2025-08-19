# RenovEasy Backend Dockerfile
# Multi-stage build for optimized Rust application

# ============================================
# Stage 1: Builder - Set up build environment
# ============================================
FROM rust:1.75-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/renoveasy

# ============================================
# Stage 2: Dependencies - Cache dependencies
# ============================================
FROM builder AS dependencies

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy crate configurations
COPY api/Cargo.toml api/Cargo.toml
COPY core/Cargo.toml core/Cargo.toml
COPY infra/Cargo.toml infra/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml

# Create dummy main files to build dependencies
RUN mkdir -p api/src core/src infra/src shared/src && \
    echo "fn main() {}" > api/src/main.rs && \
    echo "fn main() {}" > core/src/lib.rs && \
    echo "fn main() {}" > infra/src/lib.rs && \
    echo "fn main() {}" > shared/src/lib.rs

# Build dependencies in release mode
RUN cargo build --release --workspace

# Remove dummy files
RUN rm -rf api/src core/src infra/src shared/src

# ============================================
# Stage 3: Build - Compile the application
# ============================================
FROM dependencies AS build

# Copy source code
COPY api/src api/src
COPY core/src core/src
COPY infra/src infra/src
COPY shared/src shared/src

# Copy other necessary files
COPY api/i18n api/i18n
COPY migrations migrations

# Update the modification time to force rebuild
RUN touch api/src/main.rs

# Build the application in release mode
RUN cargo build --release --bin api

# ============================================
# Stage 4: Runtime - Minimal production image
# ============================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -r renoveasy && useradd -r -g renoveasy renoveasy

# Create necessary directories
RUN mkdir -p /app/migrations /app/i18n && \
    chown -R renoveasy:renoveasy /app

WORKDIR /app

# Copy the binary from builder
COPY --from=build --chown=renoveasy:renoveasy \
    /usr/src/renoveasy/target/release/api /app/renoveasy

# Copy migrations
COPY --from=build --chown=renoveasy:renoveasy \
    /usr/src/renoveasy/migrations /app/migrations

# Copy i18n files
COPY --from=build --chown=renoveasy:renoveasy \
    /usr/src/renoveasy/api/i18n /app/i18n

# ============================================
# Stage 5: Development - With development tools
# ============================================
FROM runtime AS development

# Install additional development tools
RUN apt-get update && apt-get install -y \
    vim \
    net-tools \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Keep running as root in development for flexibility
USER root

# Set development environment
ENV ENVIRONMENT=development
ENV RUST_LOG=debug

# ============================================
# Final Stage: Production
# ============================================
FROM runtime AS production

# Switch to non-root user
USER renoveasy

# Set production environment variables
ENV ENVIRONMENT=production
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose the application port
EXPOSE 8080

# Health check - adjust the endpoint if needed
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set the entrypoint
ENTRYPOINT ["/app/renoveasy"]

# ============================================
# Build Examples:
# ============================================
# Development build:
#   docker build --target development -t renoveasy-backend:dev .
#
# Production build:
#   docker build --target production -t renoveasy-backend:latest .
#
# Build with specific platform:
#   docker build --platform linux/amd64 -t renoveasy-backend:latest .
#
# ============================================
# Run Examples:
# ============================================
# Development:
#   docker run -d \
#     --name renoveasy-dev \
#     -p 8080:8080 \
#     --env-file .env.development \
#     -v $(pwd):/app \
#     renoveasy-backend:dev
#
# Production:
#   docker run -d \
#     --name renoveasy-prod \
#     -p 8080:8080 \
#     --env-file .env.production \
#     --restart unless-stopped \
#     renoveasy-backend:latest
#
# With external network:
#   docker run -d \
#     --name renoveasy-backend \
#     --network renoveasy-network \
#     -p 8080:8080 \
#     --env-file .env \
#     renoveasy-backend:latest