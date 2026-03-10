# shellHook の db コマンドを Justfile に一本化

## Context

shellHook 内の `dm-db-stop`, `dm-db-reset`, `dm-db-seed` は bash 関数として定義されており、fish シェルでは動作しない。一方、Justfile にはすでに `db-reset`, `db-seed`, `db-migrate` が存在し、機能が重複している。shellHook から便利コマンドを除去し、Justfile に一本化することでシェル非依存にする。

## 変更内容

### 1. Justfile に `db-stop` レシピを追加

```just
# PostgreSQL を停止
db-stop:
    pg_ctl stop -m fast
```

`db-migrate` の下あたりに配置。

### 2. flake.nix shellHook から bash 関数と案内メッセージを削除

削除対象（行 80〜105）:

- `dm-db-stop()` 関数
- `dm-db-reset()` 関数
- `dm-db-seed()` 関数
- 「利用可能なコマンド」の echo ブロック

残すもの:

- 環境変数の export（行 32〜37）
- `_dm_setup_db` 関数とその呼び出し（行 39〜78）— PostgreSQL の自動初期化・起動は shellHook の責務

### 3. shellHook に `just` コマンドの案内を追加（任意）

```bash
echo ""
echo "  just --list でコマンド一覧を表示"
echo ""
```

### 4. CLAUDE.md の Development Commands セクションを更新

`dm-db-*` の記述を `just db-*` に変更。

## 対象ファイル

- `Justfile` — `db-stop` 追加
- `flake.nix` — shellHook から bash 関数・案内を削除
- `CLAUDE.md` — コマンド名を更新

## 検証

1. `nix develop` で fish シェルに入る
2. `just --list` で db-stop が表示される
3. `just db-stop`, `just db-reset`, `just db-seed` が動作する
