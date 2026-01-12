FROM docker.io/library/rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies by building a dummy project first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p src/bin && \
    echo "fn main() {}" > src/bin/seed.rs && \
    echo "fn main() {}" > src/bin/compress.rs && \
    cargo build --release && \
    rm -rf src

COPY . .

RUN cargo build --release

FROM docker.io/library/debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/tulpar-api /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/seed /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/compress /usr/local/bin/

# Create non-root user (UID 10001) for rootless Podman compatibility
RUN useradd -r -u 10001 -g root tulpar && \
    mkdir -p /app/data /app/storage && \
    chown -R tulpar:root /app

ENV RUST_LOG=info \
    DATABASE_URL=sqlite:/app/data/data.db \
    STORAGE_PATH=/app/storage

USER tulpar

EXPOSE 3000

CMD ["tulpar-api"]
