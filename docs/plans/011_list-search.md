# 文書・プロジェクト一覧のテキスト検索機能

## Context

文書一覧・プロジェクト一覧ページにはフィルタ/検索UIがなく、目的のデータを探すにはページを順にめくるしかない。サーバサイドで標準SQLの `LOWER() LIKE` による部分一致検索を追加し、フロントエンドでは debounce 付きリアルタイム検索を実装する。

**制約**: DB固有の機能（`pg_trgm`, `ILIKE` 等）は使わない。標準SQLのみで実装する。

## 方針

- クエリパラメータ `q` を追加。空文字・未指定時はフィルタなし（既存動作を維持）
- SQL は `LOWER(column) LIKE '%' || $q || '%'` で部分一致検索（`$q` はサーバ側で `LOWER()` + エスケープ済み）
- ユーザ入力の `%`, `_`, `\` はサーバ側でエスケープ
- 文書: `title` と `doc_number` を OR 検索
- プロジェクト: `name` を検索
- DB固有のインデックスは追加しない（マイグレーション不要）
- フロントエンドは 300ms debounce で API 通信。検索語変更時にページを 1 にリセット
- worktree で作業する

## 変更ファイル

### 1. バックエンド

**`server/src/handlers/documents.rs`**

- `DocumentListQuery` に `q: Option<String>` を追加
- `escape_like()` ヘルパー関数を追加（`\` → `\\`, `%` → `\%`, `_` → `\_`）
- `q` を `Option::filter(|s| !s.is_empty())` → `escape_like()` → `LOWER()` → `Option<String>` に変換
- COUNT クエリと SELECT クエリの WHERE 句に条件追加:

  ```sql
  AND ($2::text IS NULL OR LOWER(d.title) LIKE '%' || $2 || '%' OR LOWER(d.doc_number) LIKE '%' || $2 || '%')
  ```

- バインド変数の番号をシフト（`$2` が `q`、`$3`/`$4` が LIMIT/OFFSET）

**`server/src/handlers/projects.rs`**

- `ProjectListQuery` に `q: Option<String>` を追加
- 同様に WHERE 句を追加:

  ```sql
  AND ($4::text IS NULL OR LOWER(p.name) LIKE '%' || $4 || '%')
  ```

- バインド変数の番号をシフト

### 3. フロントエンド API クライアント

**`frontend/src/api/documents.rs`**

- `list()` に `q: &str` パラメータを追加
- URL に `&q={encoded}` を付与（空文字なら省略）

**`frontend/src/api/projects.rs`**

- `list()` に `q: &str` パラメータを追加
- 同上

### 4. フロントエンド ページ

**`frontend/src/pages/documents/list.rs`**

- `RwSignal<String>` で `search_query`（確定済み検索語）を管理
- `<input>` の `on:input` で 300ms debounce → `search_query` 更新 + `page.set(1)`
- `LocalResource` の依存に `search_query` を追加し API に渡す

**`frontend/src/pages/projects/list.rs`**

- 同様の構造

### 4. 呼び出し元の修正

`api::documents::list()` / `api::projects::list()` のシグネチャ変更に伴い、既存の呼び出し元も `""` を渡すよう修正。

## コミット順序（TDD）

1. `test: add search query tests for document list endpoint`（RED）
   - `q=テスト` でタイトルにマッチする文書のみ返ること
   - `q=DOC-001` で文書番号にマッチする文書のみ返ること
   - `q` 未指定で全件返ること（既存動作の確認）
   - `q` に `%` が含まれても LIKE インジェクションしないこと
2. `feat: add q search parameter to document list endpoint`（GREEN）
3. `test: add search query tests for project list endpoint`（RED）
   - `q=テスト` でプロジェクト名にマッチするもののみ返ること
   - `q` 未指定で全件返ること
4. `feat: add q search parameter to project list endpoint`（GREEN）
5. `feat: add debounced search input to document and project list pages`
   - フロントエンドは自動テスト基盤がないため単一コミット

## 検証方法

1. `cargo test` — 全統合テストがパス
2. `just lint` — clippy がパス
3. 手動確認:
   - `just run` + `just frontend-dev` で起動
   - 文書一覧で検索ボックスにテキスト入力 → テーブルがフィルタされること
   - 検索語をクリア → 全件表示に戻ること
   - プロジェクト一覧でも同様に動作すること
   - ページネーションが検索結果の件数に基づいて正しく動作すること
