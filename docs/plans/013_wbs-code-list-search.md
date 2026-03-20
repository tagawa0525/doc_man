# WBSコードの一覧表示・検索追加

## Context

プロジェクト一覧と文書一覧の画面にWBSコード情報が不足している。

- プロジェクト一覧: WBSコード列なし、検索フィルタなし
- 文書一覧: WBSコード検索フィルタはあるが、テーブルにWBSコード列がない

また、プロジェクト一覧APIの`wbs_code`フィルタが完全一致（`=`）で実装されており、文書一覧の部分一致（`LIKE`）と不整合。検索UIとして使うには部分一致が必要。

## 変更対象ファイル

### サーバー

| ファイル                                    | 変更内容                                                     |
| ------------------------------------------- | ------------------------------------------------------------ |
| `server/src/handlers/projects.rs` L207-210  | wbs_codeフィルタを完全一致→部分一致(LIKE)に変更              |
| `server/src/handlers/projects.rs` L112      | wbs_codeに`escape_like` + `to_lowercase`前処理を追加         |
| `server/src/models/document.rs` L32-36      | `ProjectBrief`に`wbs_code: Option<String>`追加               |
| `server/src/handlers/documents.rs` L140     | SELECT句に`p.wbs_code`追加                                   |
| `server/src/handlers/documents.rs` L197-200 | レスポンスマッピングに`wbs_code`追加                         |
| `server/src/handlers/documents.rs` L799     | `fetch_document_by_id`のSELECT句に`p.wbs_code`追加           |
| `server/src/handlers/documents.rs` L834-837 | `fetch_document_by_id`のレスポンスマッピングに`wbs_code`追加 |

### テスト

| ファイル                    | 変更内容                                           |
| --------------------------- | -------------------------------------------------- |
| `server/tests/projects.rs`  | wbs_code部分一致テスト追加                         |
| `server/tests/documents.rs` | レスポンスに`project.wbs_code`が含まれるテスト追加 |

### フロントエンド

| ファイル                                            | 変更内容                                                              |
| --------------------------------------------------- | --------------------------------------------------------------------- |
| `frontend/src/api/types.rs` L280                    | `DocumentResponse`の`project`を`NameBrief`→独自`ProjectBrief`型に変更 |
| `frontend/src/api/projects.rs` L10-17               | `ProjectListParams`に`wbs_code`フィールド追加                         |
| `frontend/src/api/projects.rs` L19-43               | `list_filtered`にwbs_codeクエリパラメータ追加                         |
| `frontend/src/pages/projects/list.rs` L319,328-334  | テーブルにWBSコード列追加                                             |
| `frontend/src/pages/projects/list.rs` L292-305      | WBSコード検索フィルタ追加                                             |
| `frontend/src/pages/documents/list.rs` L367,376-389 | テーブルにWBSコード列追加                                             |

## 実装手順（TDDサイクル）

### 1. RED: プロジェクトwbs_code部分一致テスト

`server/tests/projects.rs`にテスト追加:

- `WBS-001-AAA`と`WBS-002-BBB`のプロジェクトを作成
- `?wbs_code=001`でクエリ → `WBS-001-AAA`のみヒットすることを検証
- 大文字小文字無視: `?wbs_code=wbs-001`でも`WBS-001-AAA`がヒット

既存テスト`get_projects_with_wbs_code_filter`は`WBS-001`完全一致なのでLIKEでも通る。

### 2. GREEN: プロジェクトwbs_codeフィルタを部分一致に修正

`server/src/handlers/projects.rs`:

L112の`params.wbs_code.as_deref()`を前処理付きに変更:

```rust
let wbs_code = params
    .wbs_code
    .filter(|s| !s.is_empty())
    .map(|s| escape_like(&s).to_lowercase());
```

`push_project_filters`呼び出しを`wbs_code.as_deref()`に変更（countクエリとdataクエリの2箇所）。

`push_project_filters`内（L207-210）を変更:

```rust
if let Some(w) = wbs_code {
    qb.push(" AND LOWER(p.wbs_code) LIKE '%' || ");
    qb.push_bind(w.to_string());
    qb.push(" || '%' ESCAPE '\\'");
}
```

### 3. RED: 文書レスポンスにproject.wbs_codeが含まれるテスト

`server/tests/documents.rs`にテスト追加:

- `insert_project_with_wbs`でwbs_code付きプロジェクト作成
- そのプロジェクトに文書を作成
- 一覧APIと個別取得APIの両方で`project.wbs_code`の存在を検証

### 4. GREEN: 文書レスポンスのProjectBriefにwbs_code追加

`server/src/models/document.rs` — `ProjectBrief`にフィールド追加:

```rust
pub struct ProjectBrief {
    pub id: Uuid,
    pub name: String,
    pub wbs_code: Option<String>,
}
```

`server/src/handlers/documents.rs` — 一覧クエリ（L140）と`fetch_document_by_id`（L799）のSELECTに`p.wbs_code`追加。レスポンスマッピング2箇所に`wbs_code: r.get("wbs_code")`追加。

### 5. フロントエンド: API型とパラメータ更新

`frontend/src/api/types.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBrief {
    pub id: Uuid,
    pub name: String,
    pub wbs_code: Option<String>,
}
```

`DocumentResponse`の`project`フィールドを`NameBrief`→`ProjectBrief`に変更。`NameBrief`は他エンティティ（employees等）で使われているので変更しない。

`frontend/src/api/projects.rs` — `ProjectListParams`に`wbs_code: String`追加、`list_filtered`にクエリパラメータ追加。

### 6. フロントエンド: プロジェクト一覧にWBSコード列+検索追加

`frontend/src/pages/projects/list.rs`:

- `wbs_code`シグナル + `on_wbs_code`デバウンスハンドラ追加
- resourceの引数に`wbs_code`追加
- テーブルヘッダーに`<th>"WBSコード"</th>`追加（マネージャーの後）
- テーブルボディに`<td>{p.wbs_code.unwrap_or_default()}</td>`追加
- 検索フィルタ欄に入力フィールド追加

### 7. フロントエンド: 文書一覧にWBSコード列追加

`frontend/src/pages/documents/list.rs`:

- テーブルヘッダーに`<th>"WBSコード"</th>`追加（プロジェクトの後）
- テーブルボディに`<td>{doc.project.wbs_code.clone().unwrap_or_default()}</td>`追加
- 検索フィルタは既存のまま（変更不要）

## 検証

```bash
cargo test                        # 全テスト通過
just lint                         # clippy通過
just fmt-check                    # フォーマットOK
just run & just frontend-dev      # 手動確認
```

手動確認項目:

- プロジェクト一覧にWBSコード列が表示される
- プロジェクト一覧のWBSコード検索が部分一致で動作する
- 文書一覧にWBSコード列が表示される
- 文書一覧の既存WBSコード検索が引き続き動作する
