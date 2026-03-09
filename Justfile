default:
    @just --list

# --- 開発 ---

# サーバーをデバッグビルド
build:
    cargo build -p doc_man

# サーバーをリリースビルド
build-release:
    cargo build --release -p doc_man

# サーバーを起動（localhost:3000）
run:
    cargo run -p doc_man

# 統合テストを実行
test:
    cargo test

# clippy（all + pedantic）
lint:
    cargo clippy --workspace --all-targets

# rustfmt でフォーマット
fmt:
    cargo fmt --all

# フォーマットチェックのみ
fmt-check:
    cargo fmt --all -- --check

# --- フロントエンド ---

# フロントエンドをリリースビルド
frontend-build:
    cd frontend && trunk build --release

# フロントエンド開発サーバー（localhost:8080）
frontend-dev:
    cd frontend && trunk serve

# --- データベース ---

# マイグレーションを実行
db-migrate:
    sqlx migrate run --source server/migrations

# PostgreSQL を停止
db-stop:
    pg_ctl stop -m fast

# DB を削除して再作成しマイグレーション
db-reset:
    dropdb -h localhost -U doc_man doc_man || true
    createdb -h localhost -U doc_man doc_man
    sqlx migrate run --source server/migrations

# シードデータを投入
db-seed:
    psql -h localhost -U doc_man -d doc_man -f server/scripts/seed.sql

# DB をリセットしてシードデータを投入
db-reset-seed: db-reset db-seed

# --- Docker ---

# Docker イメージをビルド
docker-build:
    docker compose build

# コンテナをバックグラウンドで起動
docker-up:
    docker compose up -d

# コンテナを起動してシードデータを投入
docker-up-seed:
    docker compose up -d
    docker compose --profile seed run --rm seed

# コンテナを停止
docker-down:
    docker compose down

# コンテナを停止してボリュームも削除
docker-clean:
    docker compose down -v

# app のログを表示
docker-logs:
    docker compose logs -f app
