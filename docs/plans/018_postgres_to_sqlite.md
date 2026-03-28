# PostgreSQL → SQLite3 移行計画

## Context

doc_man は現在 PostgreSQL に依存しているが、開発・デプロイの簡素化のため SQLite3 に移行する。PostgreSQL 固有機能（advisory lock、ARRAY型、`$N` バインドパラメータ等）が広範に使われており、マイグレーション・アプリコード・テスト・DevOps の全レイヤーで変更が必要。

## 変更概要

| カテゴリ | 変更箇所数 | 主な変更内容 |
|---|---|---|
| 依存関係 | 1 | sqlx feature `postgres` → `sqlite` |
| Rust型 | ~30箇所 | `PgPool` → `SqlitePool`、`PgConnection` → `SqliteConnection`、`QueryBuilder<sqlx::Postgres>` → `QueryBuilder<sqlx::Sqlite>` |
| SQL構文 | ~100箇所 | `$N` → `?N`、`now()` → `datetime('now')`、`FOR UPDATE` 削除、`ARRAY()` → 別クエリ、`ANY($1)` → `IN(...)` |
| エラーコード | ~16箇所 | PG code (`23514`等) → SQLite code (`275`等) |
| constraint名 | ~11箇所 | `db_err.constraint()` → エラーメッセージマッチ |
| マイグレーション | 22→2ファイル | 統合スキーマ + positions初期データ |
| テスト | 16ファイル+helpers | `PgPool` → `SqlitePool`、バインドパラメータ修正 |
| 設定 | 4ファイル | flake.nix、Justfile、docker-compose.yml、Containerfile |
| シードスクリプト | 6ファイル | PL/pgSQL → 純SQLに書き換え |

## 実装ステップ

### Step 1: マイグレーションファイル置換

既存の22ファイルを削除し、2ファイルに統合する。SQLite は `ALTER TABLE ADD CONSTRAINT` や `ALTER COLUMN SET NOT NULL` を非サポートのため、個別移行の書き換えではなく最終スキーマの直接定義が合理的。

**新規作成:**
- `server/migrations/20260328000001_initial_schema.sql` — 全テーブルの最終形を SQLite 構文で定義
- `server/migrations/20260328000002_seed_positions.sql` — positions 初期7件

**主な SQLite 翻訳:**

| PostgreSQL | SQLite |
|---|---|
| `UUID DEFAULT gen_random_uuid()` | `TEXT DEFAULT (lower(hex(randomblob(4)))\|\|'-'\|\|...)` ※UUID v4形式 |
| `TIMESTAMPTZ DEFAULT now()` | `TEXT DEFAULT (datetime('now'))` |
| `BOOLEAN` | `INTEGER` (sqlx が自動マッピング) |
| `GENERATED ALWAYS AS (... lpad(...)) STORED` | `GENERATED ALWAYS AS (... substr('0000000000'\|\|CAST(doc_seq AS TEXT), -frozen_seq_digits, frozen_seq_digits)) STORED` |
| `CREATE EXTENSION pgcrypto` | 不要 |

スキーマ先頭に `PRAGMA foreign_keys = ON;` を配置。

**削除するファイル:** `server/migrations/` 内の22ファイル全て

### Step 2: 依存関係・Rust型の変更

全ソースで PostgreSQL 型を SQLite 型に一括置換。これはコンパイルが通るまで一つのコミットにまとめる必要がある（部分的な型変更ではコンパイル不可）。

**`Cargo.toml` (workspace)**
```toml
# "postgres" → "sqlite"
sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "sqlite", "uuid", "chrono", "migrate"] }
```

**型置換一覧:**

| ファイル | 変更 |
|---|---|
| `server/src/state.rs` | `PgPool` → `SqlitePool` |
| `server/src/main.rs` | `PgPoolOptions` → `SqlitePoolOptions` |
| `server/src/authorization.rs` | `PgPool` → `SqlitePool` (3箇所) |
| `server/src/handlers/documents.rs` | `QueryBuilder<sqlx::Postgres>` → `QueryBuilder<sqlx::Sqlite>` (3箇所)、`sqlx::PgPool` → `sqlx::SqlitePool` (2箇所) |
| `server/src/handlers/projects.rs` | `QueryBuilder<sqlx::Postgres>` → `QueryBuilder<sqlx::Sqlite>` (3箇所) |
| `server/src/services/document_numbering.rs` | `sqlx::PgConnection` → `sqlx::SqliteConnection`、テスト内 `PgPool` → `SqlitePool` (5箇所) |
| `server/tests/helpers/mod.rs` | `PgPool` → `SqlitePool` (全箇所) |
| `server/tests/*.rs` (16ファイル) | `PgPool` → `SqlitePool` (全テスト関数) |

### Step 3: SQL 構文の移行

#### 3a: バインドパラメータ `$N` → `?N`

全 `sqlx::query()` / `sqlx::query_scalar()` 内の `$1`, `$2`, ... を `?1`, `?2`, ... に置換。SQLite は `$N` 構文を数値バインドとしてサポートしない。

対象: `server/src/` および `server/tests/` 配下の全 `.rs` ファイル（推定100箇所以上）

#### 3b: `now()` → `datetime('now')`

UPDATE/INSERT 文中の `now()` を `datetime('now')` に置換。

対象ファイル:
- `server/src/handlers/documents.rs` (3箇所: update, revise×2)
- `server/src/handlers/projects.rs` (1箇所: update)
- `server/src/handlers/approval_steps.rs` (3箇所: create, approve, reject)
- `server/src/handlers/departments.rs` (1箇所)
- `server/src/handlers/disciplines.rs` (1箇所)
- `server/src/handlers/document_kinds.rs` (1箇所)
- `server/src/handlers/document_registers.rs` (1箇所)
- `server/src/handlers/employees.rs` (1箇所)
- `server/src/handlers/positions.rs` (1箇所)

#### 3c: `FOR UPDATE` 削除

SQLite は行レベルロック非サポート。書き込みのシリアル化は WAL モードが保証する。

対象 (5箇所):
- `documents.rs:534` — update_document
- `documents.rs:705` — revise_document
- `approval_steps.rs:81` — create_approval_route
- `approval_steps.rs:195` — approve_step
- `approval_steps.rs:314` — reject_step

#### 3d: `ARRAY()` サブクエリ → 2クエリ分割

`server/src/auth.rs:88-92` の `ARRAY(SELECT ed2.department_id ...)` を削除し、別クエリで `department_ids` を取得する。

```rust
// 1. 従業員+ロール取得 (ARRAY句を除去)
let row = sqlx::query("SELECT e.id, e.name, e.is_active,
    COALESCE(e.role, drg.role, p.default_role) AS effective_role
    FROM employees e JOIN positions p ON p.id = e.position_id
    LEFT JOIN employee_departments ed ON ed.employee_id = e.id AND ed.effective_to IS NULL AND ed.is_primary = 1
    LEFT JOIN department_role_grants drg ON drg.department_id = ed.department_id
    WHERE e.employee_code = ?1")...;

// 2. 部署ID一覧を別取得
let department_ids: Vec<Uuid> = sqlx::query_scalar(
    "SELECT department_id FROM employee_departments WHERE employee_id = ?1 AND effective_to IS NULL")
    .bind(employee_id).fetch_all(&app_state.db)...;
```

#### 3e: `ANY($1)` → QueryBuilder `IN (...)`

3箇所を QueryBuilder ベースの動的 `IN (?, ?, ...)` に書き換え:
- `documents.rs:846` — `fetch_tags_batch`
- `distributions.rs:145` — メール送信先取得
- `distributions.rs:187` — 挿入結果取得

#### 3f: `$N::uuid IS NULL` 型キャスト削除

SQLite は型キャスト不要。`$1::uuid IS NULL` → `?1 IS NULL` に単純置換。

対象 (6箇所):
- `disciplines.rs:35,47` (各2箇所)
- `document_registers.rs:37,38,54,55` (各4箇所)

#### 3g: `pg_advisory_xact_lock()` 削除

`document_numbering.rs:46-50` のアドバイザリロック呼び出しを削除する。SQLite は単一ライター方式のため、トランザクション内での `SELECT MAX → 計算 → INSERT` は自動的にシリアル化される。

#### 3h: `SELECT EXISTS(...)` の型変換

SQLite の `EXISTS` は `0`/`1` (整数) を返す。`fetch_one` で `bool` に直接マップできない場合、`i32` で受けて `> 0` で変換する。

対象: `documents.rs:773`、`distributions.rs:90`

#### 3i: テストヘルパーの `::timestamptz AT TIME ZONE` 削除

`tests/helpers/mod.rs` 内の PostgreSQL 型キャスト・タイムゾーン変換を SQLite の `date()` 関数に置換。

### Step 4: エラーコードマッピング

PostgreSQL と SQLite のエラーコードが異なる。

| エラー種別 | PostgreSQL | SQLite |
|---|---|---|
| CHECK violation | `23514` | `275` |
| FK violation | `23503` | `787` |
| UNIQUE violation | `23505` | `2067` |

16箇所のエラーコード判定を置換。

**constraint名の取得:** SQLite の sqlx は `db_err.constraint()` で制約名を返さない可能性がある。その場合、`db_err.message()` のテキストマッチに変更する（例: `"UNIQUE constraint failed: departments.code"` を含むかチェック）。

対象ファイル: departments, disciplines, document_kinds, document_registers, documents, employees, positions, projects, approval_steps, distributions の各ハンドラー (11箇所の `constraint()` 呼び出し)

### Step 5: SQLite PRAGMA 設定

`server/src/main.rs` でプール作成時に必須 PRAGMA を設定:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .after_connect(|conn, _meta| Box::pin(async move {
        sqlx::query("PRAGMA journal_mode=WAL").execute(&mut *conn).await?;
        sqlx::query("PRAGMA busy_timeout=5000").execute(&mut *conn).await?;
        sqlx::query("PRAGMA foreign_keys=ON").execute(&mut *conn).await?;
        sqlx::query("PRAGMA synchronous=NORMAL").execute(&mut *conn).await?;
        Ok(())
    }))
    .connect(&db_url).await?;
```

**重要:** `foreign_keys=ON` は接続ごとに設定が必要（SQLite のデフォルトは OFF）。これがないと FK 制約が無視される。

### Step 6: 設定ファイル更新

#### `flake.nix`
- `pkgs.postgresql` → `pkgs.sqlite` に差し替え
- `shellHook` を簡素化: `PGDATA`/`pg_ctl` 等を全削除、`DATABASE_URL="sqlite:${PWD}/data/doc_man.db?mode=rwc"` に変更
- DB 初期化を `mkdir -p data && sqlx migrate run` に簡素化
- `.pgdata` を `.gitignore` から `data/` に変更

#### `Justfile`
```just
db-migrate:
    sqlx migrate run --source server/migrations

db-reset:
    rm -f data/doc_man.db data/doc_man.db-wal data/doc_man.db-shm
    mkdir -p data
    sqlx migrate run --source server/migrations

db-seed:
    sqlite3 data/doc_man.db < server/scripts/seed.sql
```
`db-stop` は削除（デーモンプロセスなし）。

#### `docker-compose.yml`
- `db` サービス (postgres:17) を削除
- `app` から `depends_on: db` を削除
- `DATABASE_URL` を `sqlite:/data/doc_man.db?mode=rwc` に変更
- `seed` サービスを SQLite ベースに変更
- `pgdata` ボリュームを `appdata` に変更

#### `Containerfile`
- Stage 3 (runtime) から `libpq5` を削除
- `libsqlite3-0` を追加（既に Debian bookworm に含まれているため実質不要だが明示）
- `RUN mkdir -p /data` を追加

### Step 7: シードスクリプト書き換え

PostgreSQL の PL/pgSQL（`DO $$ ... END $$`、`ARRAY()`、`generate_series()`、`make_date()`、`format()`、`TRUNCATE CASCADE`、`\ir`、`\echo`）は全て SQLite 非対応。

**方針:**
- `seed.sql` を単一ファイルに統合（`\ir` 非対応のため）
- `DO $$` ブロック → 削除（安全チェックはコメントに）
- `TRUNCATE ... CASCADE` → `DELETE FROM` を依存順で実行
- `\echo` → コメントに
- `01_master.sql`〜`04_workflows.sql` → 純 INSERT 文に書き換え（大部分は既にそう）
- `05_historical.sql` → Rust スクリプトで事前生成した INSERT 文に変換、またはシード投入を Rust バイナリで実行

### Step 8: CLAUDE.md 更新

`CLAUDE.md` 内の PostgreSQL 関連の記述を SQLite に更新（DB コマンド、アーキテクチャ説明等）。

## コミット構成

TDD の RED→GREEN は型レベルの一括変更には適用困難なため、以下の論理単位でコミット:

1. マイグレーションファイル置換 (Step 1)
2. 依存関係・型変更 + SQL構文移行 + エラーコード (Step 2-4) — コンパイル通過に必要な最小セット
3. PRAGMA 設定 (Step 5)
4. テストが全て通ることを確認
5. 設定ファイル更新 (Step 6)
6. シードスクリプト書き換え (Step 7)
7. CLAUDE.md 更新 (Step 8)

## 検証

1. `cargo build` — コンパイル通過
2. `cargo clippy --workspace --all-targets` — lint 通過
3. `cargo fmt --all -- --check` — フォーマット確認
4. `cargo test` — 全統合テスト通過（SQLite で `#[sqlx::test]` が自動的に一時DB作成）
5. `just db-reset && just db-seed && just run` — 手動動作確認
6. 重点確認項目:
   - 文書採番（advisory lock 削除後の連番一意性）
   - 生成列 `doc_number`（`lpad()` → `substr()` 置換の正確性）
   - 認証（`ARRAY()` → 2クエリの動作）
   - FK 制約（`PRAGMA foreign_keys=ON` の効果）
   - UUID 生成（SQLite DEFAULT 式のパース可否）

## リスク

| リスク | 影響 | 対策 |
|---|---|---|
| バインドパラメータ `$N`→`?N` の変換漏れ | ランタイムエラー | grep で残存確認、テストでカバー |
| `constraint()` が SQLite で `None` を返す | エラーハンドリング不正確 | メッセージマッチにフォールバック |
| `EXISTS` の型差異 (bool vs int) | パースエラー | `i32` で受けて変換 |
| タイムスタンプ精度（秒 vs マイクロ秒） | ソート順の差異 | 必要なら `strftime('%Y-%m-%dT%H:%M:%f','now')` |
| 同時書き込み性能 | 高負荷時のレイテンシ | WAL + busy_timeout で緩和 |
