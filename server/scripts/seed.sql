-- シードデータ: 開発・動作確認用
-- 実行: dm-db-seed (dm-db-reset 後に使用)

BEGIN;

--------------------------------------------------------------------------------
-- Tier 1: 独立テーブル
--------------------------------------------------------------------------------

-- positions（マイグレーションで初期データ投入済み、冪等に追加）
INSERT INTO positions (name, default_role, sort_order) VALUES
    ('社長',   'admin',           1),
    ('部長',   'admin',           2),
    ('課長',   'admin',           3),
    ('総合職', 'project_manager', 4),
    ('一般職', 'general',         5),
    ('嘱託',   'viewer',          6),
    ('派遣',   'viewer',          7)
ON CONFLICT (name) DO NOTHING;

-- departments (7件、3階層)
INSERT INTO departments (code, name, effective_from) VALUES
    ('HQ',   '本社',       '2020-01-01');
INSERT INTO departments (code, name, parent_id, effective_from) VALUES
    ('設計', '設計部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('品管', '品質管理部', (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('保全', '保全部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('管理', '管理部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01');
INSERT INTO departments (code, name, parent_id, effective_from) VALUES
    ('機設', '機械設計課', (SELECT id FROM departments WHERE code = '設計'), '2020-01-01'),
    ('電設', '電気設計課', (SELECT id FROM departments WHERE code = '設計'), '2020-01-01');

-- employees (8件、全ロール網羅)
-- role は個人上書き（NULL = 職位のデフォルトを使用）
INSERT INTO employees (name, employee_code, email, role, position_id, is_active) VALUES
    ('管理太郎', 'ADM001', 'kanri@example.com',    'admin',
     (SELECT id FROM positions WHERE name = '社長'),   true),
    ('山田花子', 'PM001',  'yamada@example.com',   NULL,
     (SELECT id FROM positions WHERE name = '課長'),   true),
    ('佐藤次郎', 'PM002',  'sato@example.com',    NULL,
     (SELECT id FROM positions WHERE name = '課長'),   true),
    ('鈴木一郎', 'GEN001', 'suzuki@example.com',  NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('田中美咲', 'GEN002', 'tanaka@example.com',  NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('高橋健太', 'GEN003', 'takahashi@example.com',NULL,
     (SELECT id FROM positions WHERE name = '一般職'), true),
    ('中村由紀', 'VW001',  'nakamura@example.com', NULL,
     (SELECT id FROM positions WHERE name = '嘱託'),   true),
    ('伊藤誠',   'GEN004', 'ito@example.com',     NULL,
     (SELECT id FROM positions WHERE name = '一般職'), false);

-- department_role_grants（管理部に admin を付与）
INSERT INTO department_role_grants (department_id, role) VALUES
    ((SELECT id FROM departments WHERE code = '管理'), 'admin');

-- tags (6件)
INSERT INTO tags (name) VALUES
    ('安全'), ('環境'), ('品質'), ('設計変更'), ('緊急'), ('機密');

-- document_kinds (4件)
INSERT INTO document_kinds (code, name, seq_digits) VALUES
    ('内', '社内文書', 3),
    ('外', '外部文書', 3),
    ('議', '議事録',   2),
    ('仕', '仕様書',   3);

--------------------------------------------------------------------------------
-- Tier 2
--------------------------------------------------------------------------------

-- employee_departments (9件: 佐藤次郎のみ2部署兼務)
INSERT INTO employee_departments (employee_id, department_id, is_primary, effective_from) VALUES
    ((SELECT id FROM employees WHERE employee_code = 'ADM001'),
     (SELECT id FROM departments WHERE code = '管理'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'PM001'),
     (SELECT id FROM departments WHERE code = '設計'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'PM002'),
     (SELECT id FROM departments WHERE code = '品管'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'PM002'),
     (SELECT id FROM departments WHERE code = '保全'), false, '2022-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM departments WHERE code = '機設'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM departments WHERE code = '電設'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM departments WHERE code = '品管'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'VW001'),
     (SELECT id FROM departments WHERE code = '管理'), true, '2023-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM departments WHERE code = '保全'), true, '2021-04-01');

-- disciplines (4件)
INSERT INTO disciplines (code, name, department_id) VALUES
    ('MECH',  '機械',     (SELECT id FROM departments WHERE code = '機設')),
    ('ELEC',  '電気',     (SELECT id FROM departments WHERE code = '電設')),
    ('QA',    '品質管理', (SELECT id FROM departments WHERE code = '品管')),
    ('MAINT', '保全',     (SELECT id FROM departments WHERE code = '保全'));

--------------------------------------------------------------------------------
-- Tier 3
--------------------------------------------------------------------------------

-- document_registers (4件)
INSERT INTO document_registers (register_code, doc_kind_id, department_id, file_server_root) VALUES
    ('内設計',
     (SELECT id FROM document_kinds WHERE code = '内'),
     (SELECT id FROM departments WHERE code = '設計'),
     '/files/internal/design'),
    ('仕機設',
     (SELECT id FROM document_kinds WHERE code = '仕'),
     (SELECT id FROM departments WHERE code = '機設'),
     '/files/specs/mech'),
    ('議品管',
     (SELECT id FROM document_kinds WHERE code = '議'),
     (SELECT id FROM departments WHERE code = '品管'),
     '/files/minutes/qa'),
    ('外機設',
     (SELECT id FROM document_kinds WHERE code = '外'),
     (SELECT id FROM departments WHERE code = '機設'),
     '/files/external/mech');

-- projects (4件、各ステータス)
INSERT INTO projects (name, status, start_date, end_date, wbs_code, discipline_id, manager_id) VALUES
    ('新型ポンプ開発', 'active', '2026-01-15', NULL, 'DV-2026-001',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ('制御盤更新', 'active', '2026-02-01', NULL, 'IN-2026-001',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM002')),
    ('品質改善活動', 'planning', NULL, NULL, 'MN-2026-001',
     (SELECT id FROM disciplines WHERE code = 'QA'),
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ('定期点検2025', 'completed', '2025-10-01', '2025-12-20', 'MN-2025-001',
     (SELECT id FROM disciplines WHERE code = 'MAINT'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'));

--------------------------------------------------------------------------------
-- Tier 4: documents (15件)
--------------------------------------------------------------------------------

-- draft × 4
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2603001', '新型ポンプ設計仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'),
     '設計', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('仕機設-2603001', '駆動系材料仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '仕'),
     '機設', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('議品管-260301', '品質改善キックオフ議事録',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'),
     '品管', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動')),
    ('外機設-2603001', '外部委託仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '外'),
     '機設', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新'));

-- under_review × 2
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2602001', '制御盤配線設計書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '内'),
     '設計', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新')),
    ('仕機設-2602001', '制御盤仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '仕'),
     '機設', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新'));

-- approved × 3
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2601001', 'ポンプ基本設計書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'),
     '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('外機設-2601001', '外部調達仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '外'),
     '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('議品管-260201', '品質監査報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'),
     '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動'));

-- rejected × 1
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('仕機設-2602002', '部品リスト（初版）',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '仕'),
     '機設', 'rejected', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発'));

-- approved × 4 (元circulating/completed)
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2601002', '安全対策マニュアル',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'),
     '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('議品管-260101', '定例会議議事録 1月',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'),
     '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動')),
    ('外機設-2512001', '定期点検報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '外'),
     '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025')),
    ('議品管-251201', '点検結果レビュー議事録',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'),
     '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025'));

-- restricted 機密文書 × 1 (draft)
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2603002', '機密設計資料',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'),
     '設計', 'draft', 'restricted',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発'));

-- document_revisions (全15文書の Rev.0)
INSERT INTO document_revisions (document_id, revision, file_path, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603001'), 0, '内設計-2603001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '内設計-2603001')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2603001'), 0, '仕機設-2603001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'), (SELECT created_at FROM documents WHERE doc_number = '仕機設-2603001')),
    ((SELECT id FROM documents WHERE doc_number = '議品管-260301'), 0, '議品管-260301/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'), (SELECT created_at FROM documents WHERE doc_number = '議品管-260301')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2603001'), 0, '外機設-2603001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '外機設-2603001')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'), 0, '内設計-2602001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'), (SELECT created_at FROM documents WHERE doc_number = '内設計-2602001')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602001'), 0, '仕機設-2602001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '仕機設-2602001')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'), 0, '内設計-2601001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '内設計-2601001')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'), 0, '外機設-2601001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'), (SELECT created_at FROM documents WHERE doc_number = '外機設-2601001')),
    ((SELECT id FROM documents WHERE doc_number = '議品管-260201'), 0, '議品管-260201/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'), (SELECT created_at FROM documents WHERE doc_number = '議品管-260201')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602002'), 0, '仕機設-2602002/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '仕機設-2602002')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'), 0, '内設計-2601002/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '内設計-2601002')),
    ((SELECT id FROM documents WHERE doc_number = '議品管-260101'), 0, '議品管-260101/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'), (SELECT created_at FROM documents WHERE doc_number = '議品管-260101')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2512001'), 0, '外機設-2512001/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'), (SELECT created_at FROM documents WHERE doc_number = '外機設-2512001')),
    ((SELECT id FROM documents WHERE doc_number = '議品管-251201'), 0, '議品管-251201/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'), (SELECT created_at FROM documents WHERE doc_number = '議品管-251201')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603002'), 0, '内設計-2603002/0',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'), (SELECT created_at FROM documents WHERE doc_number = '内設計-2603002'));

--------------------------------------------------------------------------------
-- Tier 5
--------------------------------------------------------------------------------

-- document_tags (~15件)
INSERT INTO document_tags (document_id, tag_id) VALUES
    -- 新型ポンプ設計仕様書: 設計変更
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    -- 駆動系材料仕様書: 品質
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2603001'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- 品質改善キックオフ議事録: 品質
    ((SELECT id FROM documents WHERE doc_number = '議品管-260301'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- 制御盤配線設計書: 設計変更
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    -- 制御盤仕様書: 設計変更, 緊急
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602001'),
     (SELECT id FROM tags WHERE name = '緊急')),
    -- ポンプ基本設計書: 設計変更, 品質
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- 外部調達仕様書: 品質, 環境
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM tags WHERE name = '品質')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM tags WHERE name = '環境')),
    -- 安全対策マニュアル: 安全, 緊急
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM tags WHERE name = '安全')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM tags WHERE name = '緊急')),
    -- 定期点検報告書: 安全, 環境
    ((SELECT id FROM documents WHERE doc_number = '外機設-2512001'),
     (SELECT id FROM tags WHERE name = '安全')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2512001'),
     (SELECT id FROM tags WHERE name = '環境')),
    -- 機密設計資料: 機密
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603002'),
     (SELECT id FROM tags WHERE name = '機密'));

-- approval_steps (~12件)
-- under_review: 内設計-2602001 (3ステップ: 1承認済み, 2-3保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-02-20 10:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 1, 2,
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     'pending', NULL),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 1, 3,
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'pending', NULL);

-- under_review: 仕機設-2602001 (2ステップ: 1承認済み, 2保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602001'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-02-18 14:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602001'),
     1, 1, 2,
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'pending', NULL);

-- approved: 内設計-2601001 (2ステップ: 全承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-01-20 09:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     1, 1, 2,
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'approved', '2026-01-21 11:00:00+09');

-- approved: 外機設-2601001 (2ステップ: 全承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     'approved', '2026-01-22 10:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     1, 1, 2,
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'approved', '2026-01-23 15:00:00+09');

-- approved: 議品管-260201 (1ステップ: 承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '議品管-260201'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-02-15 16:00:00+09');

-- rejected: 仕機設-2602002 (2ステップ: 1承認, 2却下+コメント)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at, comment) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602002'),
     1, 1, 1,
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-02-25 09:00:00+09', NULL),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602002'),
     1, 1, 2,
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'rejected', '2026-02-26 14:00:00+09', '材料規格の記載が不足しています。JIS規格番号を追記してください。');

-- distributions (approved文書2件に配布: バッチ2回)
-- バッチ1: ポンプ基本設計書を3名に配布 (PM001が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     '2026-01-25 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     '2026-01-25 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     '2026-01-25 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001'));

-- バッチ2: 外部調達仕様書を2名に配布 (PM002が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     '2026-01-28 14:30:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM002')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2026-01-28 14:30:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM002'));

COMMIT;
