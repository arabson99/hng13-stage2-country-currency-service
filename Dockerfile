# Stage 1: Builder
# Use a specific Rust version for consistency
FROM rust:1.85-bookworm AS builder

# Install sqlx-cli
RUN cargo install sqlx-cli --version 0.7

WORKDIR /app

# Copy migrations *first*
COPY ./migrations ./migrations

# Copy dependencies and build a cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Copy the rest of the source code and assets
COPY ./src ./src
COPY ./DejaVuSans.ttf ./DejaVuSans.ttf

# --- THIS IS THE KEY ---
# Railway provides the DATABASE_URL at build-time.
# We run migrations *before* building the main app.
RUN sqlx migrate run

# Build the application. The sqlx::query! macros will now
# find the tables and compile successfully.
RUN cargo build --release

# --- Stage 2: Final Image ---
# Use a minimal image for a small footprint
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies for reqwest (TLS)
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
# !!! MAKE SURE THIS NAME MATCHES YOUR Cargo.toml !!!
COPY --from=builder /app/target/release/hng13-stage2-country-currency-service .

# Copy assets and migrations (for running at startup)
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/DejaVuSans.ttf ./DejaVuSans.ttf

# Set environment variables (Railway will override PORT)
ENV RUST_LOG=info
ENV PORT=8080

# Expose the port
EXPOSE 8080

# !!! MAKE SURE THIS NAME MATCHES YOUR Cargo.toml !!!
CMD ["./hng13-stage2-country-currency-service"]