# シードデータ作成計画

## Context

doc_man の全テーブルはマイグレーション済みだがデータが空。フロントエンド（Leptos CSR）の動作確認・開発にはリアルなデータが必要。`dm-db-reset` で空のDBを再作成した後に投入できるシードデータを作成する。

## 方針

- **形式**: 単一SQLファイル `scripts/seed.sql`（`BEGIN; ... COMMIT;` でトランザクション）
- **参照方式**: 各テーブルのユニーク自然キー（`departments.code`, `employees.employee_code`, `document_kinds.code` 等）をサブクエリで参照。UUID変数不要
- **実行方法**: `flake.nix` に `dm-db-seed` 関数を追加。`dm-db-reset` 後に実行する想定
- **文書番号**: `{doc_kind_code}{dept_code}-{YYMM}{seq}` フォーマット準拠（例: `内設計-2603001`）。参照: `src/services/document_numbering.rs`

## 変更対象ファイル

1. `scripts/seed.sql` — 新規作成
2. `flake.nix` — `dm-db-seed` 関数とヘルプ行を追加

## シードデータ

### Tier 1: 独立テーブル

**departments**（7件、3階層）:

```text
本社 (HQ)
├── 設計部 (設計)
│   ├── 機械設計課 (機設)
│   └── 電気設計課 (電設)
├── 品質管理部 (品管)
├── 保全部 (保全)
└── 管理部 (管理)
```

**employees**（8件、全ロール網羅）:

| name     | employee_code | role            | is_active |
| -------- | ------------- | --------------- | --------- |
| 管理太郎 | ADM001        | admin           | true      |
| 山田花子 | PM001         | project_manager | true      |
| 佐藤次郎 | PM002         | project_manager | true      |
| 鈴木一郎 | GEN001        | general         | true      |
| 田中美咲 | GEN002        | general         | true      |
| 高橋健太 | GEN003        | general         | true      |
| 中村由紀 | VW001         | viewer          | true      |
| 伊藤誠   | GEN004        | general         | **false** |

**tags**（6件）: 安全, 環境, 品質, 設計変更, 緊急, 機密

**document_kinds**（4件）:

| code | name     | seq_digits |
| ---- | -------- | ---------- |
| 内   | 社内文書 | 3          |
| 外   | 外部文書 | 3          |
| 議   | 議事録   | 2          |
| 仕   | 仕様書   | 3          |

### Tier 2

**employee_departments**（9件）: 各社員を1部署に配属（is_primary=true）。佐藤次郎のみ2部署兼務。

**disciplines**（4件）: MECH/機械→機設, ELEC/電気→電設, QA/品質管理→品管, MAINT/保全→保全

### Tier 3

**document_registers**（4件）: 内/設計, 仕/機設, 議/品管, 外/機設

**projects**（4件、各ステータス）:

| name           | status    | discipline | manager |
| -------------- | --------- | ---------- | ------- |
| 新型ポンプ開発 | active    | MECH       | PM001   |
| 制御盤更新     | active    | ELEC       | PM002   |
| 品質改善活動   | planning  | QA         | PM001   |
| 定期点検2025   | completed | MAINT      | PM002   |

### Tier 4

**documents**（15件、全6ステータス網羅）: `draft`×4, `under_review`×2, `approved`×3, `rejected`×1, `circulating`×2, `completed`×2, `restricted`機密文書×1

### Tier 5

**document_tags**（~15件）: 文書にタグを分散配置
**approval_steps**（~12件）: under_review文書に承認途中ルート、approved文書に全承認済み、rejected文書に却下コメント付き
**circulations**（~8件）: circulating文書に未確認/確認済み混在、completed文書に全確認済み

## flake.nix への追加

```bash
dm-db-seed() {
    echo "==> シードデータを投入..."
    psql -h localhost -U doc_man -d doc_man -f scripts/seed.sql
    echo "==> シード完了"
}
```

ヘルプに `dm-db-seed` 行を追加。

## 検証

1. `dm-db-reset && dm-db-seed` でエラーなく完了すること
2. 各ロールの employee_code（ADM001, PM001, GEN001, VW001）でログインできること
3. 部署ツリーが3階層で表示されること
4. 文書一覧に15件表示されること
5. 承認・回覧データが文書詳細画面に表示されること
6. `cargo test` が全パスすること（シードは開発DB専用、テストDBに影響なし）
