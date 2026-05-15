# ==================== STAGE 1: BUILD ====================
FROM rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

# Copiamos Cargo files primero (para caching)
COPY server/Cargo.toml server/Cargo.lock* ./server/
WORKDIR /app/server

RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm src/main.rs

# Copiamos el código fuente y compilamos de verdad
COPY server/src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# ==================== STAGE 2: RUNTIME ====================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiamos el binario correcto (se llama "server")
COPY --from=builder /app/server/target/x86_64-unknown-linux-musl/release/server ./server

# Copiamos la carpeta web
COPY web ./web

EXPOSE 3000

# Ejecutamos el binario correcto
CMD ["./server"]