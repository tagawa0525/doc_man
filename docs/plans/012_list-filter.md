# 文書・プロジェクト一覧の個別フィルタ機能

## Context

テキスト検索（`q` パラメータ）は実装済みだが、数万件の文書から目的のものを探すには構造化されたフィルタが不可欠。ログインユーザーの所属部署（兼務含む）をデフォルト選択とし、部署・文書種別・年度の選択式フィルタと、プロジェクト名・作成者・WBSコードの部分一致フィルタを追加する。

**制約**: DB固有の機能は使わない。標準SQLのみ。

## フィルタ仕様

### 文書一覧

| フィルタ           | 種別     | パラメータ                  | デフォルト                     |
| ------------------ | -------- | --------------------------- | ------------------------------ |
| 部署               | 複数選択 | `dept_codes` (カンマ区切り) | ユーザーの所属部署（兼務含む） |
| 文書種別           | 単一選択 | `doc_kind_id`               | 全て                           |
| 年度               | 単一選択 | `fiscal_year`               | 当年度                         |
| タイトル・文書番号 | 部分一致 | `q`（既存）                 | 空                             |
| プロジェクト名     | 部分一致 | `project_name`              | 空                             |
| 作成者             | 部分一致 | `author_name`               | 空                             |
| WBSコード          | 部分一致 | `wbs_code`                  | 空                             |

- `dept_codes`: `frozen_dept_code` で絞り込み。sqlx `QueryBuilder` で動的に `IN ($1, $2, ...)` を構築（標準SQL）
- `fiscal_year`: 年度 → `created_at` が `{year}-04-01` 〜 `{year+1}-03-31` の範囲

### プロジェクト一覧

| フィルタ       | 種別     | パラメータ                | デフォルト         |
| -------------- | -------- | ------------------------- | ------------------ |
| 部署           | 複数選択 | `dept_ids` (カンマ区切り) | ユーザーの所属部署 |
| 年度           | 単一選択 | `fiscal_year`             | 当年度             |
| プロジェクト名 | 部分一致 | `q`（既存）               | 空                 |
| 担当者         | 部分一致 | `manager_name`            | 空                 |

- `dept_ids`: discipline → department 経由で絞り込み
- 既存の `status`, `discipline_id`, `wbs_code` パラメータは維持（フロントUIからは使わない）

## 変更ファイル

### Phase 1: `/api/v1/me` の拡張

**目的**: ログインユーザーの所属部署情報をフロントエンドに渡す

**`server/src/routes/mod.rs`** — `MeResponse` に `departments` を追加

```rust
struct MeDepartment {
    id: Uuid,
    code: String,
    name: String,
}

struct MeResponse {
    id: Uuid,
    role: serde_json::Value,
    departments: Vec<MeDepartment>,
}
```

`me()` ハンドラに `State(state)` を追加し、`employee_departments` を JOIN して取得:

```sql
SELECT d.id, d.code, d.name
FROM employee_departments ed
JOIN departments d ON d.id = ed.department_id
WHERE ed.employee_id = $1 AND ed.effective_to IS NULL
```

**`frontend/src/api/types.rs`** — `MeResponse` に `departments` を追加

```rust
pub struct MeDepartment {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

pub struct MeResponse {
    pub id: Uuid,
    pub role: String,
    pub departments: Vec<MeDepartment>,
}
```

**`frontend/src/auth.rs`** — `UserInfo` に `departments` を追加

```rust
pub struct UserInfo {
    pub id: Uuid,
    pub role: Role,
    pub departments: Vec<MeDepartment>,
}
```

`AuthContext::login()` と `verify_token()` の結果から `departments` を渡す。

**`frontend/src/pages/login.rs`** — `departments` の受け渡しを追加

### Phase 2: 文書一覧バックエンドフィルタ

**`server/src/handlers/documents.rs`**

`DocumentListQuery` を拡張:

```rust
pub struct DocumentListQuery {
    pub project_id: Option<Uuid>,
    pub q: Option<String>,
    pub dept_codes: Option<String>,    // カンマ区切り
    pub doc_kind_id: Option<Uuid>,
    pub fiscal_year: Option<i32>,
    pub project_name: Option<String>,
    pub author_name: Option<String>,
    pub wbs_code: Option<String>,
    pub pagination: PaginationParams,
}
```

固定パラメータの SQL WHERE 句:

```sql
AND ($N::uuid IS NULL OR d.doc_kind_id = $N)
AND ($N::date IS NULL OR d.created_at >= $N AND d.created_at < $N)
AND ($N::text IS NULL OR LOWER(p.name) LIKE '%' || $N || '%' ESCAPE '\')
AND ($N::text IS NULL OR LOWER(e.name) LIKE '%' || $N || '%' ESCAPE '\')
AND ($N::text IS NULL OR LOWER(p.wbs_code) LIKE '%' || $N || '%' ESCAPE '\')
```

**動的部分 (dept_codes)**: sqlx `QueryBuilder` で SQL 全体を動的構築する。`dept_codes` が指定された場合は `AND d.frozen_dept_code IN (?, ?, ...)` を動的に追加。指定なしの場合はこの条件を省略。

**年度→日付変換**: Rust 側で `fiscal_year` を日付範囲に変換してバインド（DB関数は使わない）:

```rust
let fiscal_start = NaiveDate::from_ymd_opt(year, 4, 1).unwrap();
let fiscal_end = NaiveDate::from_ymd_opt(year + 1, 4, 1).unwrap();
```

### Phase 3: プロジェクト一覧バックエンドフィルタ

**`server/src/handlers/projects.rs`**

`ProjectListQuery` を拡張:

```rust
pub struct ProjectListQuery {
    pub status: Option<String>,
    pub discipline_id: Option<Uuid>,
    pub wbs_code: Option<String>,
    pub q: Option<String>,
    pub dept_ids: Option<String>,      // カンマ区切り UUID
    pub fiscal_year: Option<i32>,
    pub manager_name: Option<String>,
    pub pagination: PaginationParams,
}
```

文書と同様に `QueryBuilder` で SQL 全体を動的構築。`dept_ids` が指定された場合は `AND d.id IN (?, ?, ...)` を動的追加（discipline → department の JOIN 済み）。年度・担当者名も同じパターン。

### Phase 4: フロントエンド文書フィルタUI

**`frontend/src/pages/documents/list.rs`**

テーブルの上にフィルタバーを配置:

```text
[部署: ☑設計 ☑製造 ☐品質] [種別: ▼全て] [年度: ▼2025] [タイトル検索: ___]
[PJ名: ___] [作成者: ___] [WBS: ___]
```

- 選択式フィルタは `RwSignal` で管理。変更時に即座にリソースを再取得
- テキストフィルタは 300ms debounce（既存パターン流用）
- 部署チェックボックスは `AuthContext` の `departments` からデフォルト選択
- 年度ドロップダウンは当年度 ± 数年を静的生成。デフォルト = 当年度
- 文書種別は `api::document_kinds::list_all()` で取得

### Phase 5: フロントエンドプロジェクトフィルタUI

**`frontend/src/pages/projects/list.rs`**

同様のフィルタバー（部署・年度・プロジェクト名・担当者）

## コミット順序（TDD）

1. `test: me endpoint returns user departments` (RED)
2. `feat: extend me endpoint with department info` (GREEN)
3. `feat: extend frontend auth context with departments`
4. `test: document list filters by dept_codes, doc_kind_id, fiscal_year, project_name, author_name, wbs_code` (RED)
5. `feat: add structured filter parameters to document list endpoint` (GREEN)
6. `test: project list filters by dept_ids, fiscal_year, manager_name` (RED)
7. `feat: add structured filter parameters to project list endpoint` (GREEN)
8. `feat: add filter controls to document list page`
9. `feat: add filter controls to project list page`

## 検証方法

1. `cargo test` — 全統合テストがパス
2. `just lint` — clippy がパス
3. 手動確認:
   - ログイン後、文書一覧で自部署の文書のみ表示されること
   - 部署チェックを外すと対象が広がること
   - 年度・文書種別を変更するとリストが更新されること
   - テキストフィルタ入力で部分一致絞り込みが動作すること
   - プロジェクト一覧でも同様に動作すること
