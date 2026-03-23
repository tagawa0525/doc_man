default:
    @just --list

# --- 開発 ---

# サーバーをデバッグビルド
build:
    cargo build -p doc_man

# サーバーをリリースビルド
build-release:
    cargo build --release -p doc_man

# サーバーを起動（localhost:3000、フロントエンド配信なし）
run:
    FRONTEND_DIST_DIR=/dev/null cargo run -p doc_man

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

# バックエンド + フロントエンドを同時起動（DB は devShell で自動起動済み）
dev:
    -just dev-stop
    just run &
    just frontend-dev &
    wait

# バックエンド + フロントエンドを停止
dev-stop:
    -pkill -f 'target/debug/doc_man'
    -pkill -f 'trunk serve'

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

# --- Podman ---

# コンテナイメージをビルド
pod-build:
    podman compose build

# コンテナをバックグラウンドで起動（変更があれば自動リビルド）
pod-up:
    podman compose up -d --build

# コンテナを起動してシードデータを投入
pod-up-seed:
    podman compose up -d --build
    podman compose --profile seed run --rm seed

# コンテナを停止
pod-down:
    podman compose down

# コンテナを停止してボリュームも削除
pod-clean:
    podman compose down -v

# app のログを表示
pod-logs:
    podman compose logs -f app
