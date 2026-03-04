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
| `effective_from` | `DATE`        | NO   | -                   | 所属開始日                  |
| `effective_to`   | `DATE`        | YES  | `NULL`              | 所属終了日。NULL = 現在所属 |
| `created_at`     | `TIMESTAMPTZ` | NO   | `now()`             | レコード作成日時            |

**制約**:

- `FK employee_id → employees(id)`
- `FK department_id → departments(id)`

**インデックス**:

- `(employee_id, effective_to)` -特定社員の現在所属取得
- `(department_id, effective_to)` -部署所属者一覧

**設計理由**:
直接 `employees.department_id` を持たず、履歴テーブルで管理する。
AD 同期で異動が発生した場合に前レコードの `effective_to` を更新し、新レコードを追加することで完全な所属履歴を保持できる。

---

## business_categories（業務種別）

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

| カラム                 | 型             | NULL | デフォルト          | 説明                     |
| ---------------------- | -------------- | ---- | ------------------- | ------------------------ |
| `id`                   | `UUID`         | NO   | `gen_random_uuid()` | PK                       |
| `name`                 | `VARCHAR(200)` | NO   | -                   | プロジェクト名           |
| `status`               | `VARCHAR(20)`  | NO   | `'planning'`        | ステータス               |
| `start_date`           | `DATE`         | YES  | `NULL`              | 開始日                   |
| `end_date`             | `DATE`         | YES  | `NULL`              | 終了日                   |
| `wbs_code`             | `VARCHAR(50)`  | YES  | `NULL`              | SAP WBS コード（参照用） |
| `business_category_id` | `UUID`         | NO   | -                   | 業務種別                 |
| `manager_id`           | `UUID`         | YES  | `NULL`              | プロジェクトマネージャー |
| `created_at`           | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時         |
| `updated_at`           | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時         |

**制約**:

- `UNIQUE (wbs_code)` （NOT NULL の場合）
- `CHECK (status IN ('planning', 'active', 'completed', 'cancelled'))`
- `FK business_category_id → business_categories(id)`
- `FK manager_id → employees(id)`

**インデックス**:

- `(business_category_id)` -業務種別別プロジェクト一覧
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
| `title`            | `VARCHAR(300)` | NO   | -                   | 文書タイトル                                   |
| `file_path`        | `VARCHAR(500)` | NO   | -                   | ファイルパス（ローカル・NAS）                  |
| `author_id`        | `UUID`         | NO   | -                   | 作成者                                         |
| `frozen_dept_code` | `VARCHAR(10)`  | NO   | -                   | 採番時の部署コード（凍結）                     |
| `confidentiality`  | `VARCHAR(20)`  | NO   | `'internal'`        | 機密グレード                                   |
| `project_id`       | `UUID`         | NO   | -                   | 紐づくプロジェクト                             |
| `created_at`       | `TIMESTAMPTZ`  | NO   | `now()`             | レコード作成日時                               |
| `updated_at`       | `TIMESTAMPTZ`  | NO   | `now()`             | レコード更新日時                               |

**制約**:

- `UNIQUE (doc_number)`
- `CHECK (confidentiality IN ('public', 'internal', 'restricted', 'confidential'))`
- `FK author_id → employees(id)`
- `FK project_id → projects(id)`

**インデックス**:

- `(doc_number)` -文書番号検索
- `(project_id)` -プロジェクト配下の文書一覧
- `(author_id)` -作成者別文書一覧
- `(confidentiality)` -機密グレードフィルタ

**設計理由**:
文書は必ずプロジェクトに紐づく。プロジェクトに属さない文書（部署規程・マニュアル等）は「部署一般」プロジェクトに登録する運用で対応する。
業務種別は `project_id → projects.business_category_id` 経由で取得するため、文書に直接持たない。
`frozen_dept_code` はプロジェクトの業務種別が担当する部署コードを登録時にコピーして凍結する。
部署再編後も文書番号と帰属部署を変更する必要がなく、過去の文書番号体系を維持できる。
`PUT /documents/:id` では `frozen_dept_code`, `doc_number` を変更不可とする。

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

| カラム        | 型            | NULL | デフォルト          | 説明                         |
| ------------- | ------------- | ---- | ------------------- | ---------------------------- |
| `id`          | `UUID`        | NO   | `gen_random_uuid()` | PK                           |
| `document_id` | `UUID`        | NO   | -                   | 対象文書                     |
| `step_order`  | `INTEGER`     | NO   | -                   | 承認順序（1 から始まる連番） |
| `approver_id` | `UUID`        | NO   | -                   | 承認者                       |
| `status`      | `VARCHAR(20)` | NO   | `'pending'`         | 処理状況                     |
| `approved_at` | `TIMESTAMPTZ` | YES  | `NULL`              | 承認・差し戻し日時           |
| `comment`     | `TEXT`        | YES  | `NULL`              | 承認者コメント               |
| `created_at`  | `TIMESTAMPTZ` | NO   | `now()`             | レコード作成日時             |

**制約**:

- `UNIQUE (document_id, step_order)`
- `CHECK (status IN ('pending', 'approved', 'rejected'))`
- `FK document_id → documents(id)`
- `FK approver_id → employees(id)`

**インデックス**:

- `(document_id, step_order)` -段階順承認処理
- `(approver_id, status)` -承認者の未処理タスク一覧

**設計理由**:
段階承認を `step_order` で表現する。現在のアクティブステップは「直前ステップが `approved` かつ自ステップが `pending`」で特定する。承認フローの詳細は `05_approval_flow.md` を参照。

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
