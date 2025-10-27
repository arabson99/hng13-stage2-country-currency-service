# Stage 1: Builder
# Use your local Rust version for consistency
FROM rust:1.90-bookworm AS builder

WORKDIR /app

# Copy dependencies and build a cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Copy the rest of the source code and assets
COPY ./src ./src
COPY ./DejaVuSans.ttf ./DejaVuSans.ttf
COPY ./migrations ./migrations

# Build the application
# This works because your db.rs now uses runtime-checked queries
RUN cargo build --release

# --- Stage 2: Final Image ---
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

ENV RUST_LOG=info
ENV PORT=8080
EXPOSE 8080

# !!! MAKE SURE THIS NAME MATCHES YOUR Cargo.toml !!!
CMD ["./hng13-stage2-country-currency-service"]