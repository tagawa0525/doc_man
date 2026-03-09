# `flatten_depts` の共通化

## Context

`flatten_depts`（`DepartmentTree` のツリーをフラットな `Vec<(id, label)>` に変換する関数）が3箇所のページコンポーネント内に入れ子関数として重複している。3回目の重複に達したため共通化する。

## 現状の重複箇所

| ファイル                                      | 行    | 子ラベル書式                   |
| --------------------------------------------- | ----- | ------------------------------ |
| `frontend/src/pages/disciplines.rs:14`        | 14-29 | `"{prefix} > {name} ({code})"` |
| `frontend/src/pages/document_registers.rs:14` | 14-29 | `"{prefix} > {name}"`          |
| `frontend/src/pages/employees/form.rs:13`     | 13-28 | `"{prefix} > {name}"`          |

ルートラベルは3箇所とも `"{name} ({code})"` で同一。子ラベルにコードを含むのは `disciplines.rs` のみ。

## 方針

- `disciplines.rs` の差異は意図的でない可能性が高い（他2箇所と統一）
- `DepartmentTree` が定義されている `frontend/src/api/types.rs` に関数を配置する（型のすぐ近くにある方が発見しやすい）
- 新モジュールは作らない

## 変更内容

### 1. `frontend/src/api/types.rs` に関数を追加

```rust
/// DepartmentTree をフラットな (id, label) リストに変換する。
/// ラベルは階層を ` > ` で連結し、ルートにはコードを付与する。
pub fn flatten_dept_tree(depts: &[DepartmentTree], result: &mut Vec<(String, String)>, prefix: &str) {
    for d in depts {
        let label = if prefix.is_empty() {
            format!("{} ({})", d.name, d.code)
        } else {
            format!("{} > {}", prefix, d.name)
        };
        result.push((d.id.to_string(), label));
        let next_prefix = if prefix.is_empty() {
            d.name.clone()
        } else {
            format!("{} > {}", prefix, d.name)
        };
        flatten_dept_tree(&d.children, result, &next_prefix);
    }
}
```

### 2. 各ページから入れ子関数を削除し、共通関数を呼び出す

- `frontend/src/pages/disciplines.rs` — 入れ子 `flatten_depts` を削除、`use crate::api::types::flatten_dept_tree;` に置換
- `frontend/src/pages/document_registers.rs` — 同上
- `frontend/src/pages/employees/form.rs` — 同上

呼び出し側は `flatten_depts(...)` → `flatten_dept_tree(...)` に変更するのみ。

## 検証

```bash
cd /home/tagawa/github/doc_man && cargo build
```

ビルドが通ることを確認。UI上で部門セレクトボックスの表示が正しいことを目視確認。
