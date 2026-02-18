# =============================================================================
# Fax Compiler - Production Docker Image
# =============================================================================
# Multi-stage build for minimal runtime image with security best practices
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# -----------------------------------------------------------------------------
# Uses Rust 1.75 (MSRV - Minimum Supported Rust Version) as specified in Cargo.toml
# QC-010 FIX: Updated from rust:1.75-slim to rust:1.75-bookworm for security updates
FROM rust:1.75-bookworm AS builder

# Labels for builder stage
LABEL stage="builder" \
      maintainer="Fax Team <fax-lang@example.com>" \
      description="Fax Compiler Builder Stage"

# Set environment variables for optimized builds
ENV CARGO_TERM_COLOR=always \
    CARGO_INCREMENTAL=0 \
    CARGO_NET_RETRY=10 \
    RUSTFLAGS="-C link-arg=-s" \
    RUST_BACKTRACE=1

# Install build dependencies
# - pkg-config: Required for native dependency resolution
# - libssl-dev: OpenSSL development headers for any SSL-related crates
# - clang, llvm, lld: Required for code generation (as per CI workflow)
# - ca-certificates: For downloading crates from crates.io
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    clang \
    llvm \
    lld \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Set working directory
WORKDIR /app

# Copy workspace Cargo configuration first for layer caching
# This allows Docker to cache dependency builds when only source code changes
COPY faxc/Cargo.toml faxc/Cargo.lock ./

# Copy individual crate Cargo.toml files for proper workspace resolution
# This ensures Docker layer caching works correctly for dependency changes
COPY faxc/crates/faxc-util/Cargo.toml ./crates/faxc-util/
COPY faxc/crates/faxc-lex/Cargo.toml ./crates/faxc-lex/
COPY faxc/crates/faxc-par/Cargo.toml ./crates/faxc-par/
COPY faxc/crates/faxc-sem/Cargo.toml ./crates/faxc-sem/
COPY faxc/crates/fgc/Cargo.toml ./crates/fgc/
COPY faxc/crates/faxc-mir/Cargo.toml ./crates/faxc-mir/
COPY faxc/crates/faxc-lir/Cargo.toml ./crates/faxc-lir/
COPY faxc/crates/faxc-gen/Cargo.toml ./crates/faxc-gen/
COPY faxc/crates/faxc-drv/Cargo.toml ./crates/faxc-drv/

# Build dependencies only (creates empty binary, caches dependency compilation)
# Using --message-format=short for cleaner output
RUN cargo build --workspace --release --bin faxc \
    && rm -rf target/release/deps/faxc* \
    && rm -rf target/release/.fingerprint \
    && rm -rf target/release/build

# Copy source code (invalidates cache only when source changes)
COPY faxc/crates ./crates

# Build the actual application in release mode
# Using LTO and optimizations as defined in Cargo.toml profile.release
RUN cargo build --workspace --release --bin faxc \
    && strip target/release/faxc

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
# Minimal Debian Bookworm slim image for production runtime
FROM debian:bookworm-slim AS runtime

# Labels for runtime stage (OCI-compliant labels)
LABEL org.opencontainers.image.title="Fax Compiler" \
      org.opencontainers.image.description="Production-ready Fax compiler runtime image" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.source="https://github.com/fax-lang/faxc" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0" \
      maintainer="Fax Team <fax-lang@example.com>" \
      stage="runtime"

# Set environment variables
ENV FAXC_HOME=/app \
    PATH="/app/bin:${PATH}" \
    # Security: Disable unnecessary features
    RUST_BACKTRACE=1 \
    # Performance: Use jemalloc if available (optional)
    # MALLOC_CONF="dirty_decay_ms:1000,muzzy_decay_ms:1000"

# Install runtime dependencies only (minimal set)
# - ca-certificates: For any HTTPS operations
# - libssl3: Runtime SSL library (if needed by the binary)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user and group for security (principle of least privilege)
# - User ID 1000 is standard for non-root users in containers
# - --disabled-password: No password authentication
# - --gecos "": Empty user info field
RUN groupadd --gid 1000 fax \
    && useradd --uid 1000 --gid 1000 --create-home --disabled-password --shell /bin/bash fax

# Set working directory
WORKDIR /app

# Create directory structure
RUN mkdir -p /app/bin /app/workspace && \
    chown -R fax:fax /app

# Copy the compiled binary from builder stage
COPY --from=builder --chown=fax:fax /app/target/release/faxc /app/bin/faxc

# Set proper permissions on the binary
RUN chmod +x /app/bin/faxc

# Switch to non-root user (security best practice)
USER fax:fax

# Set the working directory for user operations
WORKDIR /app/workspace

# Define volume for source code mounting (optional, for development)
VOLUME ["/app/workspace"]

# Health check to verify the compiler is functional
# QC-011 FIX: Tests actual compilation instead of just --help
# Creates a temp file, compiles simple program, verifies output, cleans up
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD cd /tmp && \
        echo 'fn main() { println!("health check ok"); }' > health.fax && \
        /app/bin/faxc health.fax -o health_check 2>&1 && \
        (./health_check 2>&1 | grep -q "health check ok" || exit 1) && \
        rm -f health.fax health_check && \
        echo "Health check passed" || exit 1

# Set entrypoint to the faxc binary
# Arguments can be passed via docker run fax:latest <args>
ENTRYPOINT ["/app/bin/faxc"]

# Default command shows help if no arguments provided
CMD ["--help"]

# =============================================================================
# Build Instructions:
# =============================================================================
# Build the image:
#   docker build -t fax:latest .
#
# Build with specific tag:
#   docker build -t fax:0.1.0 --target runtime .
#
# Build without cache (fresh build):
#   docker build --no-cache -t fax:latest .
#
# =============================================================================
# Usage Examples:
# =============================================================================
# Show help:
#   docker run --rm fax:latest --help
#
# Show version:
#   docker run --rm fax:latest --version
#
# Compile a Fax source file:
#   docker run --rm -v $(pwd):/app/workspace fax:latest source.fax
#
# Interactive mode:
#   docker run --rm -it -v $(pwd):/app/workspace fax:latest
#
# =============================================================================
# Security Notes:
# =============================================================================
# - Runs as non-root user (fax:fax, UID 1000)
# - Minimal base image (debian:bookworm-slim)
# - No build tools in runtime image
# - Stripped binary for reduced attack surface
# =============================================================================
