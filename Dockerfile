# ==================== STAGE 1: BUILD ====================
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Copiamos solo los archivos de Cargo primero (mejor caching)
COPY server/Cargo.toml server/Cargo.lock* ./server/
WORKDIR /app/server
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Ahora copiamos el código real
COPY server/src ./src

# Build final
RUN cargo build --release

# Copiamos la carpeta web (¡importante!)
COPY web /app/web

# ==================== STAGE 2: RUNTIME ====================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copiamos el binario compilado
COPY --from=builder /app/server/target/release/server ./radan

# Copiamos la carpeta web
COPY --from=builder /app/web ./web

EXPOSE 3000

CMD ["./radan"]