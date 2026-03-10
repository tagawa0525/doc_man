# Docker / Justfile / README.md 追加計画

## Context

doc_man には開発環境として Nix flake が整備されているが、Docker によるコンテナ化、Justfile によるタスクランナー、README.md によるプロジェクト説明がまだ存在しない。これらを追加し、開発・デプロイの運用方法を整備する。

## 前提確認

- `sqlx::query!()` マクロ（コンパイル時 DB 接続が必要）は未使用。全て `sqlx::query()` （ランタイム）なので Docker ビルド時に DB 不要
- `sqlx::migrate!()` でマイグレーションファイルがバイナリに埋め込まれている（`lib.rs` の `MIGRATOR` static）
- サーバーは `127.0.0.1:3000` にハードコードされているため、Docker 用に設定可能にする必要あり

## 変更対象ファイル

### 修正

| ファイル             | 変更内容                                                             |
| -------------------- | -------------------------------------------------------------------- |
| `server/src/main.rs` | `BIND_ADDR` 環境変数対応、起動時マイグレーション実行、tracing 初期化 |
| `flake.nix`          | `just` を buildInputs に追加                                         |
| `.gitignore`         | Docker 関連の除外パターン追加                                        |

### 新規作成

| ファイル             | 内容                                                          |
| -------------------- | ------------------------------------------------------------- |
| `Containerfile`      | マルチステージビルド（frontend + server + runtime）           |
| `docker-compose.yml` | PostgreSQL + app サービス（Podman Compose で使用）            |
| `.containerignore`   | ビルドコンテキストの除外設定                                  |
| `Justfile`           | 開発・Podman・DB 操作コマンド（`pod-*` プレフィックス）       |
| `README.md`          | プロジェクト概要と運用方法                                    |

## 実装詳細

### 1. `server/src/main.rs` の修正

```rust
use doc_man::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bind_addr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    doc_man::MIGRATOR.run(&pool).await.expect("failed to run migrations");

    let state = AppState { db: pool };
    let app = doc_man::app_with_state(state);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("failed to bind");

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server error");
}
```

変更点:

- `BIND_ADDR` 環境変数（デフォルト: `127.0.0.1:3000`、コンテナ環境では `0.0.0.0:3000`）
- `MIGRATOR.run()` で起動時にマイグレーション自動実行（冪等）
- `tracing_subscriber` 初期化（`RUST_LOG` で制御可能に）

### 2. `Containerfile`（マルチステージビルド）

```rust
Stage 1: frontend-builder
  - rust:1.85-bookworm + wasm32-unknown-unknown + trunk
  - trunk build --release
  - 出力: /app/frontend/dist/

Stage 2: server-builder
  - rust:1.85-bookworm
  - cargo build --release -p doc_man
  - 依存キャッシュレイヤー: Cargo.toml/Cargo.lock → 空ソースでビルド → 実ソースコピー
  - 出力: /app/target/release/doc_man

Stage 3: runtime
  - debian:bookworm-slim + libpq5 + ca-certificates
  - バイナリ + frontend/dist をコピー
  - BIND_ADDR=0.0.0.0:3000, FRONTEND_DIST_DIR=/app/frontend/dist
  - EXPOSE 3000
```

BuildKit により Stage 1 と Stage 2 は並列ビルド可能。

### 3. `docker-compose.yml`

```yaml
services:
  db:
    image: postgres:17
    environment:
      POSTGRES_USER: doc_man
      POSTGRES_PASSWORD: doc_man
      POSTGRES_DB: doc_man
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U doc_man"]
      interval: 2s
      timeout: 5s
      retries: 10

  app:
    build: .
    environment:
      DATABASE_URL: postgres://doc_man:doc_man@db:5432/doc_man
      BIND_ADDR: "0.0.0.0:3000"
      FRONTEND_DIST_DIR: /app/frontend/dist
    ports:
      - "3000:3000"
    depends_on:
      db:
        condition: service_healthy

  seed:
    image: postgres:17
    command: psql -h db -U doc_man -d doc_man -f /seed/seed.sql
    environment:
      PGPASSWORD: doc_man
    volumes:
      - ./server/scripts/seed.sql:/seed/seed.sql:ro
    depends_on:
      db:
        condition: service_healthy
    profiles:
      - seed

volumes:
  pgdata:
```

- `seed` サービスは `--profile seed` 指定時のみ実行

### 4. `.containerignore`

```text
target/
.pgdata/
.direnv/
.git/
.claude/
docs/plans/
```

### 5. `Justfile`

```just
default:
    @just --list

# --- 開発 ---
build:                 cargo build -p doc_man
build-release:         cargo build --release -p doc_man
run:                   cargo run -p doc_man
test:                  cargo test -p doc_man
lint:                  cargo clippy --workspace --all-targets
fmt:                   cargo fmt --all
fmt-check:             cargo fmt --all -- --check

# --- フロントエンド ---
frontend-build:        cd frontend && trunk build --release
frontend-dev:          cd frontend && trunk serve

# --- データベース ---
db-migrate:            sqlx migrate run --source server/migrations
db-reset:              (drop + create + migrate)
db-seed:               psql でシードデータ投入
db-reset-seed:         db-reset + db-seed

# --- Podman ---
pod-build:             podman compose build
pod-up:                podman compose up -d
pod-up-seed:           podman compose up -d && podman compose --profile seed run --rm seed
pod-down:              podman compose down
pod-clean:             podman compose down -v
pod-logs:              podman compose logs -f app
```

### 6. `flake.nix` の修正

`buildInputs` に `pkgs.just` を追加。

### 7. `README.md`

構成:

1. プロジェクト概要（文書管理システムの説明）
2. 技術スタック
3. プロジェクト構成（ディレクトリツリー）
4. セットアップ方法
   - Nix 開発環境（推奨）
   - Podman Compose によるデプロイ
5. 開発ワークフロー（サーバー起動、フロントエンド開発、DB 操作、テスト、リント）
6. API 概要（エンドポイント一覧、認証方式）
7. Just コマンド一覧

## コミット順序

1. `feat: make server bind address configurable and add auto-migration` — main.rs 修正
2. `feat: add Containerfile and .containerignore` — コンテナビルド設定
3. `feat: add docker-compose.yml` — コンテナオーケストレーション（Podman Compose）
4. `feat: add Justfile and just to flake.nix` — タスクランナー
5. `docs: add README.md` — プロジェクト説明と運用方法
6. `chore: update .gitignore` — Docker 関連除外

## 検証方法

1. `just build` / `just lint` / `just fmt-check` — ビルド・品質チェック
2. `just test` — 既存テストが通ること（main.rs の変更が破壊的でないこと）
3. `just pod-build` — コンテナイメージがビルドできること
4. `just pod-up` → `curl http://localhost:3000/health` — コンテナ起動確認
5. `just pod-up-seed` → API でデータ取得確認
