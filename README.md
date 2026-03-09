# Doc Man — 文書管理システム

社内文書のライフサイクル（登録・承認・回覧）を管理する Web アプリケーション。

## 技術スタック

| レイヤー       | 技術                         |
| -------------- | ---------------------------- |
| バックエンド   | Rust / Axum / sqlx           |
| フロントエンド | Rust / Leptos (WASM) / Trunk |
| データベース   | PostgreSQL 17                |
| 開発環境       | Nix flake / direnv           |
| コンテナ       | Podman / Podman Compose      |
| タスクランナー | just                         |

## プロジェクト構成

```text
doc_man/
├── server/               # Axum REST API サーバー
│   ├── src/
│   │   ├── lib.rs        # app_with_state(), MIGRATOR
│   │   ├── main.rs       # エントリーポイント
│   │   ├── auth.rs       # AuthenticatedUser extractor
│   │   ├── error.rs      # AppError → HTTP レスポンス
│   │   ├── handlers/     # HTTPハンドラ
│   │   ├── services/     # ビジネスロジック
│   │   ├── models/       # リクエスト/レスポンス DTO
│   │   ├── routes/       # ルーティング定義
│   │   ├── migrations/   # SQLマイグレーション
│   │   └── scripts/      # seed.sql
│   └── tests/            # 統合テスト
├── frontend/             # Leptos CSR SPA
│   └── src/
│       ├── api/          # HTTPクライアント・型定義
│       ├── auth.rs       # 認証コンテキスト
│       ├── components/   # 再利用UIコンポーネント
│       └── pages/        # CRUDページ
├── Containerfile
├── docker-compose.yml
├── Justfile
└── flake.nix
```

## セットアップ

### Nix 開発環境（推奨）

[Nix](https://nixos.org/download/) と [direnv](https://direnv.net/) が必要。

```bash
direnv allow        # 開発環境を有効化（PostgreSQL の自動起動・マイグレーションを含む）
```

または:

```bash
nix develop
```

### Podman によるデプロイ

Podman と Podman Compose が必要。

```bash
# イメージビルド
just pod-build

# 起動
just pod-up

# 起動確認
curl http://localhost:3000/health
# => {"status":"ok"}

# シードデータ投入
just pod-up-seed

# 停止
just pod-down

# 停止 + ボリューム削除
just pod-clean
```

## 開発ワークフロー

### サーバー起動

```bash
# Nix 環境内で
just run                    # localhost:3000
```

環境変数:

| 変数                | デフォルト       | 説明                               |
| ------------------- | ---------------- | ---------------------------------- |
| `DATABASE_URL`      | （必須）         | PostgreSQL 接続 URL                |
| `BIND_ADDR`         | `127.0.0.1:3000` | バインドアドレス                   |
| `FRONTEND_DIST_DIR` | `frontend/dist`  | フロントエンドの dist ディレクトリ |
| `RUST_LOG`          | -                | ログレベル（例: `info`, `debug`）  |

### フロントエンド開発

```bash
just frontend-dev           # localhost:8080（API は 3000 へプロキシ）
```

### データベース操作

```bash
just db-reset               # DB 削除 → 再作成 → マイグレーション
just db-seed                # シードデータ投入
just db-reset-seed          # リセット + シード
just db-migrate             # マイグレーションのみ
```

### テスト

```bash
just test                   # 全統合テスト（sqlx::test が DB を自動管理）
cargo test test_name        # 特定テスト
```

### コード品質

```bash
just lint                   # cargo clippy (all + pedantic)
just fmt                    # cargo fmt
just fmt-check              # フォーマットチェックのみ
```

## API 概要

### 認証

`Authorization: Bearer {employee_code}` ヘッダーで認証。

```bash
curl -H "Authorization: Bearer EMP001" http://localhost:3000/api/v1/me
```

### エンドポイント一覧

| メソッド       | パス                                                   | 説明                   |
| -------------- | ------------------------------------------------------ | ---------------------- |
| GET            | `/health`                                              | ヘルスチェック         |
| GET            | `/api/v1/me`                                           | 認証ユーザー情報       |
| GET/POST       | `/api/v1/departments`                                  | 部門一覧・作成         |
| GET/PUT        | `/api/v1/departments/{id}`                             | 部門取得・更新         |
| GET/POST       | `/api/v1/employees`                                    | 社員一覧・作成         |
| GET/PUT        | `/api/v1/employees/{id}`                               | 社員取得・更新         |
| GET/POST       | `/api/v1/disciplines`                                  | 分野一覧・作成         |
| GET/PUT        | `/api/v1/disciplines/{id}`                             | 分野取得・更新         |
| GET/POST       | `/api/v1/document-kinds`                               | 文書種別一覧・作成     |
| GET/PUT        | `/api/v1/document-kinds/{id}`                          | 文書種別取得・更新     |
| GET/POST       | `/api/v1/document-registers`                           | 文書台帳一覧・作成     |
| GET/PUT        | `/api/v1/document-registers/{id}`                      | 文書台帳取得・更新     |
| GET/POST       | `/api/v1/projects`                                     | プロジェクト一覧・作成 |
| GET/PUT/DELETE | `/api/v1/projects/{id}`                                | プロジェクト操作       |
| GET/POST       | `/api/v1/documents`                                    | 文書一覧・作成         |
| GET/PUT/DELETE | `/api/v1/documents/{id}`                               | 文書操作               |
| GET/POST       | `/api/v1/documents/{id}/approval-steps`                | 承認ルート取得・設定   |
| POST           | `/api/v1/documents/{id}/approval-steps/{step}/approve` | 承認                   |
| POST           | `/api/v1/documents/{id}/approval-steps/{step}/reject`  | 差し戻し               |
| GET/POST       | `/api/v1/documents/{id}/circulations`                  | 回覧一覧・作成         |
| POST           | `/api/v1/documents/{id}/circulations/confirm`          | 回覧確認               |
| GET/POST       | `/api/v1/tags`                                         | タグ一覧・作成         |

エラーレスポンス形式:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "document not found"
  }
}
```

## Just コマンド一覧

```text
just --list
```

| コマンド              | 説明                             |
| --------------------- | -------------------------------- |
| `just build`          | サーバーをデバッグビルド         |
| `just build-release`  | サーバーをリリースビルド         |
| `just run`            | サーバーを起動                   |
| `just test`           | 統合テストを実行                 |
| `just lint`           | clippy でリント                  |
| `just fmt`            | rustfmt でフォーマット           |
| `just fmt-check`      | フォーマットチェック             |
| `just frontend-build` | フロントエンドをビルド           |
| `just frontend-dev`   | フロントエンド開発サーバー起動   |
| `just db-migrate`     | マイグレーションを実行           |
| `just db-stop`        | PostgreSQL を停止                |
| `just db-reset`       | DB をリセット                    |
| `just db-seed`        | シードデータを投入               |
| `just db-reset-seed`  | リセット + シード                |
| `just pod-build`      | コンテナイメージをビルド         |
| `just pod-up`         | コンテナを起動                   |
| `just pod-up-seed`    | コンテナを起動してシードを投入   |
| `just pod-down`       | コンテナを停止                   |
| `just pod-clean`      | コンテナを停止しボリュームも削除 |
| `just pod-logs`       | app のログを表示                 |
