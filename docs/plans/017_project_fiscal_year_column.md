# プロジェクト一覧に年度カラムを追加し、ソート順を変更する

## Context

プロジェクト管理の一覧テーブルに「年度」が表示されておらず、どの年度の案件か一目でわからない。
また、デフォルトのソート順を「新しい年度が上、年度内では開始日昇順」に変更する。

## 方針

DB・サーバー・フロントエンド全体にわたる変更。
`projects.start_date` を NOT NULL 化し、年度を確実に算出可能にした上で一覧にカラム追加。

### 影響範囲

- **DB**: マイグレーションで `start_date` を NOT NULL に変更
- **サーバー**: モデル・ハンドラの型を `Option<NaiveDate>` → `NaiveDate` に変更、年度フィルタを `start_date` ベースに統一
- **フロントエンド**: 型・フォーム・一覧ページを更新

## 変更内容

### 1. DB: start_date NOT NULL 化

既存の NULL 行は `created_at::date` でバックフィルしてから ALTER。

### 2. サーバー: 型変更と年度フィルタ統一

- `ProjectRow` / `ProjectResponse` / `CreateProjectRequest` の `start_date` を非Option化
- 年度フィルタを `created_at` → `start_date` に変更（フロントエンドの年度表示と一致させる）

### 3. フロントエンド: 年度カラム追加

テーブルの先頭列（WBSコードの左）に「年度」カラムを追加する。

カラム順: **年度** | WBSコード | 名前 | マネージャー | ステータス | 文書

- `start_date` から `fiscal_year_of()` で算出した4桁西暦を表示
- ソート可能（初期ソート方向: DESC）
- フォームの開始日を必須化（バリデーション追加）

### 4. ソートロジック

- `Some("fiscal_year")` マッチアーム: 年度 → start_date → wbs_code → name で決定的にソート
- デフォルトソート: 年度 DESC → 同一年度内は start_date ASC

### 5. 既存の分野グルーピングはそのまま維持

## 検証

1. `cargo test` で全テスト合格
2. `just fmt` / `just lint` でフォーマット・リントチェック
3. `just frontend-dev` で動作確認
   - 年度カラムが表示されること
   - 年度ヘッダークリックでソート切替が動作すること
   - デフォルト状態で新しい年度が上、年度内で開始日昇順にソートされること
4. `just db-reset && just db-seed` でシードデータ投入確認
