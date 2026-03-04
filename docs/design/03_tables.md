# テーブル定義

## departments（部署）

| カラム           | 型             | NULL | デフォルト          | 説明                                         |
| ---------------- | -------------- | ---- | ------------------- | -------------------------------------------- |
| `id`             | `UUID`         | NO   | `gen_random_uuid()` | PK                                           |
| `code`           | `VARCHAR(10)`  | NO   | -                   | 採番用コード。不変。部署廃止後も再利用しない |
| `name`           | `VARCHAR(100)` | NO   | -                   | 部署名                                       |
| `parent_id`      | `UUID`         | YES  | `NULL`              | 親部署。NULL = ルート部署                    |
| `effective_from` | `DATE`         | NO   | -                   | 有効期間開始日                               |
| `effective_to`   | `DATE`         | YES  | `NULL`              | 有効期間終了日。NULL = 現在有効              |
| `merged_into_id` | `UUID`         | YES  | `NULL`              | 統合先部署。廃止理由が統合の場合に設定       |
| `created_at`     | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                             |
| `updated_at`     | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                             |

**制約**:

- `UNIQUE (code)`
- `FK parent_id → departments(id)`
- `FK merged_into_id → departments(id)`

**インデックス**:

- `(code)` -文書番号採番時の部署コード検索
- `(parent_id)` -階層ツリー走査

**設計理由**:
部署構造は AD の OU 階層から取得・加工して同期する。`name` や `parent_id` は AD 由来だが、`code` は文書番号採番用に本システムで付与する（AD 属性ではない）。
`is_active` フラグではなく `effective_from/to` で有効期間を管理する。
廃止・統合があっても過去の文書番号や所属履歴を変更せず、期間クエリで現時点の状態を再現できる。
`code` は採番キーとして不変にする。廃止後も同じコードの部署が復活することはなく、文書番号の一意性を保証する。

---

## employees（社員）

| カラム          | 型             | NULL | デフォルト          | 説明                              |
| --------------- | -------------- | ---- | ------------------- | --------------------------------- |
| `id`            | `UUID`         | NO   | `gen_random_uuid()` | PK                                |
| `name`          | `VARCHAR(100)` | NO   | -                   | 氏名                              |
| `employee_code` | `VARCHAR(20)`  | YES  | `NULL`              | 社員番号（AD 識別子）             |
| `ad_account`    | `VARCHAR(100)` | YES  | `NULL`              | AD アカウント名。退職後は NULL    |
| `role`          | `VARCHAR(20)`  | NO   | `'viewer'`          | RBAC ロール                       |
| `is_active`     | `BOOLEAN`      | NO   | `true`              | 在籍フラグ。退職時に false へ変更 |
| `created_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                  |
| `updated_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                  |

**制約**:

- `UNIQUE (employee_code)` （NOT NULL の場合）
- `UNIQUE (ad_account)` （NOT NULL の場合）
- `CHECK (role IN ('admin', 'project_manager', 'general', 'viewer'))`

**インデックス**:

- `(employee_code)` -AD 同期のキー照合
- `(ad_account)` -AD 同期のキー照合
- `(is_active)` -在籍者フィルタ

**設計理由**:
退職者のレコードを削除しない。過去の文書の `author_id` や承認履歴が孤立するのを防ぐ。
所属部署は `employee_departments` に移し、部門異動の履歴を追跡可能にする。

---

## employee_departments（社員所属履歴）

| カラム           | 型            | NULL | デフォルト          | 説明                        |
| ---------------- | ------------- | ---- | ------------------- | --------------------------- |
| `id`             | `UUID`        | NO   | `gen_random_uuid()` | PK                          |
| `employee_id`    | `UUID`        | NO   | -                   | 社員                        |
| `department_id`  | `UUID`        | NO   | -                   | 所属部署                    |
| `is_primary`     | `BOOLEAN`     | NO   | `false`             | 主務フラグ                  |
| `effective_from` | `DATE`        | NO   | -                   | 所属開始日                  |
| `effective_to`   | `DATE`        | YES  | `NULL`              | 所属終了日。NULL = 現在所属 |
| `created_at`     | `TIMESTAMPTZ` | NO   | `now()`             | レコード作成日時            |

**制約**:

- `FK employee_id → employees(id)`
- `FK department_id → departments(id)`
- 同一社員の同一期間で `is_primary = true` は 1 件のみ（アプリケーション層で保証）

**インデックス**:

- `(employee_id, effective_to)` -特定社員の現在所属取得
- `(department_id, effective_to)` -部署所属者一覧
- `(employee_id, is_primary, effective_to)` -主務所属の現在値取得

**設計理由**:
直接 `employees.department_id` を持たず、履歴テーブルで管理する。
AD 同期で異動が発生した場合に前レコードの `effective_to` を更新し、新レコードを追加することで完全な所属履歴を保持できる。

---

## disciplines（業務種別）

| カラム          | 型             | NULL | デフォルト          | 説明                                                       |
| --------------- | -------------- | ---- | ------------------- | ---------------------------------------------------------- |
| `id`            | `UUID`         | NO   | `gen_random_uuid()` | PK                                                         |
| `code`          | `VARCHAR(10)`  | NO   | -                   | 業務種別コード（採番用）。例: `CAD`, `FEM`, `MECH`, `ELEC` |
| `name`          | `VARCHAR(100)` | NO   | -                   | 業務種別名                                                 |
| `department_id` | `UUID`         | NO   | -                   | 担当部署                                                   |
| `created_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                                           |
| `updated_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                                           |

**制約**:

- `UNIQUE (code)`
- `FK department_id → departments(id)`

**インデックス**:

- `(department_id)` -部署別業務種別一覧

**設計理由**:
業種ではなく業務プロセスの分類。`code` は業務種別コードとして使用するため不変にする。
担当部署を持つことで、プロジェクトと部署を間接的に紐づける。

---

## projects（プロジェクト）

| カラム          | 型             | NULL | デフォルト          | 説明                     |
| --------------- | -------------- | ---- | ------------------- | ------------------------ |
| `id`            | `UUID`         | NO   | `gen_random_uuid()` | PK                       |
| `name`          | `VARCHAR(200)` | NO   | -                   | プロジェクト名           |
| `status`        | `VARCHAR(20)`  | NO   | `'planning'`        | ステータス               |
| `start_date`    | `DATE`         | YES  | `NULL`              | 開始日                   |
| `end_date`      | `DATE`         | YES  | `NULL`              | 終了日                   |
| `wbs_code`      | `VARCHAR(50)`  | YES  | `NULL`              | SAP WBS コード（参照用） |
| `discipline_id` | `UUID`         | NO   | -                   | 業務種別                 |
| `manager_id`    | `UUID`         | YES  | `NULL`              | プロジェクトマネージャー |
| `created_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時         |
| `updated_at`    | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時         |

**制約**:

- `UNIQUE (wbs_code)` （NOT NULL の場合）
- `CHECK (status IN ('planning', 'active', 'completed', 'cancelled'))`
- `FK discipline_id → disciplines(id)`
- `FK manager_id → employees(id)`

**インデックス**:

- `(discipline_id)` -業務種別別プロジェクト一覧
- `(status)` -ステータスフィルタ
- `(wbs_code)` -SAP 連携キー検索

**設計理由**:
`department_id` を持たない。業務種別を経由して間接的に部署と紐づくことで、部署再編時にプロジェクトレコードを変更しなくて済む。
`wbs_code` は SAP の参照情報であり、本システムの主キーには使わない。

---

## documents（文書）

| カラム             | 型             | NULL | デフォルト          | 説明                                           |
| ------------------ | -------------- | ---- | ------------------- | ---------------------------------------------- |
| `id`               | `UUID`         | NO   | `gen_random_uuid()` | PK                                             |
| `doc_number`       | `VARCHAR(30)`  | NO   | -                   | 採番済み文書番号（登録時に凍結。以降変更不可） |
| `revision`         | `INTEGER`      | NO   | `1`                 | 文書改版番号（1始まり）                        |
| `title`            | `VARCHAR(300)` | NO   | -                   | 文書タイトル                                   |
| `file_path`        | `VARCHAR(500)` | NO   | -                   | ファイルパス（ローカル・NAS）                  |
| `author_id`        | `UUID`         | NO   | -                   | 作成者                                         |
| `doc_kind_id`      | `UUID`         | NO   | -                   | 文書種別                                       |
| `frozen_dept_code` | `VARCHAR(10)`  | NO   | -                   | 採番時の部署コード（凍結）                     |
| `confidentiality`  | `VARCHAR(20)`  | NO   | `'internal'`        | 機密グレード                                   |
| `status`           | `VARCHAR(20)`  | NO   | `'draft'`           | 承認・回覧ステータス                           |
| `project_id`       | `UUID`         | NO   | -                   | 紐づくプロジェクト                             |
| `created_at`       | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                               |
| `updated_at`       | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                               |

**制約**:

- `UNIQUE (doc_number)`
- `CHECK (revision >= 1)`
- `CHECK (confidentiality IN ('public', 'internal', 'restricted', 'confidential'))`
- `CHECK (status IN ('draft', 'under_review', 'approved', 'rejected', 'circulating', 'completed'))`
- `FK author_id → employees(id)`
- `FK doc_kind_id → document_kinds(id)`
- `FK project_id → projects(id)`

**インデックス**:

- `(doc_number)` -文書番号検索
- `(project_id)` -プロジェクト配下の文書一覧
- `(author_id)` -作成者別文書一覧
- `(doc_kind_id)` -文書種別別文書一覧
- `(confidentiality)` -機密グレードフィルタ
- `(status)` -承認・回覧タスク一覧

**設計理由**:
文書は必ずプロジェクトに紐づく。プロジェクトに属さない文書（部署規程・マニュアル等）は「部署一般」プロジェクトに登録する運用で対応する。
業務種別は `project_id → projects.discipline_id` 経由で取得するため、文書に直接持たない。
文書種別は採番・検索・監査で明示的に利用するため、`doc_kind_id` を FK として保持する。
承認・回覧の状態遷移は `status` で管理し、状態遷移ルールは `05_approval_flow.md` に従う。
`frozen_dept_code` はプロジェクトの業務種別が担当する部署コードを登録時にコピーして凍結する。
部署再編後も文書番号と帰属部署を変更する必要がなく、過去の文書番号体系を維持できる。
`PUT /documents/:id` では `frozen_dept_code`, `doc_number` を変更不可とする。
`revision` はサーバー側で管理する読み取り専用値。登録時に 1 とし、`draft` または `rejected` 状態で `PUT /documents/:id` により `title`, `file_path`, `confidentiality`, `tags` のいずれかが変更された場合にサーバー側で自動的に 1 インクリメントする。承認ルートだけを再設定する場合は変更しない。クライアントからの直接指定は受け付けない。

---

## document_kinds（文書種別）

| カラム       | 型             | NULL | デフォルト          | 説明                                         |
| ------------ | -------------- | ---- | ------------------- | -------------------------------------------- |
| `id`         | `UUID`         | NO   | `gen_random_uuid()` | PK                                           |
| `code`       | `VARCHAR(10)`  | NO   | -                   | 文書種別コード（不変）。例: `契`, `議`, `内` |
| `name`       | `VARCHAR(100)` | NO   | -                   | 文書種別名                                   |
| `seq_digits` | `INTEGER`      | NO   | -                   | 採番連番桁数（2 または 3）                   |
| `created_at` | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                             |
| `updated_at` | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                             |

**制約**:

- `UNIQUE (code)`
- `CHECK (seq_digits IN (2, 3))`

**設計理由**:
`code` は文書番号のプレフィクス構成要素として使用するため不変にする。
`seq_digits` は文書種別ごとに採番連番の桁数が異なるため（議事録は 2 桁、社内文書は 3 桁など）ここで管理する。

---

## document_registers（文書台帳）

| カラム               | 型             | NULL | デフォルト          | 説明                                                    |
| -------------------- | -------------- | ---- | ------------------- | ------------------------------------------------------- |
| `id`                 | `UUID`         | NO   | `gen_random_uuid()` | PK                                                      |
| `register_code`      | `VARCHAR(15)`  | NO   | -                   | 系列コード（不変）。例: `契設計`, `入電設`              |
| `doc_kind_id`        | `UUID`         | NO   | -                   | 文書種別                                                |
| `department_id`      | `UUID`         | NO   | -                   | 担当部署                                                |
| `file_server_root`   | `VARCHAR(300)` | NO   | -                   | ファイルサーバルートパス                                |
| `new_doc_sub_path`   | `VARCHAR(300)` | YES  | `NULL`              | 新規登録時サブパステンプレート。NULL = デフォルト不使用 |
| `doc_number_pattern` | `VARCHAR(200)` | YES  | `NULL`              | 既存文書スキャン用正規表現。NULL = スキャン対象外       |
| `created_at`         | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                                        |
| `updated_at`         | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                                        |

**制約**:

- `UNIQUE (register_code)`
- `UNIQUE (doc_kind_id, department_id)`
- `FK doc_kind_id → document_kinds(id)`
- `FK department_id → departments(id)`

**インデックス**:

- `(doc_kind_id, department_id)` -系列解決
- `(department_id)` -部署別台帳一覧

**設計理由**:
文書種別と部署の組み合わせごとに保管先・命名規則・採番ルールを定義する。
`register_code` は `{文書種別コード}{部署コード}` で構成し、文書番号のプレフィクスと一致する。
`new_doc_sub_path` が NULL の場合は新規登録時にデフォルトの保管先ルールを使用する。
`doc_number_pattern` が NULL の台帳はスキャン対象外とし、既存文書の事後マッチングは行わない。

---

## path_scan_issues（スキャン要確認）

| カラム        | 型             | NULL | デフォルト          | 説明                        |
| ------------- | -------------- | ---- | ------------------- | --------------------------- |
| `id`          | `UUID`         | NO   | `gen_random_uuid()` | PK                          |
| `document_id` | `UUID`         | YES  | `NULL`              | 対象文書。未マッチ時は NULL |
| `found_path`  | `VARCHAR(500)` | NO   | -                   | スキャンで発見したパス      |
| `issue_kind`  | `VARCHAR(20)`  | NO   | -                   | 問題種別                    |
| `resolved_at` | `TIMESTAMPTZ`  | YES  | `NULL`              | 解決日時。NULL = 未解決     |
| `created_at`  | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時            |

**制約**:

- `CHECK (issue_kind IN ('no_match', 'multiple_match'))`
- `FK document_id → documents(id)`

**インデックス**:

- `(resolved_at)` -未解決一覧（`WHERE resolved_at IS NULL`）

**設計理由**:
既存文書のパス事後設定スキャン時に自動マッチできなかったケースを積み上げ、人手による確認・解決を促す。
`no_match` は `doc_number` に一致する文書レコードが見つからなかったケース。
`multiple_match` は同一 `doc_number` に複数パスがヒットしたケース。

---

## tags（タグ）

| カラム | 型            | NULL | デフォルト          | 説明   |
| ------ | ------------- | ---- | ------------------- | ------ |
| `id`   | `UUID`        | NO   | `gen_random_uuid()` | PK     |
| `name` | `VARCHAR(50)` | NO   | -                   | タグ名 |

**制約**:

- `UNIQUE (name)`

---

## document_tags（文書タグ中間テーブル）

| カラム        | 型     | NULL | デフォルト | 説明 |
| ------------- | ------ | ---- | ---------- | ---- |
| `document_id` | `UUID` | NO   | -          | 文書 |
| `tag_id`      | `UUID` | NO   | -          | タグ |

**制約**:

- `PRIMARY KEY (document_id, tag_id)`
- `FK document_id → documents(id)`
- `FK tag_id → tags(id)`

---

## approval_steps（承認ステップ）

| カラム              | 型            | NULL | デフォルト          | 説明                               |
| ------------------- | ------------- | ---- | ------------------- | ---------------------------------- |
| `id`                | `UUID`        | NO   | `gen_random_uuid()` | PK                                 |
| `document_id`       | `UUID`        | NO   | -                   | 対象文書                           |
| `route_revision`    | `INTEGER`     | NO   | -                   | 承認ルート版（1 から始まる連番）   |
| `document_revision` | `INTEGER`     | NO   | -                   | ルート設定時の文書改版番号（凍結） |
| `step_order`        | `INTEGER`     | NO   | -                   | 承認順序（1 から始まる連番）       |
| `approver_id`       | `UUID`        | NO   | -                   | 承認者                             |
| `status`            | `VARCHAR(20)` | NO   | `'pending'`         | 処理状況                           |
| `approved_at`       | `TIMESTAMPTZ` | YES  | `NULL`              | 承認・差し戻し日時                 |
| `comment`           | `TEXT`        | YES  | `NULL`              | 承認者コメント                     |
| `created_at`        | `TIMESTAMPTZ` | NO   | `now()`             | レコード作成日時                   |

**制約**:

- `UNIQUE (document_id, route_revision, step_order)`
- `CHECK (status IN ('pending', 'approved', 'rejected'))`
- `FK document_id → documents(id)`
- `FK approver_id → employees(id)`

**インデックス**:

- `(document_id, route_revision, step_order)` -段階順承認処理
- `(approver_id, status)` -承認者の未処理タスク一覧

**設計理由**:
段階承認を `route_revision + step_order` で表現する。差し戻し再提出時は旧ルートを削除せず、`route_revision` を増やして新ルートを追加する。現在のアクティブステップは最新 `route_revision` 上で特定する。

`document_revision` はルート設定時点の `documents.revision` をスナップショットとして記録し、変更不可とする。これにより「文書改版 N に対して承認ルートが何回走ったか」を正確に追跡できる。同一 `document_revision` に対して複数の `route_revision` が存在する（差し戻し後に文書を変えずに承認者だけ変更した場合など）ことは通常の運用として想定内。

承認フローの詳細は `05_approval_flow.md` を参照。

---

## circulations（回覧）

| カラム         | 型            | NULL | デフォルト          | 説明                    |
| -------------- | ------------- | ---- | ------------------- | ----------------------- |
| `id`           | `UUID`        | NO   | `gen_random_uuid()` | PK                      |
| `document_id`  | `UUID`        | NO   | -                   | 対象文書                |
| `recipient_id` | `UUID`        | NO   | -                   | 宛先社員                |
| `confirmed_at` | `TIMESTAMPTZ` | YES  | `NULL`              | 確認日時。NULL = 未確認 |
| `created_at`   | `TIMESTAMPTZ` | NO   | `now()`             | レコード作成日時        |

**制約**:

- `UNIQUE (document_id, recipient_id)`
- `FK document_id → documents(id)`
- `FK recipient_id → employees(id)`

**インデックス**:

- `(document_id, confirmed_at)` -文書の未確認者一覧
- `(recipient_id, confirmed_at)` -社員の未確認文書一覧

**設計理由**:
宛先ごとに `confirmed_at` を記録することで、未確認者を `WHERE confirmed_at IS NULL` で即座に抽出できる。既読管理の詳細は `05_approval_flow.md` を参照。
