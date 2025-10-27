# Stage 1: Builder
FROM rust:1.90-bookworm AS builder

# Install sqlx-cli (required for migrations)
RUN cargo install sqlx-cli --version '^0.7'

WORKDIR /app

# Copy migrations first to leverage Docker caching
COPY ./migrations ./migrations

# Copy dependencies and build a cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# Copy the rest of the source code and assets
COPY ./src ./src
COPY ./DejaVuSans.ttf ./DejaVuSans.ttf

# Build the application with the sqlx macros
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
COPY --from=builder /app/target/release/hng13-stage2-country-currency-service ./

# Copy assets and migrations
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/DejaVuSans.ttf ./DejaVuSans.ttf

# Set environment variables (Railway will override PORT)
ENV RUST_LOG=info
ENV PORT=8080
ENV DATABASE_URL=mysql://root:RsSsHEhhBXbQLdhEIlVaPxxFpUqwwPrp@interchange.proxy.rlwy.net:21342/railway

# Expose the app port
EXPOSE 8080

# Start the application
CMD ["./hng13-stage2-country-currency-service"]
