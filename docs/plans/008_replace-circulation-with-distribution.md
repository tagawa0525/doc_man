# 回覧機能をメール配布機能に置き換え

## Context

現在の回覧（circulation）機能は承認フロー完了後にのみ使え、アプリ内で既読管理を行う設計になっている。実際の運用では：

- 承認不要の文書も配布したい
- 既読管理は不要で、メールで文書の所在を通知できれば十分

そこで、既存の回覧機能を削除し、承認フローとは独立した「メール配布」機能に置き換える。

## 要件

- 回覧先を選んでメールで文書の file_path ディレクトリを案内する
- 承認フローとは完全独立（どのステータスの文書でも配布可能）
- 既読管理なし
- 配布履歴を DB に保存（再配布対応：同じ文書を複数回・同じ宛先にも配布可能）
- 配布履歴はバッチ単位（配布回ごと）にグループ表示
- メール送信基盤は後回し（trait インターフェースだけ用意し stub 実装）

## 実装方針

2つの PR に分ける。PR-1 で既存の回覧を削除、PR-2 で配布機能を追加。

---

## PR-1: `refactor/remove-circulation`

回覧機能の削除とステータス enum の整理。リファクタリングのため TDD サイクルは適用しない。

### Commit 1: `refactor: remove circulation handler, model, and route`

| 操作 | ファイル                                                                                                         |
| ---- | ---------------------------------------------------------------------------------------------------------------- |
| 削除 | `server/src/handlers/circulations.rs`                                                                            |
| 削除 | `server/src/models/circulation.rs`                                                                               |
| 削除 | `server/tests/circulations.rs`                                                                                   |
| 編集 | `server/src/handlers/mod.rs` - `pub mod circulations;` 削除                                                      |
| 編集 | `server/src/models/mod.rs` - `pub mod circulation;` 削除                                                         |
| 編集 | `server/src/routes/mod.rs` - `use crate::handlers::circulations;` と 2 つの `.route(...)` 削除                   |
| 編集 | `server/src/handlers/documents.rs:419-431` - circulations 存在チェック削除（distributions ガードは PR-2 で追加） |

### Commit 2: `refactor: remove circulation from frontend`

| 操作 | ファイル                                                                                     |
| ---- | -------------------------------------------------------------------------------------------- |
| 削除 | `frontend/src/pages/documents/circulation.rs`                                                |
| 削除 | `frontend/src/api/circulations.rs`                                                           |
| 編集 | `frontend/src/pages/documents/mod.rs` - `pub mod circulation;` 削除                          |
| 編集 | `frontend/src/pages/documents/detail.rs:14` - `use ...circulation::CirculationSection;` 削除 |
| 編集 | `frontend/src/pages/documents/detail.rs:285` - `<CirculationSection .../>` 削除              |
| 編集 | `frontend/src/api/mod.rs` - `pub mod circulations;` 削除                                     |
| 編集 | `frontend/src/api/types.rs` - `CirculationResponse`, `CreateCirculationRequest` 削除         |

### Commit 3: `refactor: remove circulating/completed status`

新規マイグレーション `server/migrations/YYYYMMDDNNNNNN_remove_circulation_status.sql`:

```sql
DELETE FROM circulations;
DROP TABLE IF EXISTS circulations;

UPDATE documents SET status = 'approved', updated_at = now()
WHERE status IN ('circulating', 'completed');

ALTER TABLE documents
    DROP CONSTRAINT documents_status_check,
    ADD CONSTRAINT documents_status_check
        CHECK (status IN ('draft', 'under_review', 'approved', 'rejected'));
```

フロントエンド:

| 操作 | ファイル                                                                                       |
| ---- | ---------------------------------------------------------------------------------------------- |
| 編集 | `frontend/src/components/status_badge.rs:10-11` - `circulating`, `completed` の match arm 削除 |

### Commit 4: `refactor: update seed data`

| 操作 | ファイル                                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------------------- |
| 編集 | `server/scripts/seed.sql` - circulating/completed ステータスの文書を approved に変更、`INSERT INTO circulations` を全て削除 |

### Commit 5: `docs: remove circulation from design docs`

| 操作 | ファイル                                                                           |
| ---- | ---------------------------------------------------------------------------------- |
| 編集 | `docs/design/03_tables.md` - circulations テーブルセクション削除                   |
| 編集 | `docs/design/05_approval_flow.md` - 回覧セクション削除、ステータス遷移図・表を更新 |
| 編集 | `docs/design/06_api.md` - 回覧 API セクション削除                                  |

---

## PR-2: `feat/mail-distribution`

配布機能の新規追加。TDD サイクルに従う。

### データモデル

```sql
CREATE TABLE distributions (
    id              UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id     UUID        NOT NULL REFERENCES documents(id),
    recipient_id    UUID        NOT NULL REFERENCES employees(id),
    distributed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    distributed_by  UUID        NOT NULL REFERENCES employees(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_distributions_document_id ON distributions(document_id);
CREATE INDEX idx_distributions_recipient_id ON distributions(recipient_id);
```

- UNIQUE 制約なし（再配布可能）
- `confirmed_at` なし（既読管理なし）
- `distributed_by` で配布実行者を記録
- `distributed_at` は同一 API コールで共通値（バッチのグルーピングに使用）

### API

```text
GET  /api/v1/documents/{doc_id}/distributions  → Vec<DistributionResponse>
POST /api/v1/documents/{doc_id}/distributions  → 201 + Vec<DistributionResponse>
```

POST リクエスト: `{ "recipient_ids": ["uuid", ...] }`

レスポンス:

```json
{
  "id": "uuid",
  "recipient": { "id": "uuid", "name": "...", "email": "..." },
  "distributed_by": { "id": "uuid", "name": "..." },
  "distributed_at": "2026-03-10T14:30:00Z"
}
```

ビジネスルール:

- ステータス制限なし（どの文書でも配布可能）
- admin または project_manager のみ実行可能
- `recipient_ids` は空不可、重複は自動排除
- メール送信は MailSender trait 経由（初期実装は stub）

### メール送信インターフェース

```rust
// server/src/services/mail.rs

pub trait MailSender: Send + Sync {
    fn send_distribution(
        &self,
        recipients: &[MailRecipient],
        context: &DistributionMailContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), MailError>> + Send + '_>>;
}

pub struct DistributionMailContext {
    pub doc_number: String,
    pub title: String,
    pub directory_path: String,  // file_path の親ディレクトリ
    pub distributed_by_name: String,
}
```

- `Pin<Box<dyn Future>>` パターンで dyn dispatch 対応（`async fn` in trait は object-safe でないため）
- 初期実装は `StubMailSender`（tracing::info でログ出力のみ）
- `AppState` に `mail_sender: Arc<dyn MailSender>` を追加

### フロントエンド UI（DistributionSection）

文書詳細ページのサイドバー（column is-4）に配置。

**配布操作**（admin/project_manager のみ表示）:

- 「配布する」ボタン → 社員一覧チェックボックスで宛先選択 →「送信」
- 配布履歴がある場合、直近の配布先をチェック済みの状態でフォームを初期化（編集可能）
- 初回配布時は全て未チェック

**配布履歴**（バッチ単位でグループ表示）:

```text
配布履歴

▶ 2026-03-12 10:00　管理太郎
  ・田中美咲
  ・高橋健太

▶ 2026-03-10 14:30　管理太郎
  ・鈴木一郎
  ・田中美咲
```

- `distributed_at` でグルーピング（同一 API コールで挿入されたレコードは同じタイムスタンプ）
- 新しい配布が上に表示（DESC 順）
- 配布者名と宛先名を表示

### コミット順序

| #  | 種別     | メッセージ                                          | 内容                                                                                                                          |
| -- | -------- | --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| 1  | RED      | `test: add distribution API tests`                  | `server/tests/distributions.rs` にテスト追加（空配列返却、配布作成、draft でも 201、viewer は 403、空宛先は 400、再配布可能） |
| 2  | GREEN    | `feat: add distribution migration`                  | `server/migrations/YYYYMMDDNNNNNN_create_distributions.sql`                                                                   |
| 3  | GREEN    | `feat: add distribution handler, model, and route`  | handler, model, route 追加。テスト全パス                                                                                      |
| 4  | feat     | `feat: add mail sender trait and stub`              | `server/src/services/mail.rs` - trait + StubMailSender                                                                        |
| 5  | refactor | `refactor: integrate MailSender into AppState`      | `state.rs` に `mail_sender` 追加、`main.rs` と `build_test_app` を更新、handler から呼び出し                                  |
| 6  | RED      | `test: add distribution guard for document delete`  | 配布済み文書の削除が 409 になるテスト                                                                                         |
| 7  | GREEN    | `feat: guard document delete against distributions` | `handlers/documents.rs` に distributions 存在チェック追加                                                                     |
| 8  | feat     | `feat: add distribution frontend API client`        | `frontend/src/api/distributions.rs` + types 追加                                                                              |
| 9  | feat     | `feat: add DistributionSection component`           | `frontend/src/pages/documents/distribution.rs` - 配布先選択・バッチ単位の履歴表示、`detail.rs` に組み込み                     |
| 10 | docs     | `docs: add distribution to design docs`             | 設計ドキュメント更新                                                                                                          |
| 11 | refactor | `refactor: add distribution records to seed data`   | シードデータに配布レコード追加                                                                                                |

### 変更対象の主要ファイル

| ファイル                                       | 変更内容                                   |
| ---------------------------------------------- | ------------------------------------------ |
| `server/src/state.rs`                          | `mail_sender: Arc<dyn MailSender>` 追加    |
| `server/src/handlers/distributions.rs`         | 新規: list + create ハンドラ               |
| `server/src/models/distribution.rs`            | 新規: リクエスト/レスポンス DTO            |
| `server/src/services/mail.rs`                  | 新規: MailSender trait + StubMailSender    |
| `server/src/routes/mod.rs`                     | distribution ルート追加                    |
| `server/src/handlers/documents.rs`             | delete ガードに distributions チェック追加 |
| `server/src/main.rs`                           | StubMailSender の初期化                    |
| `server/tests/distributions.rs`                | 新規: 統合テスト                           |
| `server/tests/helpers/mod.rs`                  | `build_test_app` に StubMailSender 追加    |
| `frontend/src/pages/documents/distribution.rs` | 新規: DistributionSection コンポーネント   |
| `frontend/src/pages/documents/detail.rs`       | DistributionSection 組み込み               |
| `frontend/src/api/distributions.rs`            | 新規: API クライアント                     |

---

## 検証

### 自動テスト

```bash
cargo test                          # 全テスト通過
cargo test --test distributions     # 配布テスト
cargo test --test documents         # 文書削除ガードテスト
just lint                           # clippy 通過
just fmt-check                      # フォーマット確認
```

### 手動確認

1. `just db-reset && just db-seed` でシードデータ投入
2. `just run` + `just frontend-dev` で起動
3. 任意のステータスの文書詳細ページで「配布」セクションが表示されることを確認
4. 配布先を選択して送信 → 配布履歴に表示されることを確認
5. 同じ文書に再度配布 → 履歴が追記されることを確認
6. サーバーログに `stub: distribution mail would be sent` が出力されることを確認
