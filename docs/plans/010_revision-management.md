# 改訂管理の充実

## Context

文書の改訂（revision）管理が不十分。現状は `documents.revision` が PUT 更新のたびに自動インクリメントされるだけで、改訂履歴テーブルがなく、旧版の追跡ができない。また `file_path` がユーザー手入力であり、改訂ごとのディレクトリ構造が保証されない。

**目標**: 明示的な改訂操作と改訂履歴により、承認済み文書の改訂サイクルを正しく管理する。ファイルパスは `{doc_number}/{revision}` で自動生成する。

## 方針

### ファイルパス

- `file_path` は `document_revisions` テーブルのみに保持
- `documents` テーブルから `file_path` カラムを DROP
- 形式: `{doc_number}/{revision}`（例: `内設計-2603001/0`）
- `DocumentResponse` には `document_revisions`（`effective_to IS NULL`）から JOIN して取得

### Revision は 0 始まり

- Rev.0 = 初版（文書作成時）
- Rev.1 = 最初の明示的改訂（承認後）
- Rev.N = N 回目の改訂

### 改訂ライフサイクル

```text
作成 → Rev.0 / draft
  ↓ 自由に編集（revision は変わらない）
  ↓ 承認ルート作成 → under_review
  ↓ 全員承認 → approved
  ↓ 明示的に「改訂」操作（POST /revise + 理由）
改訂 → Rev.1 / draft
  ↓ （同じサイクル）
```

- PUT での revision 自動インクリメントは**廃止**
- revision は `POST /documents/{id}/revise` でのみ増加
- revise は `approved` 状態のみ許可
- 却下（rejected）→ 同じ revision のまま再編集・再承認

### 改訂履歴テーブル

- 全 revision を記録（Rev.0 含む）
- Rev.0: 文書作成時に自動記録、reason = NULL
- Rev.1+: revise 時に記録、reason 必須
- `effective_from` / `effective_to` で各改訂の有効期間を管理（departments テーブルと同パターン）
  - 現行改訂: `effective_to = NULL`
  - 新改訂作成時: 旧改訂の `effective_to` を現在時刻で閉じる

## データベース

### Migration 17: `document_revisions` テーブル作成 + `documents.file_path` 削除

```sql
CREATE TABLE document_revisions (
    id             UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id    UUID        NOT NULL REFERENCES documents(id),
    revision       INTEGER     NOT NULL CHECK (revision >= 0),
    file_path      VARCHAR(500) NOT NULL,
    reason         TEXT,
    created_by     UUID        NOT NULL REFERENCES employees(id),
    effective_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_to   TIMESTAMPTZ,
    UNIQUE (document_id, revision)
);

ALTER TABLE documents DROP COLUMN file_path;
ALTER TABLE documents ALTER COLUMN revision SET DEFAULT 0;
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_revision_check;
ALTER TABLE documents ADD CONSTRAINT documents_revision_check CHECK (revision >= 0);
```

### Migration 18: 既存データのバックフィル

```sql
-- 既存文書の revision (1始まり) を document_revisions に展開
-- revision=N の文書に対し Rev.0〜Rev.N-1 を作成
-- Rev.N-1 が現行改訂 (effective_to = NULL)、それ以前は閉じる
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from, effective_to)
SELECT
    d.id,
    gs.rev,
    d.doc_number || '/' || gs.rev,
    NULL,
    d.author_id,
    d.created_at,
    CASE WHEN gs.rev < d.revision - 1 THEN d.created_at ELSE NULL END
FROM documents d
CROSS JOIN LATERAL generate_series(0, d.revision - 1) AS gs(rev)
WHERE NOT EXISTS (
    SELECT 1 FROM document_revisions dr WHERE dr.document_id = d.id AND dr.revision = gs.rev
);

-- 1始まり → 0始まりに変換 (revision N → N-1)
UPDATE documents SET revision = revision - 1 WHERE revision >= 1;
```

## サーバー変更

### モデル

**新規: `server/src/models/document_revision.rs`**

- `ReviseDocumentRequest { reason: String }`
- `DocumentRevisionResponse { id, document_id, revision, file_path, reason, created_by: NameBrief, effective_from, effective_to }`

**変更: `server/src/models/document.rs`**

- `CreateDocumentRequest`: `file_path` フィールドを削除
- `UpdateDocumentRequest`: `file_path` フィールドを削除
- `DocumentResponse`: `file_path` は残す（`document_revisions` から JOIN で取得）

### ハンドラ

**変更: `server/src/handlers/documents.rs`**

1. **`create_document`**: INSERT から `file_path` を除外。トランザクション内で `document_revisions` に Rev.0 を INSERT（`file_path = {doc_number}/0`）
2. **`update_document`**: revision 自動インクリメント（315-326行目）を削除。`UpdateDocumentRequest` から `file_path` を除外し、UPDATE SQL からも `file_path` と `revision` を除外
3. **`delete_document`**: `document_revisions` を先に削除（FK 制約）
4. **`fetch_document_by_id`** / **`list_documents`**: `document_revisions`（`effective_to IS NULL`）を JOIN して `file_path` を取得

**新規ハンドラ（同ファイルに追加）:**

5. **`revise_document`** (`POST /documents/{id}/revise`):
   - 権限: viewer 以外
   - status == "approved" のみ許可（それ以外は 422）
   - reason 必須（空なら 400）
   - トランザクション内で:
     - 旧改訂の `effective_to` を現在時刻で閉じる
     - `document_revisions` に新 revision を INSERT（`effective_from = now()`, `effective_to = NULL`）
     - `documents` の revision, status(→draft) を更新
   - 更新後の `DocumentResponse` を返却

6. **`list_document_revisions`** (`GET /documents/{id}/revisions`):
   - 権限: 認証済みユーザー
   - `document_revisions` JOIN `employees` を revision DESC で返却

### ルート

**`server/src/routes/mod.rs`** に追加:

```rust
.route("/api/v1/documents/{id}/revise", post(documents::revise_document))
.route("/api/v1/documents/{id}/revisions", get(documents::list_document_revisions))
```

## フロントエンド変更

### API 型 (`frontend/src/api/types.rs`)

- `CreateDocumentRequest`: `file_path` 削除
- `UpdateDocumentRequest`: `file_path` 削除
- `ReviseDocumentRequest` と `DocumentRevisionResponse` を追加

### API クライアント (`frontend/src/api/client.rs` or `documents.rs`)

- `revise(id, req)` と `list_revisions(id)` を追加

### 作成ページ (`frontend/src/pages/documents/create.rs`)

- `form_file_path` シグナルとフォームフィールドを削除
- バリデーションから `file_path.is_empty()` チェックを削除

### 詳細ページ (`frontend/src/pages/documents/detail.rs`)

- ファイルパス行を常に読み取り専用に変更（編集モードでも input にしない）
- 編集モードの `form_file_path` と `UpdateDocumentRequest` への `file_path` 設定を削除
- 「改訂」ボタン追加: `status == "approved"` かつ `can_edit` の場合に表示。理由入力フォーム付き
- 改訂履歴セクション追加: サイドバーまたはテーブル下部に revision 一覧を表示（effective_from/to 表示）

### 一覧ページ (`frontend/src/pages/documents/list.rs`)

- 「Rev.」列を追加（文書番号の隣）

## テストヘルパー変更

**`server/tests/helpers/mod.rs`**

- `insert_document`: `file_path` を INSERT 文から除外（カラムが存在しないため）。`document_revisions` に Rev.0 を INSERT（`file_path = {doc_number}/0`, `effective_from = now()`, `effective_to = NULL`）

## シードデータ変更

**`server/scripts/seed.sql`**

- 全文書の INSERT から `file_path` カラムを除外
- `revision` のデフォルトを利用（0）
- 各文書に対応する `document_revisions` の Rev.0 レコードを追加（`file_path = {doc_number}/0`）

## 実装フェーズ（TDD）

### Phase 1: DB マイグレーション + テストヘルパー + シードデータ

- マイグレーション 17, 18 を追加
- テストヘルパーの `insert_document` を更新（`file_path` 除外、`document_revisions` 追加）
- シードデータ更新
- 既存テストが引き続きパスすることを確認

### Phase 2: 文書作成 — file_path 自動生成 + revision レコード

- RED: `post_document_auto_generates_file_path`, `post_document_creates_revision_record`
- GREEN: `create_document` ハンドラ改修、`CreateDocumentRequest` から `file_path` 削除、`fetch_document_by_id` に JOIN 追加
- 既存テスト更新: リクエスト JSON から `file_path` 削除、アサーション更新

### Phase 3: 文書更新 — revision 自動インクリメント廃止

- RED: `put_document_does_not_increment_revision`
- GREEN: `update_document` ハンドラ改修
- 既存テスト更新: `put_document_updates_title_and_increments_revision` → `revision == 0` に変更
- 備考: `UpdateDocumentRequest` から `file_path` を削除。未知フィールドは serde が無視するため 422 テストは不要

### Phase 4: 改訂エンドポイント

- RED: `revise_approved_document_creates_new_revision`, `revise_creates_revision_record`, `revise_draft_returns_422`, `revise_requires_reason`, `revise_viewer_returns_403`
- GREEN: `revise_document` ハンドラ + ルート追加

### Phase 5: 改訂履歴エンドポイント

- RED: `get_revisions_returns_history`, `get_revisions_for_new_document_returns_one`
- GREEN: `list_document_revisions` ハンドラ + ルート追加

### Phase 6: 削除時の改訂レコード処理

- RED: `delete_document_with_revisions_succeeds`
- GREEN: `delete_document` に `document_revisions` 削除を追加

### Phase 7: フロントエンド

- API 型・クライアント更新
- 作成ページ: file_path 除去
- 詳細ページ: file_path 読み取り専用化、改訂ボタン、改訂履歴セクション
- 一覧ページ: Rev. 列追加

## 検証

1. `cargo test` — 全テストパス
2. `just lint && just fmt-check` — 警告なし
3. ブラウザ確認:
   - 文書作成: file_path 入力欄がない、作成後に `{doc_number}/0` が表示される
   - 文書編集: file_path が編集不可、タイトル変更しても revision が変わらない
   - 承認済み文書: 「改訂」ボタンが表示、理由入力後に Rev.1 / draft になる
   - 改訂履歴: 各 revision の file_path、理由、作成者、有効期間が表示される
   - 文書一覧: Rev. 列が表示される
