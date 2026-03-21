# 職位(Position)追加 + 3-tier ロール解決 + 権限昇格UX

## Context

現在、従業員には `role` カラム（admin/project_manager/general/viewer）が直接設定されている。
これを **職位（社長・部長・課長など）** をマスタ管理し、職位からデフォルトRoleを導出する仕組みに変更する。
さらに、個人・部署単位での Role 上書きを可能にし（3-tier ロール解決）、
フロントエンドでは「ページ単位の権限昇格」UX を実装して誤操作を防止する。

### 認可モデル（二軸認可）

現在の認可はグローバルなロールチェックのみ。たとえば project_manager であれば全部署のプロジェクトを変更できてしまう。

本計画では **二軸認可** を導入する:

1. **ロール（capability）**: 3-tier で決定。「何ができるか」を表す
   - admin: 全操作。部署スコープをバイパス
   - project_manager: プロジェクト管理 + 文書作成/編集
   - general: 文書作成/編集
   - viewer: 閲覧のみ
2. **部署スコープ（scope）**: 「どのリソースに対してできるか」を表す
   - 非 admin ユーザーは、自分が所属する部署のリソースのみ書き込み可能
   - リソースの所属部署はリソースチェインで解決:
     - Project → discipline → department
     - Document → project → discipline → department
   - 読み取りは全部署に対して可能（業務上、他部署の文書を閲覧できる必要がある）

admin のみのマスタ管理エンドポイント（departments, disciplines, document_kinds 等）は
ロールチェックだけで十分であり、部署スコープの対象外とする。

## PR 構成

### PR 1: positions マスタテーブル + CRUD

### PR 2: department_role_grants + employees.position_id + 3-tier ロール解決

### PR 3: フロントエンド（職位表示 + 権限昇格UX）

---

## PR 1: positions マスタテーブル + CRUD

### マイグレーション

`server/migrations/20260321000019_create_positions.sql`

```sql
CREATE TABLE positions (
    id           UUID         NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name         VARCHAR(100) NOT NULL UNIQUE,
    default_role VARCHAR(20)  NOT NULL DEFAULT 'viewer'
                 CHECK (default_role IN ('admin', 'project_manager', 'general', 'viewer')),
    sort_order   INT          NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_positions_sort_order ON positions(sort_order);

-- 初期データ（PR 2 のバックフィルに必要なためマイグレーションに含める）
INSERT INTO positions (name, default_role, sort_order) VALUES
    ('社長',   'admin',           1),
    ('部長',   'admin',           2),
    ('課長',   'admin',           3),
    ('総合職', 'project_manager', 4),
    ('一般職', 'general',         5),
    ('嘱託',   'viewer',          6),
    ('派遣',   'viewer',          7);
```

### バックエンド

**モデル**: `server/src/models/position.rs`（新規）

- `PositionRow { id, name, default_role, sort_order }`
- `PositionResponse { id, name, default_role, sort_order }`
- `CreatePositionRequest { name, default_role, sort_order }`
- `UpdatePositionRequest { name: Option, default_role: Option, sort_order: Option }`
- `mod.rs` に登録

**ハンドラ**: `server/src/handlers/positions.rs`（新規）

- `GET /api/v1/positions` — 認証済みユーザー。sort_order 順。ページネーション不要（小マスタ）
- `POST /api/v1/positions` — admin のみ
- `GET /api/v1/positions/{id}` — 認証済みユーザー
- `PUT /api/v1/positions/{id}` — admin のみ
- `handlers/mod.rs`, `routes/mod.rs` に登録

**テスト**（TDD: RED → GREEN → REFACTOR）:

- `server/tests/positions.rs`（新規）: tags.rs パターンに準拠した CRUD テスト
- `server/tests/permissions.rs` に positions エンドポイントの権限テスト追加
- `server/tests/helpers/mod.rs` に `insert_position()` ヘルパー追加

### フロントエンド

- `frontend/src/api/positions.rs`（新規）: list, create, get, update
- `frontend/src/api/types.rs`: `PositionResponse`, `CreatePositionRequest`, `UpdatePositionRequest` 追加
- `frontend/src/pages/positions/`（新規）: admin 専用 CRUD ページ（disciplines パターン準拠）
- `frontend/src/main.rs`: `/positions` ルート追加
- `frontend/src/components/layout.rs`: ナビに「職位」追加（admin_only: true）

### シードデータ

`server/scripts/seed.sql` に positions INSERT を追加（マイグレーションと同じデータ、冪等に `ON CONFLICT DO NOTHING`）

---

## PR 2: department_role_grants + employees.position_id + 3-tier ロール解決

### マイグレーション

`server/migrations/20260321000020_add_position_and_role_grants.sql`

```sql
-- 1. department_role_grants テーブル
CREATE TABLE department_role_grants (
    id            UUID         NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    department_id UUID         NOT NULL UNIQUE REFERENCES departments(id),
    role          VARCHAR(20)  NOT NULL
                  CHECK (role IN ('admin', 'project_manager', 'general', 'viewer')),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 2. employees に position_id 追加
ALTER TABLE employees ADD COLUMN position_id UUID REFERENCES positions(id);

-- 3. 既存データのバックフィル（現在の role から職位を推定）
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '課長')   WHERE role = 'admin';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '総合職') WHERE role = 'project_manager';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '一般職') WHERE role = 'general';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '嘱託')   WHERE role = 'viewer';

-- 4. NOT NULL 制約追加
ALTER TABLE employees ALTER COLUMN position_id SET NOT NULL;

-- 5. role を nullable に変更（NULL = 上書きなし、職位/部署のデフォルトを使用）
ALTER TABLE employees ALTER COLUMN role DROP NOT NULL;
ALTER TABLE employees ALTER COLUMN role DROP DEFAULT;
```

### auth.rs の変更（最重要）

`server/src/auth.rs` の認証クエリを 3-tier ロール解決に変更:

```sql
SELECT
    e.id,
    e.name,
    e.is_active,
    COALESCE(
        e.role,                    -- Tier 1: 個人上書き
        drg.role,                  -- Tier 2: 部署付与
        p.default_role             -- Tier 3: 職位デフォルト
    ) AS effective_role,
    ARRAY(
        SELECT ed2.department_id
        FROM employee_departments ed2
        WHERE ed2.employee_id = e.id AND ed2.effective_to IS NULL
    ) AS department_ids
FROM employees e
JOIN positions p ON p.id = e.position_id
LEFT JOIN employee_departments ed
    ON ed.employee_id = e.id
    AND ed.effective_to IS NULL
    AND ed.is_primary = true
LEFT JOIN department_role_grants drg
    ON drg.department_id = ed.department_id
WHERE e.employee_code = $1
```

`AuthenticatedUser` 構造体を拡張:

```rust
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub name: String,
    pub role: Role,                   // effective_role
    pub is_active: bool,
    pub department_ids: Vec<Uuid>,    // 全アクティブ所属部署
}

impl AuthenticatedUser {
    /// admin はバイパス、それ以外は所属部署に含まれるかチェック
    pub fn can_access_department(&self, department_id: Uuid) -> bool {
        self.role == Role::Admin || self.department_ids.contains(&department_id)
    }
}
```

ハンドラ側の既存ロールチェック（`user.role != Role::Admin` 等）は変更不要。
部署スコープが必要なハンドラにのみ `can_access_department` チェックを追加する。

### 部署スコープ認可（新規）

`server/src/authorization.rs`（新規）にリソース→部署 解決ヘルパーを配置:

```rust
/// discipline_id → department_id
pub async fn get_discipline_department_id(pool: &PgPool, discipline_id: Uuid) -> Result<Uuid, AppError>

/// project_id → discipline → department_id
pub async fn get_project_department_id(pool: &PgPool, project_id: Uuid) -> Result<Uuid, AppError>

/// document_id → project → discipline → department_id
pub async fn get_document_department_id(pool: &PgPool, document_id: Uuid) -> Result<Uuid, AppError>
```

**部署スコープチェックが必要なエンドポイント**:

| エンドポイント                     | 解決チェイン                | 現在の権限                   |
| ---------------------------------- | --------------------------- | ---------------------------- |
| POST/PUT projects                  | discipline → dept           | admin or PM                  |
| POST/PUT/DELETE documents          | project → discipline → dept | non-viewer (DELETE は admin) |
| POST documents/{id}/revise         | project → discipline → dept | non-viewer                   |
| POST documents/{id}/approval-steps | project → discipline → dept | admin or PM                  |
| POST documents/{id}/distributions  | project → discipline → dept | non-viewer                   |

**不要なエンドポイント**（admin のみ or 全リソース共通）:

- departments, employees, disciplines, document_kinds, document_registers, positions の CRUD（admin のみ）
- tags の POST（グローバルリソース）
- approval-steps の approve/reject（承認者本人チェックがスコープを兼ねる）
- GET 系（閲覧は全部署に対して許可）

### /api/v1/me の変更

`MeResponse` にフィールド追加:

- `position_name: String` — 表示用
- `role_override: Option<String>` — 個人上書きがある場合のみ

`role` フィールドは引き続き effective_role を返す（後方互換性）。

### department_role_grants API

- `GET /api/v1/department-role-grants` — admin のみ。全件取得
- `PUT /api/v1/departments/{id}/role-grant` — admin のみ。付与/更新
- `DELETE /api/v1/departments/{id}/role-grant` — admin のみ。削除

ハンドラ: `server/src/handlers/department_role_grants.rs`（新規）

### employees モデル/ハンドラ変更

**モデル** (`server/src/models/employee.rs`):

- `EmployeeRow` に `position_id: Uuid`, `position_name: Option<String>` 追加
- `EmployeeResponse` に `position: NameBrief` 追加。`role` は effective_role を返す
- `CreateEmployeeRequest` に `position_id: Uuid` 追加。`role` は上書き用（Optional のまま）
- `UpdateEmployeeRequest`: `role` の扱いを `Option<Option<String>>` に変更
  - JSON フィールド未指定 → `None`（変更なし）
  - `"role": null` → `Some(None)`（上書きクリア）
  - `"role": "admin"` → `Some(Some("admin"))`（上書き設定）

**ハンドラ** (`server/src/handlers/employees.rs`):

- SQL クエリを positions JOIN に更新
- create: `position_id` を INSERT に追加
- update: `position_id` の更新対応。`role` の nullable 対応

### テストヘルパーの変更（全テストに影響）

`server/tests/helpers/mod.rs`:

- `insert_default_positions(pool)` ヘルパー追加 → デフォルト職位を INSERT して一般職の ID を返す
- `insert_employee(pool, code, role)` を修正: 内部で `insert_default_positions` を呼び、`position_id` を設定
  - 既存の `role` 引数は引き続き個人上書きとして設定（後方互換性）
- `insert_department_role_grant(pool, department_id, role)` ヘルパー追加

**テスト**:

- `server/tests/auth.rs` に 3-tier テスト追加:
  - 個人上書きあり → 上書き値を使用
  - 上書きなし + 部署付与あり → 部署付与を使用
  - 上書きなし + 部署付与なし → 職位デフォルトを使用
  - プライマリ部署なし → 職位デフォルトにフォールバック
- `server/tests/department_role_grants.rs`（新規）
- `server/tests/employees.rs` に position_id 関連テスト追加
- `server/tests/department_scope.rs`（新規）— 部署スコープ認可テスト:
  - PM が自部署のプロジェクトを作成/更新 → 成功
  - PM が他部署のプロジェクトを作成/更新 → 403
  - admin が他部署のプロジェクトを操作 → 成功（バイパス）
  - general が自部署の文書を作成 → 成功
  - general が他部署の文書を作成 → 403
  - 複数部署所属ユーザーが各所属部署のリソースを操作 → 成功

### シードデータ

`server/scripts/seed.sql`:

- employees INSERT に `position_id` 追加（各従業員に適切な職位を割当）
- department_role_grants のサンプル追加（例: 管理部にadmin付与）

---

## PR 3: フロントエンド（職位表示 + 権限昇格UX）

### AuthContext 変更

`frontend/src/auth.rs`:

```rust
pub struct AuthContext {
    pub user: RwSignal<Option<UserInfo>>,
    pub loading: RwSignal<bool>,
    pub escalated: RwSignal<bool>,  // 新規: 権限昇格状態
}
```

新メソッド:

- `effective_role(&self) -> Option<Role>` — ユーザーの実際の effective role
- `display_role(&self) -> Option<Role>` — 昇格中なら effective_role、そうでなければ Viewer
- `escalate(&self)` — `escalated = true`
- `de_escalate(&self)` — `escalated = false`
- `needs_escalation(&self) -> bool` — effective_role が Viewer より上なら true

`UserInfo` に `position_name: String` 追加。

### 権限昇格コンポーネント

`frontend/src/components/escalation.rs`（新規）:

既存の `ConfirmModal` を利用したラッパーコンポーネント:

- ページマウント時に `escalated` が false なら確認モーダルを表示
- メッセージ: 「この操作には {effective_role} 権限が必要です。権限を昇格しますか？」
- 確認 → `auth.escalate()` して内容を表示
- キャンセル → 前のページに戻る（`window.history.back()`）

### ページ適用

権限昇格が必要なページ（編集/作成系）にラッパーを適用:

- `/employees/new`, `/employees/{id}` （従業員作成/編集）
- `/documents/new` （文書作成）
- `/projects/new`, `/projects/{id}` （プロジェクト作成/編集）
- inline CRUD ページ: departments, disciplines, document-kinds, document-registers, positions

  → 作成/編集ボタン押下時にモーダル表示

### ルート変更リスナーによる自動解除

`frontend/src/main.rs` または `layout.rs`:

- ルートの pathname signal を watch し、変更時に `auth.de_escalate()` を呼ぶ

### レイアウト変更

`frontend/src/components/layout.rs`:

- サイドバーのナビ項目は **effective_role** に基づいて表示（昇格前でも見える）
- ロールタグ:
  - 通常時: 「閲覧者」（青タグ）
  - 昇格時: 「{effective_role} (昇格中)」（黄色/オレンジタグ）
- 手動で降格するボタン（昇格中のみ表示）

### 従業員フォーム変更

`frontend/src/pages/employees/form.rs`:

- 職位ドロップダウン追加（positions API からロード、必須）
- Role ドロップダウンのラベルを「ロール上書き」に変更
- 「自動（職位から決定）」オプション追加（`role: null` を送信）

### 従業員一覧変更

`frontend/src/pages/employees/list.rs`:

- 職位カラム追加

### API types 変更

`frontend/src/api/types.rs`:

- `EmployeeResponse` に `position: NameBrief` 追加
- `MeResponse` に `position_name`, `role_override` 追加
- `CreateEmployeeRequest` に `position_id` 追加

---

## 変更対象ファイル一覧

### 新規ファイル

- `server/migrations/20260321000019_create_positions.sql`
- `server/migrations/20260321000020_add_position_and_role_grants.sql`
- `server/src/models/position.rs`
- `server/src/handlers/positions.rs`
- `server/src/handlers/department_role_grants.rs`
- `server/src/authorization.rs` — 部署スコープ認可ヘルパー
- `server/tests/positions.rs`
- `server/tests/department_role_grants.rs`
- `server/tests/department_scope.rs` — 部署スコープ認可テスト
- `frontend/src/api/positions.rs`
- `frontend/src/pages/positions/mod.rs`
- `frontend/src/pages/positions/list.rs`
- `frontend/src/components/escalation.rs`

### 変更ファイル

- `server/src/auth.rs` — 3-tier ロール解決クエリ、`AuthenticatedUser` に `department_ids` 追加
- `server/src/lib.rs` — authorization モジュール追加
- `server/src/models/mod.rs` — position モジュール追加
- `server/src/models/employee.rs` — position_id, effective_role 対応
- `server/src/handlers/mod.rs` — positions, department_role_grants モジュール追加
- `server/src/handlers/employees.rs` — SQL クエリ更新、position_id 対応
- `server/src/handlers/projects.rs` — 部署スコープチェック追加
- `server/src/handlers/documents.rs` — 部署スコープチェック追加
- `server/src/handlers/approval_steps.rs` — 部署スコープチェック追加（create_approval_route）
- `server/src/handlers/distributions.rs` — 部署スコープチェック追加
- `server/src/routes/mod.rs` — 新エンドポイント登録、MeResponse 拡張
- `server/tests/helpers/mod.rs` — insert_default_positions, insert_employee 修正
- `server/tests/employees.rs` — position_id テスト追加
- `server/tests/auth.rs` — 3-tier テスト追加
- `server/tests/permissions.rs` — positions 権限テスト追加
- `server/scripts/seed.sql` — positions, position_id, department_role_grants 追加
- `frontend/src/auth.rs` — escalated signal, display_role, UserInfo 変更
- `frontend/src/api/types.rs` — DTO 更新
- `frontend/src/api/mod.rs` — positions モジュール追加
- `frontend/src/pages/mod.rs` — positions ページ追加
- `frontend/src/pages/employees/form.rs` — 職位ドロップダウン追加
- `frontend/src/pages/employees/list.rs` — 職位カラム追加
- `frontend/src/components/mod.rs` — escalation モジュール追加
- `frontend/src/components/layout.rs` — ナビ項目追加、昇格状態表示
- `frontend/src/main.rs` — ルート追加、ルート変更時の降格処理

## 検証方法

1. **ユニットテスト**: `cargo test` で全テスト通過を確認
2. **3-tier ロール解決**: auth テストで各 tier の優先順位を検証
3. **権限テスト**: permissions テストで全エンドポイントの RBAC を検証
4. **部署スコープテスト**: department_scope テストで以下を検証
   - 自部署のリソースに対する書き込み → 成功
   - 他部署のリソースに対する書き込み → 403
   - admin による他部署リソースの操作 → 成功（バイパス）
   - 複数部署所属ユーザーの各部署リソースへのアクセス → 成功
5. **E2E 手動確認**:
   - `just run` + `just frontend-dev` で起動
   - 各職位の従業員でログインし、effective_role が正しいことを確認
   - 部署付与・個人上書きを設定し、ロール解決の優先順位を確認
   - **他部署のプロジェクト/文書を編集しようとして 403 が返ることを確認**
   - 編集ページ遷移時に権限昇格モーダルが表示されることを確認
   - ページ離脱時に自動降格されることを確認
6. **Lint**: `just lint` + `just fmt-check` 通過
