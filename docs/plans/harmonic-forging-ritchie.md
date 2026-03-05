# 実装計画: doc_man REST API

## Context

設計文書（`docs/design/`）が完成し、実装フェーズへ移行する。
Rust + Axum + sqlx (PostgreSQL) で全 REST API エンドポイントを実装する。
認証は Bearer スタブ（`employee_code` をそのままトークンとして DB 検索）。
TDD（RED→GREEN→REFACTOR）を各機能単位で厳守し、PR 単位で段階的にマージする。

---

## スタック

| 項目          | 選択                                            |
| ------------- | ----------------------------------------------- |
| Web framework | axum 0.8                                        |
| DB ドライバ   | sqlx 0.8 (PostgreSQL, compile-time query check) |
| 非同期        | tokio 1                                         |
| 認証          | Bearer スタブ（JWT は後回し）                   |
| エラー        | thiserror 2                                     |
| テスト        | `#[sqlx::test]` マクロ（テストごとに独立 DB）   |

---

## Cargo.toml 依存クレート

```toml
[dependencies]
axum            = { version = "0.8", features = ["macros"] }
tower           = { version = "0.5", features = ["util"] }
tower-http      = { version = "0.6", features = ["trace", "cors"] }
tokio           = { version = "1",   features = ["full"] }
sqlx            = { version = "0.8", features = ["runtime-tokio-rustls","postgres","uuid","chrono","migrate"] }
serde           = { version = "1",   features = ["derive"] }
serde_json      = "1"
uuid            = { version = "1",   features = ["v4","serde"] }
chrono          = { version = "0.4", features = ["serde"] }
thiserror       = "2"
tracing         = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter","fmt"] }

[dev-dependencies]
axum-test = "15"
```

---

## ディレクトリ構成

```text
src/
├── main.rs                   # Tokio エントリーポイント
├── lib.rs                    # テストから app() を呼べるよう公開
├── config.rs                 # 設定（DATABASE_URL 等）
├── error.rs                  # AppError + IntoResponse
├── auth.rs                   # AuthenticatedUser Extractor (スタブ)
├── state.rs                  # AppState { db: PgPool }
├── pagination.rs             # PaginationParams, PaginatedResponse<T>
├── routes/mod.rs             # build_router()
├── handlers/
│   ├── departments.rs
│   ├── employees.rs
│   ├── disciplines.rs
│   ├── projects.rs
│   ├── document_kinds.rs
│   ├── document_registers.rs
│   ├── documents.rs
│   ├── approval_steps.rs
│   ├── circulations.rs
│   └── tags.rs
├── models/
│   └── {各テーブル対応 .rs}  # FromRow 行型 + Request/Response 型
└── services/
    ├── document_numbering.rs  # assign_doc_number() + リトライ
    ├── approval.rs            # find_active_step / approve / reject
    └── circulation.rs         # start / confirm

migrations/
├── 20260305000001_create_departments.sql
├── 20260305000002_create_employees.sql
├── 20260305000003_create_employee_departments.sql
├── 20260305000004_create_disciplines.sql
├── 20260305000005_create_projects.sql
├── 20260305000006_create_document_kinds.sql
├── 20260305000007_create_document_registers.sql
├── 20260305000008_create_documents.sql
├── 20260305000009_create_tags.sql
├── 20260305000010_create_document_tags.sql
├── 20260305000011_create_approval_steps.sql
├── 20260305000012_create_circulations.sql
└── 20260305000013_create_path_scan_issues.sql

tests/
├── helpers/mod.rs             # insert_test_* / build_test_app ヘルパー
└── integration/
    ├── departments.rs
    ├── employees.rs
    ├── ...
```

---

## 主要な型定義

### AppError（`src/error.rs`）

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]        NotFound(String),
    #[error("unauthorized")]           Unauthorized,
    #[error("forbidden: {0}")]        Forbidden(String),
    #[error("invalid request: {0}")]  InvalidRequest(String),
    #[error("conflict: {0}")]         Conflict(String),
    #[error("unprocessable: {0}")]    Unprocessable(String),
    #[error("database error: {0}")]   Database(#[from] sqlx::Error),
    #[error("internal error: {0}")]   Internal(String),
}
// IntoResponse: { "error": { "code": "...", "message": "..." } }
```

HTTP コード対応: NotFound→404, Unauthorized→401, Forbidden→403,
InvalidRequest→400, Conflict→409, Unprocessable→422, Database/Internal→500

### AuthenticatedUser（`src/auth.rs`）

```rust
pub struct AuthenticatedUser { pub id: Uuid, pub role: Role, pub is_active: bool }
// Axum extractor: Authorization: Bearer {employee_code} → DB lookup
// 見つからない / is_active=false → AppError::Unauthorized
```

### PaginationParams（`src/pagination.rs`）

```rust
pub struct PaginationParams { pub page: u32, pub per_page: u32 }  // max per_page: 100
pub struct PaginatedResponse<T> { pub data: Vec<T>, pub meta: PaginationMeta }
pub struct PaginationMeta { pub total: i64, pub page: u32, pub per_page: u32 }
```

---

## 実装順序（PR 単位、TDD）

各 PR は `git-branch` → RED コミット → GREEN コミット → REFACTOR コミット → PR 作成 → Copilot レビュー → マージ の順で進める。

### PR-0: プロジェクト骨格 + ヘルスチェック

- Cargo.toml 依存追加
- `src/lib.rs`, `src/error.rs`, `src/state.rs`, `src/auth.rs`, `src/pagination.rs`, `src/routes/mod.rs`
- `GET /health` → `200 OK`
- テスト: ヘルスチェックが 200 を返す

### PR-1: DB マイグレーション + 認証スタブ

- 全 13 マイグレーションファイル作成（`sqlx migrate run` で適用確認）
- `AuthenticatedUser` extractor 実装
- テスト: 有効トークン→200、無効→401

### PR-2: departments CRUD

- `GET /departments`（ツリー構造。include_inactive クエリ対応）
- `POST /departments`, `GET /departments/:id`, `PUT /departments/:id`
- 権限: POST/PUT は admin のみ
- 注意: `effective_to IS NULL` が現在有効部署。ツリーは Rust 側で組み立て

### PR-3: employees CRUD

- `GET /employees`（department_id, is_active フィルタ、ページネーション）
- `POST /employees`, `GET /employees/:id`, `PUT /employees/:id`
- POST 時に `employee_departments` に初期所属レコードも挿入
- 退職処理: `is_active: false` + `employee_departments.effective_to` 設定

### PR-4: disciplines CRUD

- `GET/POST /disciplines`, `GET/PUT /disciplines/:id`
- `code` フィールドは変更不可 → 422

### PR-5: document_kinds & document_registers CRUD

- `GET/POST /document-kinds`, `PUT /document-kinds/:id`（`code` 不変）
- `GET/POST /document-registers`, `PUT /document-registers/:id`（`register_code` 不変）
- クエリフィルタ: `doc_kind_id`, `department_id`

### PR-6: projects CRUD

- `GET/POST /projects`, `GET/PUT/DELETE /projects/:id`
- DELETE: 紐づく文書存在時 → 409
- project_manager の PUT は `projects.manager_id == user.id` のみ
- `GET /projects/:id/documents` はスタブ（PR-9 で実装）

### PR-7: tags CRUD

- `GET/POST /tags`（ページネーション対応）

### PR-8: 文書採番サービス（`services/document_numbering.rs`）

採番フォーマット: `{文書種別コード}{部署コード}-{YYMM}{連番(seq_digits桁)}`
例: `内設計-2603001`

```rust
pub async fn assign_doc_number(
    tx: &mut PgConnection,
    doc_kind_code: &str, dept_code: &str,
    seq_digits: i32, registered_at_jst: NaiveDateTime,
) -> Result<String, AppError>
```

- `SELECT doc_number FROM documents WHERE doc_number LIKE $1 ORDER BY doc_number DESC LIMIT 1 FOR UPDATE`
- 連番 +1 してゼロ埋め
- `UNIQUE(doc_number)` 違反時は最大 3 回リトライ
- テスト: 10 並列同時実行で重複なし

### PR-9: documents CRUD

- `POST /documents`: 採番（PR-8）→ `status=draft`, `revision=1`
- `GET /documents`: 全フィルタ（project_id, discipline_id, doc_kind_id, confidentiality, status, author_id, tag, q）、ページネーション
- `GET /documents/:id`
- `PUT /documents/:id`:
  - `doc_number`, `frozen_dept_code`, `status` は変更不可 → 422
  - title/file_path/confidentiality/tags 変更時に `revision` 自動 +1（`FOR UPDATE`）
  - tags は差分更新（DELETE + INSERT）
- `DELETE /documents/:id`: approval_steps or circulations 存在時 → 409
- `GET /projects/:id/documents`: PR-6 のスタブを実装

revision 変更条件: `draft` or `rejected` 状態のみ

### PR-10: 承認フロー（`services/approval.rs`）

- `GET /documents/:id/approval-steps`: 全 route_revision のステップ一覧
- `POST /documents/:id/approval-steps`:
  - `draft` or `rejected` 状態のみ可
  - `route_revision = MAX + 1`（自動）、`document_revision = 現在 revision`（自動）
  - 文書 `status → under_review`
- `POST .../approval-steps/:step_id/approve`:
  - 最新 route_revision のアクティブステップ（最小 step_order の pending）でなければ 422
  - 呼び出し者が `approver_id` でなければ 403
  - 全ステップ承認完了 → 文書 `status → approved`
- `POST .../approval-steps/:step_id/reject`:
  - 同 route_revision の残 pending ステップを全て rejected へ
  - 文書 `status → rejected`

アクティブステップの特定:

```sql
SELECT * FROM approval_steps
WHERE document_id = $1
  AND route_revision = (SELECT MAX(route_revision) FROM approval_steps WHERE document_id = $1)
  AND status = 'pending'
ORDER BY step_order LIMIT 1
```

### PR-11: 回覧（`services/circulation.rs`）

- `GET /documents/:id/circulations`
- `POST /documents/:id/circulations`:
  - `approved` 状態のみ可
  - 文書 `status → circulating`
- `POST /documents/:id/circulations/confirm`:
  - 呼び出し者の `recipient_id` レコードの `confirmed_at = now()`
  - 全宛先確認済み → 文書 `status → completed`

### PR-12: 権限マトリクス網羅テスト

全エンドポイントの権限テストが漏れなく実装されているか確認し、不足を補完。

---

## マイグレーション設計（主要テーブルのみ抜粋）

**departments**: `UNIQUE(code)`, `effective_to IS NULL` が現在有効
**employees**: `CHECK(role IN ('admin','project_manager','general','viewer'))`
**documents**: `UNIQUE(doc_number)`, `CHECK(status IN (...))`, `CHECK(revision >= 1)`
**approval_steps**: `UNIQUE(document_id, route_revision, step_order)`
**circulations**: `UNIQUE(document_id, recipient_id)`

全テーブルに `created_at TIMESTAMPTZ NOT NULL DEFAULT now()` を付与。
更新するテーブルには `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()` を付与。

---

## テスト戦略

```rust
// 基本パターン
#[sqlx::test(migrations = "migrations")]
async fn test_post_document_assigns_doc_number(pool: PgPool) {
    let dept = insert_test_department(&pool, "設計").await;
    // ... マスタデータ準備 ...
    let client = TestClient::new(build_test_app(pool));

    let resp = client.post("/api/v1/documents")
        .bearer_auth("E001")   // スタブ: employee_code をトークンとして使用
        .json(&json!({ ... })).send().await;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = resp.json().await;
    assert!(body["doc_number"].as_str().unwrap().starts_with("内設計-"));
}
```

- `#[sqlx::test]`: テストごとに独立 DB（並列実行安全）
- `tests/helpers/mod.rs`: `insert_test_*` / `build_test_app` を共有
- CI: GitHub Actions + PostgreSQL サービスコンテナ、`SQLX_OFFLINE=true`
- オフラインキャッシュ: `cargo sqlx prepare` で `sqlx-data.json` 生成

---

## 検証方法

1. **ユニット/統合テスト**: `cargo test` で全テスト通過を確認
2. **マイグレーション**: `cargo sqlx migrate run` でエラーなく適用できること
3. **エンドツーエンド**: `cargo run` 後、`curl` で各エンドポイントを手動確認
4. **採番同時実行**: 並列テストで重複 doc_number が発生しないこと
5. **承認フロー**: draft→under_review→approved→circulating→completed の状態遷移を統合テストで確認
6. **権限マトリクス**: 各ロールで許可/拒否が正しく返ることをテストで確認
