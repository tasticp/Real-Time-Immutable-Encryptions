# Rust Backend Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libopencv-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    libopencv-core4.6 \
    libopencv-imgproc4.6 \
    libopencv-imgcodecs4.6 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

# Copy binary from builder
COPY --from=builder /app/target/release/encryption-node /usr/local/bin/encryption-node
COPY --from=builder /app/target/release/verification-client /usr/local/bin/verification-client
COPY --from=builder /app/target/release/blockchain-anchor /usr/local/bin/blockchain-anchor

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/keys && \
    chown -R appuser:appuser /app

USER appuser
WORKDIR /app

# Expose ports
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["encryption-node", "--port", "8080"]