# ドキュメント台帳の統合方針

## Context

ドキュメント台帳（メタデータ）が14のDBに分散。register_code（文書番号のハイフン前）ごとにパスルール・所在DBが異なる。

| ソース          | 文書DB数 | 更新状況   | 廃止可否               |
| --------------- | -------- | ---------- | ---------------------- |
| MS SQL Server A | 2        | **更新中** | 不可（他システム参照） |
| MS SQL Server B | 5        | ほぼ凍結   | 不可（他システム参照） |
| MySQL           | 4        | ほぼ凍結   | 可能                   |
| Access          | 3        | ほぼ凍結   | 可能                   |
| **計**          | **14**   |            |                        |

doc_man は PostgreSQL 単一DB で運用中。レガシーデータは**閲覧・検索のみ**の要件。
**特定DBへのロックインを避ける**方針のため、doc_man に SQL Server 依存を直接入れず、ETL を介して疎結合に保つ。

## 方針

### 1. MySQL + Access + SQL Server B → SQL Server A に統合

Server A に統一スキーマの新規DB（DB-unified）を作成し、12DB のデータを集約。

- Server A の既存2DB（更新中・他システム参照）はそのまま維持
- 凍結データ12DB分を統一スキーマで1つのDBに集約
- DBエンジンを 4種→2種 に削減、サーバも1台に集約
- MySQL・Access を廃止（Access のファイル破損リスク解消）
- Server B も最終的に廃止可能

**統一スキーマの設計方針：**

- 共通カラム: doc_number, title, author, created_date, register_code 等
- DB固有フィールド: `extra_attributes` (JSON) で吸収。共通検索は構造化カラムで行い、特殊フィールドも失わない
- `source_db` カラムで元のDB出自を記録（トレーサビリティ）

移行後：

```text
Server A
├── DB-1（既存・更新中・他システム参照）── そのまま維持
├── DB-2（既存・更新中・他システム参照）── そのまま維持
└── DB-unified（新規・統一スキーマ）    ── 12DB分を集約
```

### 2. doc_man からの読み取り：定期同期

SQL Server のデータを定期 ETL で PostgreSQL `legacy_documents` テーブルに同期。doc_man 本体は PostgreSQL のみ。

```text
Server A DB-1 (更新中) ──(ETL 高頻度)──┐
Server A DB-2 (更新中) ──(ETL 高頻度)──┼→ PostgreSQL legacy_documents
Server A DB-unified    ──(ETL 日次)────┘         ↑
                                             doc_man (sqlx のみ)
```

- DB-1, DB-2（更新中）: 15〜30分間隔
- DB-unified（凍結データ）: 日次 or オンデマンド（初回同期後はほぼ不変）

### 3. ルーティング：`document_registers` を拡張

既存の `document_registers` テーブルに `legacy_data_source` カラムを追加。register_code ごとに「どのサーバ・DBからデータを取得するか」を管理。

```sql
ALTER TABLE document_registers
    ADD COLUMN legacy_data_source VARCHAR(50);
```

| register_code | legacy_data_source |
| ------------- | ------------------ |
| 内設計        | ss_a_db1           |
| 仕機設        | ss_a_db2           |
| 議品管        | ss_b_db3           |
| 外機設        | ss_b_db4           |
| ...           | ...                |

ETL スクリプトはこのテーブルを参照して、各 register のデータを対応するサーバ・DB から取得。

### 4. 接続設定

環境変数でデータソース名と接続情報を対応（認証情報はDBに入れない）：

```text
LEGACY_DS_SS_A_DB1=mssql://user:pass@server-a/db1
LEGACY_DS_SS_A_DB2=mssql://user:pass@server-a/db2
LEGACY_DS_SS_A_DB3=mssql://user:pass@server-a/db3
...
```

## 実装ステップ

### Phase 1: 12DB のスキーマ調査 + 統一スキーマ設計

1. 12DB（SQL Server B 5 + MySQL 4 + Access 3）のスキーマ・データ調査
2. 共通カラムの特定、差異の洗い出し
3. DB-unified の統一スキーマ設計（共通カラム + `extra_attributes` JSON）
4. register_code → 元DB のマッピング表作成

### Phase 2: SQL Server A に DB-unified 作成・データ移行

**言語: Python**（pyodbc で SQL Server・MySQL・Access 全てに接続可。Windows 環境で OLE DB/ODBC ネイティブ利用）

1. Server A に DB-unified を作成
2. Python 移行スクリプト作成・実行（DB ごとにマッピング定義 → 統一スキーマに変換）
3. 件数照合・サンプル突合で検証
4. MySQL・Access・SQL Server B 廃止

### Phase 3: doc_man にレガシー閲覧機能を追加

1. **migration**: `document_registers` に `legacy_data_source` カラム追加
2. **migration**: `legacy_documents` テーブル作成
   - register_code, doc_number, title, author 等の共通メタデータ
   - `data_source` カラム（出自の記録）
   - `synced_at` カラム（同期日時）
3. **ETL スクリプト（Python）**: SQL Server A → PostgreSQL 同期
   - `document_registers` の `legacy_data_source` を参照してソース切替
   - パラメータ化して全DB分を1つのスクリプトで処理
   - 差分同期 or 全件洗い替え（DB ごとに適切な方式を選定）
   - Phase 1 の移行スクリプトを再利用・拡張
4. **server**: `GET /api/v1/legacy-documents` ハンドラ追加
   - 検索・一覧（既存の pagination/error を再利用）
   - register_code, title, doc_number 等でフィルタ可能
5. **frontend**: レガシーデータ閲覧ページ
   - 既存 document list UI を参考に
   - ネイティブデータとの視覚的区別

### Phase 4: 運用整備

1. ETL の定期実行設定（cron: SQL Server A は高頻度、B は日次）
2. 同期結果の監視（件数差異検知）

## 修正対象ファイル

- `server/migrations/` — 新規 migration 2件
- `server/src/models/` — `document_register.rs` 更新、`legacy_document.rs` 新規
- `server/src/handlers/` — `legacy_documents.rs` 新規
- `server/src/routes/mod.rs` — エンドポイント追加
- `frontend/src/pages/` — レガシー閲覧ページ新規
- `frontend/src/api/` — types, client にレガシー用追加
- ETL スクリプト（`server/scripts/etl/` 等）

## 検証

- Phase 2: 旧DB と DB-unified の件数・サンプル突合
- Phase 3: ETL 同期後、PostgreSQL と SQL Server の件数一致
- doc_man: レガシーデータの検索・表示が正常動作
- 既存機能: `cargo test` 全テストパス

## 着手前に必要な情報（Phase 1 開始時）

- 14DB それぞれのスキーマ（カラム一覧）とレコード件数
- register_code → DB のマッピング表
- SQL Server A への接続情報・権限
- 各DBの既存スキーマのサンプル（統一スキーマ設計の入力）
