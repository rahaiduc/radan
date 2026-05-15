FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY server/Cargo.toml server/Cargo.lock* ./server/
WORKDIR /app/server

RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm src/main.rs

COPY server/src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libgcc-s1 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/server/target/release/server ./server
COPY web ./web

EXPOSE 3000

CMD ["./server"]
