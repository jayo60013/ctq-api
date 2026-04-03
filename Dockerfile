FROM rust:1.94-alpine AS builder

RUN apk add --no-cache build-base musl-dev

WORKDIR /app

RUN cargo install cargo-chef

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.94-alpine AS cacher
RUN apk add --no-cache build-base musl-dev
WORKDIR /app

RUN cargo install cargo-chef
COPY --from=builder /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

# ---------- BUILD APP ----------
FROM rust:1.94-alpine AS builder-final
RUN apk add --no-cache build-base musl-dev
WORKDIR /app

COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release

# ---------- RUNTIME ----------
FROM scratch

# Copy app
COPY --from=builder-final /app/target/release/ctq-api /ctq-api

# Provide CA certificates
COPY --from=builder-final /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

EXPOSE 9100
ENTRYPOINT ["/ctq-api"]