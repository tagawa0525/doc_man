-- シードデータ: 開発・動作確認用
-- 実行: just db-seed (just db-reset 後に使用)
-- ⚠️ 本番環境では絶対に実行しないこと

-- 開発DB以外での実行を防止
DO $$
BEGIN
    IF current_database() NOT IN ('doc_man', 'doc_man_test') THEN
        RAISE EXCEPTION 'シードデータは開発DB (doc_man) でのみ実行可能です。現在のDB: %', current_database();
    END IF;
END $$;

\echo 'シードデータ投入開始...'
BEGIN;

-- 既存データを削除して冪等にする（CASCADE で依存テーブルも連鎖削除）
\echo '  → 既存シードデータをクリア'
TRUNCATE
    distributions, document_tags, approval_steps, document_revisions,
    documents, projects, document_registers,
    disciplines, employee_departments, department_role_grants,
    tags, employees, departments, positions, document_kinds
CASCADE;

\echo ''
\echo '[Tier 1] マスタデータ'
\ir seed/01_master.sql

\echo ''
\echo '[Tier 2] プロジェクト'
\ir seed/02_projects.sql

\echo ''
\echo '[Tier 3] 文書・リビジョン'
\ir seed/03_documents.sql

\echo ''
\echo '[Tier 4] タグ・承認・配布'
\ir seed/04_workflows.sql

\echo ''
\echo '[Tier 5] 過去15年分の履歴データ'
\ir seed/05_historical.sql

\echo ''
COMMIT;
\echo 'シードデータ投入完了'
\echo ''
\echo '  従業員 16件 / 部署 10件 / 分野 11件'
\echo '  プロジェクト 164件 / 文書 1575件 (2011〜2024の履歴 + 2025〜2026のサンプル)'
\echo '  承認 22件 / 配布 12件'
