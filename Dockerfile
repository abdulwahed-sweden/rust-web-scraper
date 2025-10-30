# Multi-stage build for Rust web scraper
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and static files from builder
COPY --from=builder /app/target/release/rust-web-scraper /usr/local/bin/rust-web-scraper
COPY static ./static

# Create output directory
RUN mkdir -p /app/output

# Expose port
EXPOSE 8080

# Set environment
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Run the web server
CMD ["rust-web-scraper"]
