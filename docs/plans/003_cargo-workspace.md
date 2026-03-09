# Cargo Workspace 化リファクタリング

## Context

`src/`（バックエンド）と `frontend/` が並列しており、`src/` が暗黙的にバックエンドであることが名前から読み取れない非対称な構造になっている。また `frontend/` は独立した `Cargo.lock` を持ち、`serde` / `uuid` / `chrono` 等の共通依存のバージョンが個別管理されている。

Cargo workspace 化により、対称的な命名・依存バージョンの一元管理・将来の共有crate追加を容易にする。

## 現状 → 目標

```text
現状:                              目標:
doc_man/                           doc_man/
├── Cargo.toml  ← backend pkg     ├── Cargo.toml  ← workspace
├── Cargo.lock                     ├── Cargo.lock  ← 統一
├── src/        ← backend         ├── server/
├── frontend/                      │   ├── Cargo.toml
│   ├── Cargo.toml                │   ├── src/
│   ├── Cargo.lock  ← 別lock     │   ├── migrations/
│   └── src/                      │   ├── tests/
├── migrations/                    │   └── scripts/
├── tests/                         │        └── seed.sql
├── scripts/                       ├── frontend/
│   ├── seed.sql                  │   ├── Cargo.toml  ← workspace dep
│   └── fix-markdown-lint.py      │   ├── Trunk.toml
├── docs/                          │   └── src/
└── flake.nix                      ├── scripts/
                                      │   └── fix-markdown-lint.py
                                      ├── docs/
                                      └── flake.nix
```

## 変更不要なファイル（理由）

| ファイル               | 理由                                                                                                                                                                  |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `server/src/lib.rs`    | `sqlx::migrate!()` は `CARGO_MANIFEST_DIR` 相対 → `server/migrations/` を自動検出。`frontend/dist` のデフォルトパスも workspace root からの相対パスとして引き続き有効 |
| `server/src/main.rs`   | `use doc_man::` のインポートは crate 名不変のため動作                                                                                                                 |
| `server/tests/**/*.rs` | `doc_man::MIGRATOR` / `doc_man::app_with_state` の参照は crate 名不変のため動作                                                                                       |
| `frontend/Trunk.toml`  | Trunk は `frontend/` から実行、すべて相対パス                                                                                                                         |
| `frontend/index.html`  | アセット参照はすべて相対                                                                                                                                              |

## 実装手順

### Commit 1: ファイル移動

```sh
mkdir -p server/scripts
git mv src/ server/src/
git mv migrations/ server/migrations/
git mv tests/ server/tests/
git mv scripts/seed.sql server/scripts/seed.sql
```

純粋なファイル移動のみ。Cargo.toml は次コミットで更新する（git の rename tracking を最適化するため）。

### Commit 2: Cargo workspace 構成

**Root `Cargo.toml`** — 全依存バージョンを集約した workspace manifest:

```toml
[workspace]
members = ["server", "frontend"]
default-members = ["server"]
resolver = "3"

[workspace.dependencies]
# === 共通 ===
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0.149"
uuid = { version = "1.22.0", features = ["v4", "serde"] }
chrono = { version = "0.4.44", features = ["serde"] }

# === server ===
axum = { version = "0.8.8", features = ["macros"] }
tower = { version = "0.5.3", features = ["util"] }
tower-http = { version = "0.6.8", features = ["trace", "cors", "fs"] }
tokio = { version = "1.50.0", features = ["full"] }
sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
thiserror = "2.0.18"
tracing = "0.1.44"
tracing-subscriber = { version = "0.3.22", features = ["env-filter", "fmt"] }

# === frontend ===
leptos = { version = "0.8.17", features = ["csr"] }
leptos_router = "0.8.12"
leptos_meta = "0.8.6"
gloo-net = { version = "0.6.0", features = ["json"] }
gloo-storage = "0.3.0"
gloo-timers = { version = "0.3.0", features = ["futures"] }
wasm-bindgen = "0.2.114"
wasm-bindgen-futures = "0.4.64"
web-sys = { version = "0.3.91", features = ["Window", "Location", "Storage", "HtmlInputElement", "HtmlSelectElement", "console"] }
console_error_panic_hook = "0.1.7"
```

- `default-members = ["server"]` — `cargo build` / `cargo test` がサーバーのみ対象（frontend は WASM ターゲットなので native build 不可）
- `resolver = "3"` — edition 2024 が要求
- 全依存のバージョンを root に集約し、メンバーでは `.workspace = true` のみ

**`server/Cargo.toml`** — 新規作成（旧 root から移行、バージョンは workspace 参照）:

```toml
[package]
name = "doc_man"
version = "0.1.0"
edition = "2024"

[lib]
name = "doc_man"
path = "src/lib.rs"

[dependencies]
axum.workspace = true
tower.workspace = true
tower-http.workspace = true
tokio.workspace = true
sqlx.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
```

- package name は `doc_man` のまま（テストの `use doc_man::` を壊さない）

**`frontend/Cargo.toml`** — 全依存を workspace 参照に変更:

```toml
[dependencies]
leptos.workspace = true
leptos_router.workspace = true
leptos_meta.workspace = true
gloo-net.workspace = true
gloo-storage.workspace = true
gloo-timers.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid = { workspace = true, features = ["js"] }
chrono.workspace = true
wasm-bindgen.workspace = true
wasm-bindgen-futures.workspace = true
web-sys.workspace = true
console_error_panic_hook.workspace = true
```

- `uuid` は workspace の `["v4", "serde"]` に frontend 固有の `["js"]` をマージ

**削除**: `frontend/Cargo.lock`（workspace の統一 lock に統合）

### Commit 3: flake.nix パス更新

`flake.nix` 内の 3 箇所のパスを更新:

| 行 | 変更前                                 | 変更後                                        |
| -- | -------------------------------------- | --------------------------------------------- |
| 73 | `sqlx migrate run --source migrations` | `sqlx migrate run --source server/migrations` |
| 87 | `sqlx migrate run --source migrations` | `sqlx migrate run --source server/migrations` |
| 93 | `-f scripts/seed.sql`                  | `-f server/scripts/seed.sql`                  |

ヘルプテキスト（99行目）も更新:
`cargo run` → `cargo run -p doc_man`

### Commit 4: .gitignore 更新

`/frontend/target` 行を削除（workspace では root の `/target` に統合されるため不要）。

## 検証手順

workspace root から実行:

1. `cargo build` — サーバーがビルドできること
2. `cargo test` — 14 件の統合テストが全パスすること
3. `cargo clippy` — 新規警告がないこと
4. `cd frontend && trunk build` — WASM バンドルが `frontend/dist/` に生成されること
5. `cargo run` — サーバーが起動し `frontend/dist/` を配信すること
6. `direnv reload` → `dm-db-reset` / `dm-db-seed` — flake.nix のパスが正しいこと

## 対象ファイル一覧

| ファイル              | 操作                                 |
| --------------------- | ------------------------------------ |
| `src/`                | `git mv` → `server/src/`             |
| `migrations/`         | `git mv` → `server/migrations/`      |
| `tests/`              | `git mv` → `server/tests/`           |
| `scripts/seed.sql`    | `git mv` → `server/scripts/seed.sql` |
| `Cargo.toml` (root)   | 全面書き換え（workspace manifest）   |
| `server/Cargo.toml`   | 新規作成                             |
| `frontend/Cargo.toml` | 全依存を workspace 参照に変更        |
| `frontend/Cargo.lock` | 削除                                 |
| `flake.nix`           | パス 3 箇所 + ヘルプテキスト更新     |
| `.gitignore`          | `/frontend/target` 行削除            |
