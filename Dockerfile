# Multi-stage build for Rust web scraper
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary for web server
RUN cargo build --release --bin scraper-web

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/scraper-web /usr/local/bin/scraper-web

# Create output directory
RUN mkdir -p /app/output

# Expose port
EXPOSE 8080

# Set environment
ENV RUST_LOG=info

# Run the web server
CMD ["scraper-web"]
