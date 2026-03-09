# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

Nix devShell で開発環境が整う（`nix develop` or `direnv allow`）。シェル起動時にPostgreSQLが自動起動しマイグレーションが実行される。

### Build

cargo build                    # サーバー（default member）
cd frontend && trunk build     # フロントエンド（WASM）

### Run

cargo run -p doc_man           # バックエンド localhost:3000
cd frontend && trunk serve     # フロントエンド localhost:8080（APIは3000へプロキシ）

### Test

cargo test                     # 全統合テスト（PostgreSQL必須、sqlx::testがテストDBを自動管理）
cargo test --test health       # 単一テストファイル実行
cargo test test_name           # 特定テスト関数の実行

### Lint

cargo clippy                   # clippy all+pedantic（deny）
cargo fmt                      # rustfmt
cargo fmt -- --check           # フォーマットチェックのみ

### Database

just db-reset                  # DB削除→再作成→マイグレーション
just db-seed                   # server/scripts/seed.sql を投入
just db-stop                   # PostgreSQL停止

## Architecture

Cargo workspace（resolver 3）で `server` と `frontend` の2クレート構成。`default-members = ["server"]`。

### Server (`server/`)

Axum + sqlx + PostgreSQL のREST APIサーバー。

- `lib.rs` — `MIGRATOR`（sqlx組込みマイグレーション）と `app_with_state()` を公開。テストと本番で共通利用
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

## Testing Patterns

統合テストは `server/tests/` に配置。各テスト関数は `#[sqlx::test(migrator = "doc_man::MIGRATOR")]` を使い、テストごとに独立したDBを自動作成・破棄する。

- `tests/helpers/mod.rs` — `build_test_app(pool)`, `parse_body()`, `insert_*()` 等のヘルパー群
- テストはAxumの `oneshot()` でHTTPリクエストを直接送信（実サーバー不要）

## Clippy Configuration

workspace レベルで `clippy::all` + `clippy::pedantic` を deny に設定。許可項目は `Cargo.toml` の `[workspace.lints.clippy]` を参照。
