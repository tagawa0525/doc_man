-- 03_documents.sql: 文書・リビジョン
-- 35件: draft×8 / under_review×4 / approved×18 / rejected×2 / restricted×3

\echo '  → documents: 文書 (35件)'

-- === 新型ポンプ開発 (DV-2026-001) : 8文書 ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2603001', '新型ポンプ設計仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('仕機設-2603001', '駆動系材料仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '機設', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('内設計-2601001', 'ポンプ基本設計書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('外機設-2601001', '外部調達仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM document_kinds WHERE code = '外'), '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('内設計-2601002', '安全対策マニュアル',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('仕機設-2602002', '部品リスト（初版）',
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '機設', 'rejected', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('内設計-2603002', '機密設計資料',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'draft', 'restricted',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発')),
    ('手保全-2603001', 'ポンプ保守手順書',
     (SELECT id FROM employees WHERE employee_code = 'STF001'),
     (SELECT id FROM document_kinds WHERE code = '手'), '保全', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '新型ポンプ開発'));

-- === 制御盤更新 (IN-2026-001) : 5文書 ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内設計-2602001', '制御盤配線設計書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新')),
    ('仕電設-2602001', '制御盤仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '電設', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新')),
    ('外機設-2603001', '制御盤外部委託仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM document_kinds WHERE code = '外'), '機設', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新')),
    ('仕電設-2603002', 'PLC プログラム仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '電設', 'draft', 'restricted',
     (SELECT id FROM projects WHERE name = '制御盤更新')),
    ('仕電設-2602003', '電源回路設計書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '電設', 'rejected', 'internal',
     (SELECT id FROM projects WHERE name = '制御盤更新'));

-- === 計装システム刷新 (IN-2026-002) : 3文書 ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('仕機設-2601001', '計装フロー図',
     (SELECT id FROM employees WHERE employee_code = 'GEN005'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '計設', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '計装システム刷新')),
    ('内設計-2603003', 'DCS 移行計画書',
     (SELECT id FROM employees WHERE employee_code = 'GEN005'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '計装システム刷新')),
    ('手保全-2603002', '計装キャリブレーション手順書',
     (SELECT id FROM employees WHERE employee_code = 'GEN005'),
     (SELECT id FROM document_kinds WHERE code = '手'), '計設', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '計装システム刷新'));

-- === 品質改善活動 (MN-2026-001) : 4文書 ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('議品管-260301', '品質改善キックオフ議事録',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動')),
    ('議品管-260201', '品質監査報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動')),
    ('議品管-260101', '定例会議議事録 1月',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動')),
    ('内品管-2603001', '品質マニュアル改訂案',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '内'), '品管', 'under_review', 'internal',
     (SELECT id FROM projects WHERE name = '品質改善活動'));

-- === 予防保全強化 (MN-2026-002) : 2文書 ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('手保全-2603003', '予防保全チェックリスト',
     (SELECT id FROM employees WHERE employee_code = 'STF001'),
     (SELECT id FROM document_kinds WHERE code = '手'), '保全', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '予防保全強化')),
    ('内品管-2603002', '設備劣化診断報告書',
     (SELECT id FROM employees WHERE employee_code = 'STF002'),
     (SELECT id FROM document_kinds WHERE code = '内'), '品管', 'draft', 'internal',
     (SELECT id FROM projects WHERE name = '予防保全強化'));

-- === 定期点検2025 (MN-2025-001) : 4文書 (全approved) ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('外機設-2512001', '定期点検報告書',
     (SELECT id FROM employees WHERE employee_code = 'STF001'),
     (SELECT id FROM document_kinds WHERE code = '外'), '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025')),
    ('議品管-251201', '点検結果レビュー議事録',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025')),
    ('手保全-2510001', '点検作業手順書',
     (SELECT id FROM employees WHERE employee_code = 'STF002'),
     (SELECT id FROM document_kinds WHERE code = '手'), '保全', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025')),
    ('内品管-2511001', '点検品質報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '内'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '定期点検2025'));

-- === 配管更新工事 (DV-2025-001) : 4文書 (全approved) ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('仕機設-2504001', '配管設計仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '配管更新工事')),
    ('外機設-2504001', '配管材料調達仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM document_kinds WHERE code = '外'), '機設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '配管更新工事')),
    ('内設計-2509001', '配管施工完了報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '配管更新工事')),
    ('議品管-250901', '配管更新工事完了レビュー',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '議'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '配管更新工事'));

-- === 電気設備更新 (IN-2025-001) : 3文書 (全approved) ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('仕電設-2506001', 'モーター仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '仕'), '電設', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '電気設備更新')),
    ('内設計-2511001', '電気設備更新完了報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '内'), '設計', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '電気設備更新')),
    ('外機設-2506001', '変圧器調達仕様書',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM document_kinds WHERE code = '外'), '機設', 'approved', 'restricted',
     (SELECT id FROM projects WHERE name = '電気設備更新'));

-- === 品質マネジメント体制構築 (MN-2024-001) : 2文書 (全approved) ===
INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, status, confidentiality, project_id) VALUES
    ('内品管-2503001', 'QMS 構築報告書',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '内'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質マネジメント体制構築')),
    ('手保全-2503001', '内部監査手順書',
     (SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM document_kinds WHERE code = '手'), '品管', 'approved', 'internal',
     (SELECT id FROM projects WHERE name = '品質マネジメント体制構築'));

--------------------------------------------------------------------------------
-- document_revisions
-- 全35文書の Rev.0 + 改訂ありの文書は Rev.1/2 も作成
--------------------------------------------------------------------------------
\echo '  → document_revisions: リビジョン (全文書 Rev.0 + 改訂分)'

-- 全文書の Rev.0 を一括作成
INSERT INTO document_revisions (document_id, revision, file_path, created_by, effective_from)
SELECT d.id, 0, d.doc_number || '/0', d.author_id, d.created_at
FROM documents d;

-- Rev.1: ポンプ基本設計書（設計レビュー反映）
UPDATE documents SET revision = 1 WHERE doc_number = '内設計-2601001';
UPDATE document_revisions SET effective_to = now() - interval '30 days'
WHERE document_id = (SELECT id FROM documents WHERE doc_number = '内設計-2601001') AND revision = 0;
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'), 1, '内設計-2601001/1',
     '設計レビュー指摘事項の反映',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     now() - interval '30 days');

-- Rev.1: 配管設計仕様書（材料変更）
UPDATE documents SET revision = 1 WHERE doc_number = '仕機設-2504001';
UPDATE document_revisions SET effective_to = now() - interval '60 days'
WHERE document_id = (SELECT id FROM documents WHERE doc_number = '仕機設-2504001') AND revision = 0;
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'), 1, '仕機設-2504001/1',
     'SUS304 から SUS316L への材料変更',
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     now() - interval '60 days');

-- Rev.2: 配管設計仕様書（寸法修正）
UPDATE documents SET revision = 2 WHERE doc_number = '仕機設-2504001';
UPDATE document_revisions SET effective_to = now() - interval '30 days'
WHERE document_id = (SELECT id FROM documents WHERE doc_number = '仕機設-2504001') AND revision = 1;
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'), 2, '仕機設-2504001/2',
     'フランジ寸法の修正（10A→15A）',
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     now() - interval '30 days');

-- Rev.1: 点検作業手順書（手順追加）
UPDATE documents SET revision = 1 WHERE doc_number = '手保全-2510001';
UPDATE document_revisions SET effective_to = now() - interval '45 days'
WHERE document_id = (SELECT id FROM documents WHERE doc_number = '手保全-2510001') AND revision = 0;
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '手保全-2510001'), 1, '手保全-2510001/1',
     '安全確認項目の追加',
     (SELECT id FROM employees WHERE employee_code = 'STF002'),
     now() - interval '45 days');

-- Rev.1: モーター仕様書（定格変更）
UPDATE documents SET revision = 1 WHERE doc_number = '仕電設-2506001';
UPDATE document_revisions SET effective_to = now() - interval '90 days'
WHERE document_id = (SELECT id FROM documents WHERE doc_number = '仕電設-2506001') AND revision = 0;
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2506001'), 1, '仕電設-2506001/1',
     '定格出力 5.5kW → 7.5kW に変更',
     (SELECT id FROM employees WHERE employee_code = 'GEN002'),
     now() - interval '90 days');
