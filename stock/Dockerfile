# syntax=docker/dockerfile:1
#
# Multi-stage build: compile in the builder image, run in a slim runtime image.
#
# Build:
#   docker build -t stock:latest .
#
# Run:
#   docker run -d --name stock -p 3001:3001 -v ./data:/app/data stock:latest

###############################################################################
# Stage 1: Builder
###############################################################################
FROM rust:1.84-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies for Rust crates that link C libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      pkg-config \
      libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first to leverage Docker layer caching for dependency fetching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs so that `cargo build` can download & compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Now copy the real source code and rebuild
COPY src/ src/
COPY templates/ templates/
RUN cargo build --release

###############################################################################
# Stage 2: Runtime
###############################################################################
FROM debian:bookworm-slim

# Install CA certificates for HTTPS scraping, and runtime shared libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/stock /app/stock

# Copy runtime assets
COPY templates/ /app/templates/
COPY static/ /app/static/

# Create data directory for persistent commodity price files
RUN mkdir -p /app/data/commodities

EXPOSE 3001

USER nobody:nogroup

CMD ["/app/stock"]
