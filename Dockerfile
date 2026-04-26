# ===== Build frontend Stage =====
FROM rust:1 AS frontend-builder
WORKDIR /app/
COPY common ./common
COPY frontend ./frontend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/frontend/target \
    rustup target add wasm32-unknown-unknown \
      && curl -sSfL https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz \
         | tar -xz -C /usr/local/bin trunk \
      && cd frontend && trunk build

# ===== Build backend Stage =====
FROM rust:1 AS backend-builder
WORKDIR /app/
COPY common ./common
COPY backend ./backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/backend/target \
    cd backend && cargo build --release \
      && cp target/release/machine-launcher /app/machine-launcher

# ===== Runtime Stage =====
FROM debian:bookworm-slim

RUN apt-get update \
      && apt-get install -y libssl-dev ca-certificates \
      && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend
COPY --from=backend-builder /app/machine-launcher ./machine-launcher
COPY --from=frontend-builder /app/frontend/public ../frontend/public
COPY --from=frontend-builder /app/frontend/dist ../frontend/dist
ENTRYPOINT ["./machine-launcher"]
