# 文書ページ構成のリファクタリング

## Context

文書管理の画面構成に問題がある:

1. **一覧に削除ボタンがある** — 削除は「編集」操作の一部。一覧から直接削除できるのは誤操作リスクが高い
2. **編集ページが薄すぎる** — 変更できるのは4項目（タイトル、ファイルパス、機密区分、タグ）だけ。詳細と分離する意義が弱い
3. **プロジェクト一覧にも同じ問題がある** — 削除ボタンが一覧に露出

## 方針

### 文書ページ

| Before                              | After                                      |
| ----------------------------------- | ------------------------------------------ |
| `/documents` 一覧 + 削除            | `/documents` 一覧のみ（削除なし）          |
| `/documents/new` 作成フォーム       | `/documents/new` 作成フォーム（変更なし）  |
| `/documents/{id}` 閲覧のみ          | `/documents/{id}` 閲覧 + 編集トグル + 削除 |
| `/documents/{id}/edit` 編集フォーム | **廃止**                                   |

**詳細ページに編集機能を統合する（トグルモード）:**

- デフォルトは閲覧モード（現状と同じ読み取り専用テーブル）
- 「編集」ボタンで編集モードに切替 → 編集可能な4項目がinput/selectに変わる
- 「保存」「キャンセル」ボタンが表示される。保存後は閲覧モードに戻りデータ再読込
- 「削除」ボタンを追加（Admin のみ、確認モーダル付き）
- 右サイドバー（承認・回覧）は両モードで不変

**作成ページは維持:**

- doc_kind と project は作成時必須・作成後変更不可なので、作成専用ページが必要
- `form.rs` → `create.rs` にリネームし、編集ロジックを除去

### プロジェクトページ

- 一覧から削除ボタンを除去
- 編集フォームページ（`/projects/{id}`）に削除ボタンを追加

## 変更ファイル

### Phase 1: 文書一覧 — 削除を除去

**`frontend/src/pages/documents/list.rs`**

- `delete_target` シグナル、`do_delete` クロージャ、`ConfirmModal` を削除
- `refresh` シグナルを削除（削除がなくなれば不要）
- テーブルの「操作」列を削除（タイトルがリンクなので閲覧アイコンも不要）
- `ConfirmModal` と `ToastContext` の import を削除

### Phase 2: 文書作成 — form.rs を create.rs に分離

**`frontend/src/pages/documents/form.rs` → `frontend/src/pages/documents/create.rs`**

- ファイルをリネーム
- コンポーネント名を `DocumentFormPage` → `DocumentCreatePage` に変更
- 編集モード関連コードを除去: `is_edit`, `doc_id`, `loaded`, `_load` Effect, update 分岐
- 作成成功後のリダイレクト先を `/documents` → `/documents/{id}`（新規文書の詳細）に変更

**`frontend/src/pages/documents/mod.rs`**

- `pub mod form;` → `pub mod create;`

**`frontend/src/main.rs`**

- `use pages::documents::form::DocumentFormPage` → `use pages::documents::create::DocumentCreatePage`
- `/documents/new` ルートを `DocumentCreatePage` に変更
- `/documents/:id/edit` ルートを削除

### Phase 3: 文書詳細 — 編集トグル + 削除を統合

**`frontend/src/pages/documents/detail.rs`**

現在の閲覧専用ページに以下を追加:

- **シグナル追加**: `editing: RwSignal<bool>`, `form_title`, `form_file_path`, `form_confidentiality`, `form_tags`, `saving`, `delete_target`
- **ヘッダーボタン変更**:
  - 閲覧モード: 「編集」「一覧に戻る」（現状に近い）
  - 編集モード: 「保存」「キャンセル」
- **テーブル内容**: `editing` に応じて各行の `<td>` を切替
  - 閲覧モード: テキスト表示（現状通り）
  - 編集モード: タイトル/ファイルパス → `<input>`, 機密区分 → `<select>`, タグ → `<input>`
  - 編集不可項目（文書番号、ステータス、リビジョン等）は常にテキスト
- **保存ハンドラ**: `api::documents::update()` を呼出 → 成功で `editing=false` + `refresh` で再読込
- **削除セクション**: テーブル下部に Admin のみ表示。`ConfirmModal` 付き。削除成功で `/documents` にリダイレクト

import 追加: `UpdateDocumentRequest`, `web_sys::HtmlInputElement`, `ConfirmModal`, `ToastContext`

### Phase 4: プロジェクト — 同パターン適用

**`frontend/src/pages/projects/list.rs`**

- 文書一覧と同じ: `delete_target`, `do_delete`, `ConfirmModal`, 削除ボタン、`refresh` を除去
- 「操作」列を除去（名前がリンクなので十分）

**`frontend/src/pages/projects/form.rs`**

- 編集モード（`is_edit()`）時のみ削除ボタンを表示
- Admin のみ。`ConfirmModal` 付き
- 削除成功で `/projects` にリダイレクト
- import 追加: `ConfirmModal`, `AuthContext`

## 検証方法

各 Phase 完了後:

1. `cargo build -p doc-man-frontend` でビルドエラーがないこと
2. `trunk serve` でフロントエンド起動
3. ブラウザで以下を確認:
   - 文書一覧: 削除ボタンがない、タイトルクリックで詳細へ遷移
   - 文書作成: `/documents/new` でフォーム表示、作成後に詳細ページへ遷移
   - 文書詳細: 閲覧モードで全情報表示、「編集」で入力切替、「保存」で更新、「削除」（Admin）で確認後削除
   - `/documents/{id}/edit` が 404 になること
   - プロジェクト一覧: 削除ボタンがない
   - プロジェクト編集: Admin で削除ボタン表示、確認後削除できること
   - 承認・回覧サイドバーが正常動作すること
