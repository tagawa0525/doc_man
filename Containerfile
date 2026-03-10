# Stage 1: フロントエンドビルド（WASM）
FROM rust:1.85-bookworm AS frontend-builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install trunk --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml

# 依存関係キャッシュレイヤー: 空のソースでビルドしてキャッシュ活用
RUN mkdir -p server/src && echo "fn main() {}" > server/src/main.rs && \
    mkdir -p server/src && echo "pub fn lib() {}" > server/src/lib.rs && \
    mkdir -p frontend/src && echo "fn main() {}" > frontend/src/main.rs

COPY frontend/ frontend/
RUN cd frontend && trunk build --release


# Stage 2: サーバービルド
FROM rust:1.85-bookworm AS server-builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/Cargo.toml
COPY frontend/Cargo.toml frontend/Cargo.toml

# 依存関係キャッシュレイヤー
RUN mkdir -p server/src && echo "fn main() {}" > server/src/main.rs && \
    mkdir -p server/src && echo "" > server/src/lib.rs && \
    mkdir -p frontend/src && echo "fn main() {}" > frontend/src/main.rs && \
    cargo build --release -p doc_man 2>/dev/null; true

COPY server/ server/
# タイムスタンプを更新してキャッシュを無効化
RUN touch server/src/main.rs server/src/lib.rs && \
    cargo build --release -p doc_man


# Stage 3: ランタイム
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=server-builder /app/target/release/doc_man ./doc_man
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist

ENV BIND_ADDR="0.0.0.0:3000"
ENV FRONTEND_DIST_DIR="/app/frontend/dist"
ENV RUST_LOG="info"

EXPOSE 3000

CMD ["./doc_man"]
