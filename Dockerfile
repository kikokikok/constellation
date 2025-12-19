# Constellation Dockerfile
# Multi-stage build for production-ready Constellation application

# Stage 1: Builder
FROM rust:1.80-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy source files
COPY . .

# Build in release mode
RUN cargo build --release --package constellation-core --examples

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 constellation

# Copy binaries from builder
COPY --from=builder /app/target/release/examples/iggy_message_broker_example /usr/local/bin/
COPY --from=builder /app/target/release/examples/http_server_example /usr/local/bin/

# Create necessary directories
RUN mkdir -p /etc/constellation /var/lib/constellation /var/log/constellation \
    && chown -R constellation:constellation /etc/constellation /var/lib/constellation /var/log/constellation

# Switch to non-root user
USER constellation

# Set working directory
WORKDIR /home/constellation

# Expose ports
# - 8080: HTTP API (if implemented)
# - 8090: Iggy server (default Iggy port)
EXPOSE 8080 8090

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command (can be overridden)
CMD ["http_server_example"]