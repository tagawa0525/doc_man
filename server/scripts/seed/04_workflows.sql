-- 04_workflows.sql: タグ紐付け・承認ステップ・配布記録

--------------------------------------------------------------------------------
-- document_tags (25件)
--------------------------------------------------------------------------------
\echo '  → document_tags: 文書タグ紐付け (25件)'
INSERT INTO document_tags (document_id, tag_id) VALUES
    -- 新型ポンプ設計仕様書: 設計変更
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    -- 駆動系材料仕様書: 品質
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2603001'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- ポンプ基本設計書: 設計変更, 品質
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- 外部調達仕様書: 品質, 環境, コスト削減
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM tags WHERE name = '品質')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM tags WHERE name = '環境')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM tags WHERE name = 'コスト削減')),
    -- 安全対策マニュアル: 安全, 緊急, 法規制
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM tags WHERE name = '安全')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM tags WHERE name = '緊急')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM tags WHERE name = '法規制')),
    -- 機密設計資料: 機密
    ((SELECT id FROM documents WHERE doc_number = '内設計-2603002'),
     (SELECT id FROM tags WHERE name = '機密')),
    -- 制御盤配線設計書: 設計変更
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    -- 制御盤仕様書: 設計変更, 緊急
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2602001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2602001'),
     (SELECT id FROM tags WHERE name = '緊急')),
    -- PLC プログラム仕様書: 機密
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2603002'),
     (SELECT id FROM tags WHERE name = '機密')),
    -- 品質改善キックオフ議事録: 品質
    ((SELECT id FROM documents WHERE doc_number = '議品管-260301'),
     (SELECT id FROM tags WHERE name = '品質')),
    -- 定期点検報告書: 安全, 環境
    ((SELECT id FROM documents WHERE doc_number = '外機設-2512001'),
     (SELECT id FROM tags WHERE name = '安全')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2512001'),
     (SELECT id FROM tags WHERE name = '環境')),
    -- 配管設計仕様書: 設計変更, コスト削減
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     (SELECT id FROM tags WHERE name = '設計変更')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     (SELECT id FROM tags WHERE name = 'コスト削減')),
    -- 変圧器調達仕様書: 機密, 法規制
    ((SELECT id FROM documents WHERE doc_number = '外機設-2506001'),
     (SELECT id FROM tags WHERE name = '機密')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2506001'),
     (SELECT id FROM tags WHERE name = '法規制')),
    -- QMS 構築報告書: 品質, 法規制
    ((SELECT id FROM documents WHERE doc_number = '内品管-2503001'),
     (SELECT id FROM tags WHERE name = '品質')),
    ((SELECT id FROM documents WHERE doc_number = '内品管-2503001'),
     (SELECT id FROM tags WHERE name = '法規制')),
    -- 予防保全チェックリスト: 安全
    ((SELECT id FROM documents WHERE doc_number = '手保全-2603003'),
     (SELECT id FROM tags WHERE name = '安全'));

--------------------------------------------------------------------------------
-- approval_steps (22件)
--------------------------------------------------------------------------------
\echo '  → approval_steps: 承認ステップ (22件)'

-- under_review: 内設計-2602001 (3ステップ: 1承認, 2-3保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM003'),
     'approved', '2026-02-20 10:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'MGR001'),
     'pending', NULL),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2602001'),
     1, 0, 3, (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'pending', NULL);

-- under_review: 仕電設-2602001 (2ステップ: 1承認, 2保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2602001'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM003'),
     'approved', '2026-02-18 14:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2602001'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'pending', NULL);

-- under_review: 計装フロー図 (2ステップ: 全保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕計設-2601001'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'pending', NULL),
    ((SELECT id FROM documents WHERE doc_number = '仕計設-2601001'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'MGR001'),
     'pending', NULL);

-- under_review: 品質マニュアル改訂案 (2ステップ: 1承認, 2保留)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内品管-2603001'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM002'),
     'approved', '2026-03-10 09:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '内品管-2603001'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'MGR002'),
     'pending', NULL);

-- approved: 内設計-2601001 (2ステップ: 全承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     1, 1, 1, (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-01-20 09:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     1, 1, 2, (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'approved', '2026-01-21 11:00:00+09');

-- approved: 外機設-2601001 (2ステップ: 全承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-01-22 10:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'approved', '2026-01-23 15:00:00+09');

-- approved: 議品管-260201 (1ステップ)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '議品管-260201'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM002'),
     'approved', '2026-02-15 16:00:00+09');

-- approved: 配管設計仕様書 (2ステップ: 全承認)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     1, 2, 1, (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2025-05-10 09:00:00+09'),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     1, 2, 2, (SELECT id FROM employees WHERE employee_code = 'MGR001'),
     'approved', '2025-05-11 14:00:00+09');

-- rejected: 仕機設-2602002 (2ステップ: 1承認, 2却下+コメント)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at, comment) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602002'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM001'),
     'approved', '2026-02-25 09:00:00+09', NULL),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2602002'),
     1, 0, 2, (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     'rejected', '2026-02-26 14:00:00+09', '材料規格の記載が不足しています。JIS規格番号を追記してください。');

-- rejected: 電源回路設計書 (1ステップ: 却下+コメント)
INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id, status, approved_at, comment) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕電設-2602003'),
     1, 0, 1, (SELECT id FROM employees WHERE employee_code = 'PM003'),
     'rejected', '2026-02-28 11:00:00+09', '過電流保護の計算が不十分です。再計算してください。');

--------------------------------------------------------------------------------
-- distributions (12件)
--------------------------------------------------------------------------------
\echo '  → distributions: 配布記録 (12件)'

-- ポンプ基本設計書を4名に配布 (PM001が実行)
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
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     '2026-01-25 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001'));

-- 外部調達仕様書を2名に配布 (PM001が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN001'),
     '2026-01-28 14:30:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '外機設-2601001'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2026-01-28 14:30:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001'));

-- 配管設計仕様書を3名に配布 (PM001が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     (SELECT id FROM employees WHERE employee_code = 'STF001'),
     '2025-06-01 09:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     (SELECT id FROM employees WHERE employee_code = 'STF002'),
     '2025-06-01 09:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001')),
    ((SELECT id FROM documents WHERE doc_number = '仕機設-2504001'),
     (SELECT id FROM employees WHERE employee_code = 'GEN004'),
     '2025-06-01 09:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM001'));

-- QMS 構築報告書を2名に配布 (PM002が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内品管-2503001'),
     (SELECT id FROM employees WHERE employee_code = 'MGR002'),
     '2025-03-20 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM002')),
    ((SELECT id FROM documents WHERE doc_number = '内品管-2503001'),
     (SELECT id FROM employees WHERE employee_code = 'ADM001'),
     '2025-03-20 10:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'PM002'));

-- 安全対策マニュアルを全課長に配布 (ADM001が実行)
INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by) VALUES
    ((SELECT id FROM documents WHERE doc_number = '内設計-2601002'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2026-02-01 09:00:00+09',
     (SELECT id FROM employees WHERE employee_code = 'ADM001'));
