# プロジェクト・文書一覧のソート機能

## Context

プロジェクト一覧と文書一覧のテーブルにソート機能がない。カラムヘッダーをクリックして昇順/降順を切り替えられるようにする。加えて、デフォルトのソート順を設定する。

両ページとも `per_page=0` で全件取得済みのため、フロントエンドのみの変更でクライアントサイドソートを実装する。APIの変更は不要。

## デフォルトソート順

- **プロジェクト一覧**: 年度（`start_date` から算出）DESC → 同一年度内は `start_date` ASC。`start_date` が None のものは末尾
- **文書一覧**: `created_at` DESC（サーバーの返却順と一致するが、クライアント側でも明示的に適用）

## 方針

- 各グループ（分野別/種別別）内でソートする（グループ構造は維持）
- ソート状態は全グループ共通
- カラムヘッダーにクリックイベントと方向インジケータ（▲/▼）を表示
- 初期表示はデフォルトソート（ユーザーがカラムをクリックすると切り替わる）

## ソート状態モデル

```rust
let sort_column = RwSignal::new(None::<&'static str>);  // None = デフォルト
let sort_ascending = RwSignal::new(true);
```

- `None`: デフォルトソート（上記の複合ソート）を適用
- `Some(col)`: そのカラムで `sort_ascending` の方向にソート
- クリック動作: 非アクティブカラム → 昇順で選択、アクティブカラム → 方向トグル

## ソート対象カラム

**プロジェクト一覧** (`frontend/src/pages/projects/list.rs`):

| カラム       | キー        | フィールド                             |
| ------------ | ----------- | -------------------------------------- |
| WBSコード    | `"wbs"`     | `wbs_code: Option<String>`             |
| 名前         | `"name"`    | `name: String`                         |
| マネージャー | `"manager"` | `manager: Option<NameBrief>` → `.name` |
| ステータス   | `"status"`  | `status: String`                       |

**文書一覧** (`frontend/src/pages/documents/list.rs`):

| カラム    | キー           | フィールド                         |
| --------- | -------------- | ---------------------------------- |
| 文書番号  | `"doc_number"` | `doc_number: String`               |
| Rev.      | `"revision"`   | `revision: i32`                    |
| タイトル  | `"title"`      | `title: String`                    |
| WBSコード | `"wbs"`        | `project.wbs_code: Option<String>` |
| 作成者    | `"author"`     | `author.name: String`              |

## 実装手順

### 1. プロジェクト一覧のソート (`frontend/src/pages/projects/list.rs`)

**a) `current_fiscal_year` を汎用化** — `fn fiscal_year_of(date: NaiveDate) -> i32` を追加（月 < 4 なら year - 1）

**b) ソート状態シグナル追加** — `sort_column`, `sort_ascending`

**c) グループテーブルのヘッダーをクリック可能に** — 各 `<th>` に `on:click`、`cursor: pointer`、アクティブカラムにインジケータ表示

**d) グループ内ソートロジック** — `groups.remove()` 後の `Vec<ProjectResponse>` に対して:

- `sort_column` が `None`: `fiscal_year_of(start_date)` DESC → `start_date` ASC
- `sort_column` が `Some(col)`: 該当フィールドで比較。`Option` 値は `None` を末尾に

### 2. 文書一覧のソート (`frontend/src/pages/documents/list.rs`)

**a) ソート状態シグナル追加**

**b) グループテーブルのヘッダーをクリック可能に**

**c) グループ内ソートロジック**:

- `sort_column` が `None`: `created_at` DESC
- `sort_column` が `Some(col)`: 該当フィールドで比較

## 修正対象ファイル

| ファイル                               | 変更内容                                                     |
| -------------------------------------- | ------------------------------------------------------------ |
| `frontend/src/pages/projects/list.rs`  | ソートシグナル、ヘッダーUI、ソートロジック、`fiscal_year_of` |
| `frontend/src/pages/documents/list.rs` | ソートシグナル、ヘッダーUI、ソートロジック                   |

## 検証方法

1. `just frontend-dev` でフロントエンドを起動
2. プロジェクト一覧: 初期表示で新しい年度のプロジェクトが上、同年度内は古い順であることを確認
3. カラムヘッダーをクリックしてソートが切り替わることを確認（各グループ内）
4. 同じカラムを再度クリックで昇順/降順がトグルされることを確認
5. 文書一覧: 初期表示で新しい文書が上に来ることを確認
6. カラムヘッダークリックで同様のソート切り替えを確認
7. `just lint` と `just fmt-check` でコード品質を確認
