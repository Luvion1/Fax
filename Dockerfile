# Fax Compiler Docker Image
# Multi-stage build for optimal size

# Stage 1: Build environment
FROM ubuntu:22.04 AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    clang \
    llvm \
    lld \
    protobuf-compiler \
    libprotobuf-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Install Lean 4
RUN curl https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh -sSf | sh -s -- -y
ENV PATH="/root/.elan/bin:${PATH}"

# Set working directory
WORKDIR /build

# Copy source code
COPY . .

# Build the project
RUN lake build

# Stage 2: Runtime environment
FROM ubuntu:22.04 AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    clang \
    llvm \
    lld \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy built binaries from builder
COPY --from=builder /build/.lake/build/bin/faxc /usr/local/bin/faxc
COPY --from=builder /root/.elan/bin/lake /usr/local/bin/lake

# Create non-root user
RUN useradd -m -s /bin/bash fax

# Set working directory
WORKDIR /workspace

# Switch to non-root user
USER fax

# Default command
ENTRYPOINT ["faxc"]
CMD ["--help"]
