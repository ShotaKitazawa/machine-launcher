# ===== Build frontend Stage =====
FROM rust:1 AS frontend-builder
WORKDIR /app/
COPY common ./common
COPY frontend ./frontend
RUN rustup target add wasm32-unknown-unknown \
      && cargo install --locked trunk \
      && cd frontend && trunk build

# ===== Build backend Stage =====
FROM rust:1 AS backend-builder
WORKDIR /app/
COPY common ./common
COPY backend ./backend
RUN cd backend && cargo build --release

# ===== Runtime Stage =====
FROM debian:bookworm-slim

RUN apt-get update \
      && apt-get install -y libssl-dev ca-certificates \
      && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend
COPY --from=backend-builder /app/backend/target/release/machine-launcher ./machine-launcher
COPY --from=frontend-builder /app/frontend/public ../frontend/public
COPY --from=frontend-builder /app/frontend/dist ../frontend/dist
ENTRYPOINT ["./machine-launcher"]
