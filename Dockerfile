# ===== Build frontend Stage =====
FROM rust:1 AS frontend-builder
WORKDIR /app/
COPY Makefile .
COPY utils ./utils
COPY frontend ./frontend
RUN make build-frontend

# ===== Build backend Stage =====
FROM rust:1 AS backend-builder
WORKDIR /app/
COPY Makefile .
COPY utils ./utils
COPY backend ./backend
COPY --from=frontend-builder /app/frontend/public /app/frontend/public
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
RUN make build-backend

# ===== Runtime Stage =====
FROM debian:bookworm-slim

RUN apt-get update \
      && apt-get install -y libssl-dev ca-certificates \
      && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/backend/target/release/machine-launcher ./machine-launcher
ENTRYPOINT ["./machine-launcher"]
