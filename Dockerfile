FROM rust:1.94-alpine AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/crack-the-quote-api /usr/local/bin/

EXPOSE 9100

CMD ["crack-the-quote-api"]
