# Build stage
FROM rust:latest AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

# Runtime stage - minimal image for security
FROM debian:bookworm-slim

# Install runtime dependencies
# ca-certificates needed for HTTPS requests (reqwest dependency)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for running the application
# Using UID 1000 (common non-root UID) and GID 1000
RUN groupadd -r cubeapp -g 1000 && \
    useradd -r -u 1000 -g cubeapp -d /app -s /bin/false cubeapp && \
    mkdir -p /app /app/storage /app/db && \
    chown -R cubeapp:cubeapp /app

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder --chown=cubeapp:cubeapp /build/target/release/cube /app/cube

# Switch to non-root user
USER cubeapp

# Note: Core dumps are disabled via docker run --ulimit core=0
# or in docker-compose with ulimits: core: 0

# Expose port
EXPOSE 6272

# Run the application as non-root user
CMD ["./cube", "pruned", "signet", "node", "http://127.0.0.1:38332", "user", "password"]