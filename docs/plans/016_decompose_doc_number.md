# 文書番号の複合カラム化

## Context

`documents.doc_number` は `{種別コード}{部署コード}-{期間}{連番}` の書式で構成される派生値だが、現在は単一 `VARCHAR(30)` カラムとして保存されている。このため:

- シードスクリプト (`05_historical.sql`) で `format('%03s', ...)` が空白埋めになるバグが発生
- 連番が文字列の中に埋もれており、`LIKE` + 文字列パースで次番を取得している
- `議`（2桁連番）と他の種別（3桁連番）の区別が採番ロジック依存

構成要素を個別カラムに分解し `doc_number` を生成列にすることで、連番を整数として扱い、フォーマットをDBが保証する構造に変更する。

## ブランチ

`feat/decompose-doc-number` ブランチで実施。

## 新スキーマ

```sql
-- 追加カラム
frozen_kind_code   VARCHAR(5)  NOT NULL  -- document_kinds.code を凍結
doc_period         TEXT        NOT NULL  -- 現在は 4 桁の 'YYMM' を使用（将来の柔軟性を考慮して TEXT 型）
doc_seq            INT         NOT NULL  -- 連番（整数）
frozen_seq_digits  SMALLINT    NOT NULL  -- document_kinds.seq_digits を凍結

-- doc_number を生成列に変更
doc_number VARCHAR(30) GENERATED ALWAYS AS (
    frozen_kind_code || frozen_dept_code || '-'
    || doc_period
    || lpad(doc_seq::text, frozen_seq_digits, '0')
) STORED

-- 複合ユニーク制約
UNIQUE (frozen_kind_code, frozen_dept_code, doc_period, doc_seq)
```

`frozen_kind_code` / `frozen_seq_digits` は `frozen_dept_code` と同じ凍結パターン。マスタ変更が将来あっても既存の文書番号が変わらない。

## マイグレーション

**ファイル:** `server/migrations/20260324000021_decompose_doc_number.sql`

```sql
-- 1. カラム追加（nullable）
ALTER TABLE documents
    ADD COLUMN frozen_kind_code  VARCHAR(5),
    ADD COLUMN doc_period        TEXT,
    ADD COLUMN doc_seq           INT,
    ADD COLUMN frozen_seq_digits SMALLINT;

-- 2. 既存データのバックフィル
UPDATE documents d SET
    frozen_kind_code  = dk.code,
    frozen_seq_digits = dk.seq_digits,
    doc_period = left(
        split_part(d.doc_number, '-', 2),
        length(split_part(d.doc_number, '-', 2)) - dk.seq_digits
    ),
    doc_seq = right(
        split_part(d.doc_number, '-', 2),
        dk.seq_digits
    )::INT
FROM document_kinds dk
WHERE dk.id = d.doc_kind_id;

-- 3. NOT NULL 制約
ALTER TABLE documents
    ALTER COLUMN frozen_kind_code  SET NOT NULL,
    ALTER COLUMN doc_period        SET NOT NULL,
    ALTER COLUMN doc_seq           SET NOT NULL,
    ALTER COLUMN frozen_seq_digits SET NOT NULL;

-- 4. 旧 doc_number カラム削除
DROP INDEX IF EXISTS idx_documents_doc_number;
ALTER TABLE documents DROP CONSTRAINT documents_doc_number_unique;
ALTER TABLE documents DROP COLUMN doc_number;

-- 5. 生成列として再作成
ALTER TABLE documents ADD COLUMN doc_number VARCHAR(30) GENERATED ALWAYS AS (
    frozen_kind_code || frozen_dept_code || '-'
    || doc_period
    || lpad(doc_seq::text, frozen_seq_digits, '0')
) STORED;

-- 6. 制約・インデックス再作成
ALTER TABLE documents ADD CONSTRAINT documents_doc_number_unique UNIQUE (doc_number);
CREATE INDEX idx_documents_doc_number ON documents(doc_number);
ALTER TABLE documents ADD CONSTRAINT documents_composite_unique
    UNIQUE (frozen_kind_code, frozen_dept_code, doc_period, doc_seq);
```

## サーバー変更

### 1. 採番サービス (`server/src/services/document_numbering.rs`)

戻り値を構造体に変更。`LIKE` + 文字列パースを `MAX(doc_seq)` に置換。

```rust
pub struct DocNumberParts {
    pub frozen_kind_code: String,
    pub doc_period: String,        // YYMM
    pub doc_seq: i32,
    pub frozen_seq_digits: i32,
}

pub async fn assign_doc_number(...) -> Result<DocNumberParts, AppError> {
    // prefix ハッシュ → advisory lock（既存ロジック流用）
    // SELECT COALESCE(MAX(doc_seq), 0) + 1
    //   FROM documents
    //   WHERE frozen_kind_code = $1
    //     AND frozen_dept_code = $2
    //     AND doc_period = $3
    // → DocNumberParts を返す
}
```

### 2. ハンドラ (`server/src/handlers/documents.rs`)

**CREATE (POST):**

- `assign_doc_number()` から `DocNumberParts` を受け取る
- INSERT に `frozen_kind_code`, `doc_period`, `doc_seq`, `frozen_seq_digits` をバインド
- `RETURNING id, doc_number` で生成列の値を取得
- 返された `doc_number` で `document_revisions.file_path` を構成

**LIST (GET):**

- `doc_number` の LIKE 検索は生成列に対してそのまま動作。変更不要

**UPDATE (PUT):**

- `doc_number` 変更拒否ロジックはそのまま（生成列なので DB レベルでも拒否される）

**REVISE (POST /:id/revise):**

- `SELECT doc_number` は生成列から読めるので変更不要

### 3. モデル (`server/src/models/document.rs`)

`DocumentResponse` は変更不要（`doc_number: String` は SELECT で取得可能）。

## シード変更

### `03_documents.sql`

手動 INSERT の `doc_number` を構成要素に分解:

```sql
-- Before:
INSERT INTO documents (doc_number, ..., frozen_dept_code, ...)
VALUES ('内設計-2603001', ..., '設計', ...);

-- After:
INSERT INTO documents (frozen_kind_code, frozen_dept_code, doc_period, doc_seq, frozen_seq_digits, ...)
VALUES ('内', '設計', '2603', 1, 3, ...);
-- doc_number は DB が '内設計-2603001' を自動生成
```

### `05_historical.sql`

`format()` による文字列構築を廃止し、構成要素を直接 INSERT:

```sql
INSERT INTO documents (
    frozen_kind_code, frozen_dept_code, doc_period, doc_seq, frozen_seq_digits,
    title, author_id, doc_kind_id, status, confidentiality, project_id, created_at
) VALUES (
    v_dk_codes[v_ki], v_disc_depts[v_di], v_yymm, v_ni, v_dk_digits[v_ki],
    ...
);
```

`doc_number` のフォーマットは DB の生成列が担当。シードで `%03s` を使う必要がなくなる。

### `04_workflows.sql`

`doc_number` を直接参照している箇所があれば、サブクエリで生成列を参照（変更不要の可能性が高い）。

## テスト変更

### ヘルパー (`server/tests/helpers/mod.rs`)

`insert_document()` のシグネチャ変更:

```rust
pub async fn insert_document(
    pool: &PgPool,
    kind_code: &str,    // "内"
    dept_code: &str,    // "設計"
    period: &str,       // "2603"
    seq: i32,           // 1
    title: &str,
    author_id: Uuid,
    doc_kind_id: Uuid,
    project_id: Uuid,
) -> Uuid
```

`seq_digits` はヘルパー内で `document_kinds` テーブルから取得。

### 採番テスト (`document_numbering.rs`)

- `assign_doc_number` の戻り値が `DocNumberParts` に変わる
- 既存の `assert_eq!(result, "内設計-2603001")` → `assert_eq!(result.doc_seq, 1)` 等に変更
- `increments_existing_sequence` テスト: 文書挿入を構成要素ベースに書き換え

### 統合テスト (`server/tests/documents.rs`)

- ヘルパー呼び出しの引数変更
- `doc_number` アサーションは API レスポンスの `doc_number` フィールドで確認（生成列の値）

## TDD サイクル

1. **RED:** テストヘルパーと採番テストを新スキーマ対応に書き換え（失敗する状態でコミット）
2. **GREEN:** マイグレーション + サービス + ハンドラを実装（テスト通過でコミット）
3. **REFACTOR:** シードの簡素化、不要コードの整理

## 検証

```bash
# マイグレーション + シードの投入
just db-reset && just db-seed

# 生成列が正しく動作すること
psql -c "SELECT doc_number, frozen_kind_code, doc_period, doc_seq FROM documents LIMIT 10"

# 議事録が2桁連番であること
psql -c "SELECT doc_number FROM documents WHERE frozen_kind_code = '議' LIMIT 5"

# 空白を含む番号がないこと
psql -c "SELECT doc_number FROM documents WHERE doc_number ~ '\s'"

# テスト
cargo test

# Lint
just lint
```
