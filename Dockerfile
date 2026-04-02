FROM rust:1.94-alpine AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Build dependencies (layer caching)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src

# Build application
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ctq-api /usr/local/bin/

EXPOSE 9100

CMD ["ctq-api"]
