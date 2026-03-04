# API 設計

## 共通仕様

### ベース URL

```text
/api/v1
```

### 認証

全エンドポイントは認証必須（Bearer トークン）。ロールによるアクセス制御は各エンドポイントに記載する。

### Content-Type

- リクエスト: `application/json`
- レスポンス: `application/json`

### 日付・日時フォーマット

- 日付: `YYYY-MM-DD`
- 日時: ISO 8601（`2026-02-15T10:30:00Z`）

### ページネーション

一覧系エンドポイントはクエリパラメータでページネーションを指定する。

| パラメータ | 型      | デフォルト | 説明                             |
| ---------- | ------- | ---------- | -------------------------------- |
| `page`     | integer | 1          | ページ番号（1 始まり）           |
| `per_page` | integer | 20         | 1 ページあたりの件数（最大 100） |

レスポンスには以下のメタ情報を含む:

```json
{
  "data": [...],
  "meta": {
    "total": 150,
    "page": 1,
    "per_page": 20
  }
}
```

### エラーレスポンス

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Document not found"
  }
}
```

| HTTP ステータス | code              | 説明                                               |
| --------------- | ----------------- | -------------------------------------------------- |
| 400             | `INVALID_REQUEST` | リクエストパラメータ不正                           |
| 401             | `UNAUTHORIZED`    | 未認証                                             |
| 403             | `FORBIDDEN`       | 権限不足                                           |
| 404             | `NOT_FOUND`       | リソースが存在しない                               |
| 409             | `CONFLICT`        | 採番重複など                                       |
| 422             | `UNPROCESSABLE`   | ビジネスルール違反（凍結フィールドの変更試行など） |
| 500             | `INTERNAL_ERROR`  | サーバー内部エラー                                 |

---

## 部署 /departments

### GET /departments

部署ツリーを返す。

**クエリパラメータ**:

- `include_inactive=true` -廃止部署を含める（デフォルト: false）

**レスポンス 200**:

```json
[
  {
    "id": "uuid",
    "code": "001",
    "name": "技術部",
    "parent_id": null,
    "effective_from": "2020-04-01",
    "effective_to": null,
    "children": [
      {
        "id": "uuid",
        "code": "002",
        "name": "機械設計課",
        "parent_id": "uuid",
        "effective_from": "2020-04-01",
        "effective_to": null,
        "children": []
      }
    ]
  }
]
```

### POST /departments

**必要ロール**: `admin`

**リクエスト**:

```json
{
  "code": "003",
  "name": "電気工事課",
  "parent_id": "uuid",
  "effective_from": "2026-04-01"
}
```

**レスポンス 201**:

```json
{
  "id": "uuid",
  "code": "003",
  "name": "電気工事課",
  "parent_id": "uuid",
  "effective_from": "2026-04-01",
  "effective_to": null
}
```

### GET /departments/:id

**レスポンス 200**: POST と同じ構造（`children` なし）

### PUT /departments/:id

**必要ロール**: `admin`

**リクエスト**（変更したいフィールドのみ）:

```json
{
  "name": "電気設備課",
  "effective_to": "2026-03-31",
  "merged_into_id": "uuid"
}
```

---

## 社員 /employees

### GET /employees

**クエリパラメータ**:

- `department_id` -所属部署フィルタ（現在所属）
- `is_active=true|false` -在籍フィルタ（デフォルト: true）

**レスポンス 200**:

```json
{
  "data": [
    {
      "id": "uuid",
      "name": "山田 太郎",
      "employee_code": "E001",
      "ad_account": "yamada.taro",
      "role": "general",
      "is_active": true,
      "current_department": {
        "id": "uuid",
        "name": "機械設計課"
      }
    }
  ],
  "meta": { "total": 42, "page": 1, "per_page": 20 }
}
```

### POST /employees

**必要ロール**: `admin`

**リクエスト**:

```json
{
  "name": "鈴木 花子",
  "employee_code": "E002",
  "ad_account": "suzuki.hanako",
  "role": "general",
  "department_id": "uuid",
  "effective_from": "2026-04-01"
}
```

### GET /employees/:id

### PUT /employees/:id

**必要ロール**: `admin`

退職処理: `is_active: false` をセット。`employee_departments` の現在所属レコードの `effective_to` を同時に更新する。

---

## 業務種別 /business-categories

### GET /business-categories

**クエリパラメータ**:

- `department_id` -担当部署フィルタ

**レスポンス 200**:

```json
{
  "data": [
    {
      "id": "uuid",
      "code": "CAD",
      "name": "CAD 設計",
      "department": {
        "id": "uuid",
        "code": "002",
        "name": "機械設計課"
      }
    }
  ],
  "meta": { "total": 4, "page": 1, "per_page": 20 }
}
```

### POST /business-categories

**必要ロール**: `admin`

**リクエスト**:

```json
{
  "code": "CAD",
  "name": "CAD 設計",
  "department_id": "uuid"
}
```

### GET /business-categories/:id

### PUT /business-categories/:id

**必要ロール**: `admin`

`code` の変更は不可（採番への影響を防ぐため）。

---

## プロジェクト /projects

### GET /projects

**クエリパラメータ**:

- `status` -ステータスフィルタ（`planning|active|completed|cancelled`）
- `business_category_id` -業務種別フィルタ
- `wbs_code` -WBS コード検索

**レスポンス 200**:

```json
{
  "data": [
    {
      "id": "uuid",
      "name": "〇〇設備更新工事",
      "status": "active",
      "start_date": "2026-01-10",
      "end_date": "2026-12-31",
      "wbs_code": "P-2026-001",
      "business_category": {
        "id": "uuid",
        "code": "MECH",
        "name": "機械補修",
        "department": {
          "id": "uuid",
          "name": "機械設計課"
        }
      },
      "manager": {
        "id": "uuid",
        "name": "田中 一郎"
      }
    }
  ],
  "meta": { "total": 10, "page": 1, "per_page": 20 }
}
```

### POST /projects

**必要ロール**: `admin`, `project_manager`

**リクエスト**:

```json
{
  "name": "〇〇設備更新工事",
  "status": "planning",
  "start_date": "2026-01-10",
  "end_date": "2026-12-31",
  "wbs_code": "P-2026-001",
  "business_category_id": "uuid",
  "manager_id": "uuid"
}
```

**レスポンス 201**: プロジェクト詳細オブジェクト

### GET /projects/:id

### PUT /projects/:id

**必要ロール**: `admin`, `project_manager`（担当プロジェクトのみ）

### DELETE /projects/:id

**必要ロール**: `admin`

紐づく文書が存在する場合は `409 CONFLICT`。

### GET /projects/:id/documents

プロジェクトに紐づく文書一覧。`GET /documents?project_id=` と同等。

---

## 文書 /documents

### GET /documents

**クエリパラメータ**:

- `project_id` -プロジェクトフィルタ
- `business_category_id` -業務種別フィルタ
- `confidentiality` -機密グレードフィルタ
- `author_id` -作成者フィルタ
- `tag` -タグ名フィルタ（複数指定可: `tag=CAD&tag=FEM`）
- `q` -タイトルキーワード検索

**レスポンス 200**:

```json
{
  "data": [
    {
      "id": "uuid",
      "doc_number": "CAD001-2602001",
      "title": "〇〇設備 外形図",
      "file_path": "/nas/projects/2026/drawing/cad001.dwg",
      "confidentiality": "internal",
      "author": { "id": "uuid", "name": "山田 太郎" },
      "business_category": { "id": "uuid", "code": "CAD", "name": "CAD 設計" },
      "project": { "id": "uuid", "name": "〇〇設備更新工事" },
      "tags": ["外形図", "設備"],
      "created_at": "2026-02-15T10:30:00Z",
      "updated_at": "2026-02-15T10:30:00Z"
    }
  ],
  "meta": { "total": 5, "page": 1, "per_page": 20 }
}
```

### POST /documents

文書番号を自動採番してレコードを作成する。採番仕様は `04_document_numbering.md` を参照。

**必要ロール**: `admin`, `project_manager`, `general`

**リクエスト**:

```json
{
  "title": "〇〇設備 外形図",
  "file_path": "/nas/projects/2026/drawing/cad001.dwg",
  "business_category_id": "uuid",
  "confidentiality": "internal",
  "project_id": "uuid",
  "tags": ["外形図", "設備"]
}
```

**レスポンス 201**: 採番済み `doc_number` を含む文書詳細オブジェクト

### GET /documents/:id

### PUT /documents/:id

`doc_number`, `frozen_dept_code`, `business_category_id` は変更不可（422 を返す）。

**リクエスト**（変更したいフィールドのみ）:

```json
{
  "title": "〇〇設備 外形図 Rev.1",
  "file_path": "/nas/projects/2026/drawing/cad001_r1.dwg",
  "confidentiality": "restricted",
  "project_id": "uuid",
  "tags": ["外形図", "設備", "Rev1"]
}
```

### DELETE /documents/:id

**必要ロール**: `admin`

承認ステップまたは回覧レコードが存在する場合は `409 CONFLICT`。

---

## 承認 /documents/:id/approval-steps

### GET /documents/:id/approval-steps

承認ルート一覧（全ステップ）を返す。

**レスポンス 200**:

```json
[
  {
    "id": "uuid",
    "step_order": 1,
    "approver": { "id": "uuid", "name": "佐藤 部長" },
    "status": "approved",
    "approved_at": "2026-02-16T09:00:00Z",
    "comment": "問題なし"
  },
  {
    "id": "uuid",
    "step_order": 2,
    "approver": { "id": "uuid", "name": "鈴木 役員" },
    "status": "pending",
    "approved_at": null,
    "comment": null
  }
]
```

### POST /documents/:id/approval-steps

承認ルートを設定する。文書が `draft` 状態のみ可能。既存のステップは全削除して置き換える。

**必要ロール**: `admin`, `project_manager`

**リクエスト**:

```json
{
  "steps": [
    { "step_order": 1, "approver_id": "uuid" },
    { "step_order": 2, "approver_id": "uuid" }
  ]
}
```

文書ステータスを `under_review` に変更する。

### POST /documents/:id/approval-steps/:step_order/approve

アクティブステップの承認者が承認する。

**リクエスト**:

```json
{
  "comment": "確認しました"
}
```

**レスポンス 200**: 更新後のステップオブジェクト

全ステップ承認完了時、文書ステータスを `approved` に変更する。

### POST /documents/:id/approval-steps/:step_order/reject

アクティブステップの承認者が差し戻す。

**リクエスト**:

```json
{
  "comment": "〇〇ページの寸法を修正してください"
}
```

文書ステータスを `rejected` に変更する。

---

## 回覧 /documents/:id/circulations

### GET /documents/:id/circulations

回覧宛先と確認状況の一覧を返す。

**レスポンス 200**:

```json
[
  {
    "id": "uuid",
    "recipient": { "id": "uuid", "name": "高橋 次郎" },
    "confirmed_at": "2026-02-17T14:00:00Z"
  },
  {
    "id": "uuid",
    "recipient": { "id": "uuid", "name": "渡辺 三郎" },
    "confirmed_at": null
  }
]
```

### POST /documents/:id/circulations

回覧を開始する。文書が `approved` 状態のみ可能。

**必要ロール**: `admin`, `project_manager`

**リクエスト**:

```json
{
  "recipient_ids": ["uuid", "uuid"]
}
```

文書ステータスを `circulating` に変更する。

### POST /documents/:id/circulations/confirm

呼び出した本人（認証済みユーザー）が確認済みにする。

**リクエスト**: なし

**レスポンス 200**: 更新後の回覧オブジェクト

全宛先が確認済みになった場合、文書ステータスを `completed` に変更する。

---

## タグ /tags

### GET /tags

```json
{
  "data": [
    { "id": "uuid", "name": "外形図" },
    { "id": "uuid", "name": "設備" }
  ],
  "meta": { "total": 20, "page": 1, "per_page": 20 }
}
```

### POST /tags

**必要ロール**: `admin`, `project_manager`, `general`

**リクエスト**:

```json
{ "name": "Rev1" }
```

**レスポンス 201**: タグオブジェクト
