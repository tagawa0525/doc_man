# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

Nix devShell で開発環境が整う（`nix develop` or `direnv allow`）。シェル起動時にPostgreSQLが自動起動しマイグレーションが実行される。タスクランナーとして just を使用。

### Build / Run

just build                     # サーバーをデバッグビルド
just run                       # バックエンド localhost:3000
just frontend-build            # フロントエンド（WASM）
just frontend-dev              # フロントエンド localhost:8080（APIは3000へプロキシ）

### Test

cargo test                     # 全統合テスト（PostgreSQL必須、sqlx::testがテストDBを自動管理）
cargo test --test health       # 単一テストファイル実行
cargo test test_name           # 特定テスト関数の実行

### Lint

just lint                      # clippy all+pedantic（deny）
just fmt                       # rustfmt
just fmt-check                 # フォーマットチェックのみ

### Database

just db-reset                  # DB削除→再作成→マイグレーション
just db-seed                   # server/scripts/seed.sql を投入
just db-stop                   # PostgreSQL停止
just db-migrate                # マイグレーションのみ

### Podman

just pod-build                 # コンテナイメージをビルド
just pod-up                    # コンテナを起動
just pod-up-seed               # 起動+シードデータ投入
just pod-down                  # コンテナを停止
just pod-clean                 # 停止+ボリューム削除

## Architecture

Cargo workspace（resolver 3）で `server` と `frontend` の2クレート構成。`default-members = ["server"]`。

### Server (`server/`)

Axum + sqlx + PostgreSQL のREST APIサーバー。

- `lib.rs` — `MIGRATOR`（sqlx組込みマイグレーション）と `app_with_state()` を公開。テストと本番で共通利用
- `main.rs` — エントリーポイント。`BIND_ADDR` 環境変数でバインドアドレスを制御（デフォルト: `127.0.0.1:3000`）。起動時にマイグレーション自動実行
- `routes/mod.rs` — 全エンドポイントの登録。`/api/v1/*` 配下のCRUDルート
- `auth.rs` — `AuthenticatedUser` Axum extractor。`Authorization: Bearer {employee_code}` でDB認証
- `error.rs` — `AppError` enum → HTTP status + `{ error: { code, message } }` JSON
- `handlers/` — HTTPハンドラ（リクエスト/レスポンス変換）
- `services/` — ビジネスロジック（document_numbering: advisory lockによる採番）
- `models/` — リクエスト/レスポンスDTO
- `pagination.rs` — `PaginationParams` クエリパラメータ
- `state.rs` — `AppState { db: PgPool }`
- `migrations/` — sqlx SQLマイグレーション（15ファイル）

### Frontend (`frontend/`)

Leptos CSR + Trunk のWASM SPA。

- `api/client.rs` — HTTPクライアント（トークン管理）
- `api/types.rs` — 全リクエスト/レスポンス型。`CodeBrief`/`NameBrief` が共通brief型
- `auth.rs` — `AuthContext`、`Role`
- `components/` — 再利用UIコンポーネント
- `pages/` — 各エンティティのCRUDページ

### Container (`Containerfile` / `docker-compose.yml`)

マルチステージビルド（frontend-builder → server-builder → runtime）。Podman Compose で PostgreSQL + app を起動。

## Testing Patterns

統合テストは `server/tests/` に配置。各テスト関数は `#[sqlx::test(migrator = "doc_man::MIGRATOR")]` を使い、テストごとに独立したDBを自動作成・破棄する。

- `tests/helpers/mod.rs` — `build_test_app(pool)`, `parse_body()`, `insert_*()` 等のヘルパー群
- テストはAxumの `oneshot()` でHTTPリクエストを直接送信（実サーバー不要）

## Clippy Configuration

workspace レベルで `clippy::all` + `clippy::pedantic` を deny に設定。許可項目は `Cargo.toml` の `[workspace.lints.clippy]` を参照。
