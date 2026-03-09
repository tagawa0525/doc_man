# 重複コード調査結果

## Context

コードベース内の不要な重複を調査し、3回以上繰り返されている重複（設計指針のルール・オブ・スリー超過）を特定する。

## 調査結果

### 1. Server: `DepartmentBrief` が3ファイルで同一定義 (3回)

```rust
pub struct DepartmentBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}
```

| ファイル                                 | 行    |
| ---------------------------------------- | ----- |
| `server/src/models/discipline.rs`        | 16-20 |
| `server/src/models/project.rs`           | 31-36 |
| `server/src/models/document_register.rs` | 27-32 |

**対策:** `server/src/models/mod.rs` に共通の `DepartmentBrief` を定義し、3ファイルから `use super::DepartmentBrief` で参照する。

### 2. Server: `DocKindBrief` が2ファイルで同一定義 (2回)

| ファイル                                 | 行    |
| ---------------------------------------- | ----- |
| `server/src/models/document.rs`          | 30-35 |
| `server/src/models/document_register.rs` | 20-25 |

**判定:** 2回なのでルール・オブ・スリー未満。現時点では許容。

### 3. Frontend: `{ id, code, name }` 構造のBrief型が5種類

| 型名                        | ファイル                    | 行      |
| --------------------------- | --------------------------- | ------- |
| `DisciplineDepartmentBrief` | `frontend/src/api/types.rs` | 129-133 |
| `DocKindBrief`              | `frontend/src/api/types.rs` | 184-188 |
| `RegisterDepartmentBrief`   | `frontend/src/api/types.rs` | 191-195 |
| `ProjectDepartmentBrief`    | `frontend/src/api/types.rs` | 229-233 |
| `DocumentDocKindBrief`      | `frontend/src/api/types.rs` | 292-296 |

全て `{ id: Uuid, code: String, name: String }` で同一構造。

**対策:** `CodeBrief` を1つ定義し、5型を置き換え。各Responseの型アノテーション（`department: CodeBrief`等）はそのまま。

### 4. Frontend: `{ id, name }` 構造のBrief型が6種類

| 型名                   | ファイル                    | 行      |
| ---------------------- | --------------------------- | ------- |
| `DepartmentSummary`    | `frontend/src/api/types.rs` | 92-95   |
| `ManagerBrief`         | `frontend/src/api/types.rs` | 244-247 |
| `AuthorBrief`          | `frontend/src/api/types.rs` | 286-289 |
| `DocumentProjectBrief` | `frontend/src/api/types.rs` | 299-302 |
| `ApproverBrief`        | `frontend/src/api/types.rs` | 346-349 |
| `RecipientBrief`       | `frontend/src/api/types.rs` | 383-386 |

全て `{ id: Uuid, name: String }` で同一構造。

**対策:** `NameBrief` を1つ定義し、6型を置き換え。

### 5. その他の重複（3回未満、またはパターン的重複）

以下は構造的な繰り返しだが、各インスタンスがドメイン固有のSQL・フィールドを持つため、マクロ等での抽象化は早すぎる共通化になる恐れがある。記録のみ。

- **Admin権限チェック** (`if user.role != Role::Admin`) - 12箇所。ミドルウェア化の候補だが、今後エンドポイントごとに権限粒度が変わる可能性がある
- **DB制約エラーハンドリング** - 4箇所。テーブル名・制約名がそれぞれ異なる
- **Frontend CRUDページパターン** - disciplines, document_kinds, document_registers が類似構造。ただしフィールド構成が異なるため汎用化は複雑

## 実装計画

### Step 1: Server側 `DepartmentBrief` の共通化

1. `server/src/models/mod.rs` に共通Brief型を追加:

   ```rust
   pub struct DepartmentBrief {
       pub id: Uuid,
       pub code: String,
       pub name: String,
   }
   ```

2. `discipline.rs`, `project.rs`, `document_register.rs` から個別定義を削除し、`use super::DepartmentBrief` に変更

### Step 2: Frontend `CodeBrief` の導入

1. `frontend/src/api/types.rs` の共通セクションに追加:

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct CodeBrief {
       pub id: Uuid,
       pub code: String,
       pub name: String,
   }
   ```

2. 5つの型を `CodeBrief` に置き換え（type alias ではなく直接置き換え）
3. 各Responseの使用箇所を更新

### Step 3: Frontend `NameBrief` の導入

1. `frontend/src/api/types.rs` の共通セクションに追加:

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct NameBrief {
       pub id: Uuid,
       pub name: String,
   }
   ```

2. 6つの型を `NameBrief` に置き換え
3. 各Responseの使用箇所を更新

### Step 4: Server側 `DocKindBrief` の共通化（ついでに）

Step 1で `mod.rs` に共通型の置き場ができるため、2回の重複だが `DocKindBrief` も共通化しておく。

## 検証

- `cargo check -p doc_man` (server)
- `cargo check -p doc-man-frontend --target wasm32-unknown-unknown` (frontend)
- 既存のintegration testがあれば実行
