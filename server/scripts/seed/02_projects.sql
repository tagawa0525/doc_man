-- 02_projects.sql: 専門分野・文書台帳・プロジェクト

--------------------------------------------------------------------------------
-- disciplines (11件: 各部署2〜3分野)
--------------------------------------------------------------------------------
\echo '  → disciplines: 専門分野 (11件)'
INSERT INTO disciplines (code, name, department_id) VALUES
    -- 機械設計課 (3分野)
    ('MECH',  '機械',     (SELECT id FROM departments WHERE code = '機設')),
    ('PIPE',  '配管',     (SELECT id FROM departments WHERE code = '機設')),
    ('ROTR',  '回転機',   (SELECT id FROM departments WHERE code = '機設')),
    -- 電気設計課 (2分野)
    ('ELEC',  '電気',     (SELECT id FROM departments WHERE code = '電設')),
    ('CTRL',  '制御',     (SELECT id FROM departments WHERE code = '電設')),
    -- 計装設計課 (2分野)
    ('INST',  '計装',     (SELECT id FROM departments WHERE code = '計設')),
    ('PROC',  'プロセス', (SELECT id FROM departments WHERE code = '計設')),
    -- 品質管理部 (2分野)
    ('QA',    '品質管理', (SELECT id FROM departments WHERE code = '品管')),
    ('INSP',  '検査',     (SELECT id FROM departments WHERE code = '品管')),
    -- 保全部 (2分野)
    ('MAINT', '保全',     (SELECT id FROM departments WHERE code = '保全')),
    ('DIAG',  '診断',     (SELECT id FROM departments WHERE code = '保全'));

--------------------------------------------------------------------------------
-- document_registers (7件)
--------------------------------------------------------------------------------
\echo '  → document_registers: 文書台帳 (7件)'
INSERT INTO document_registers (register_code, doc_kind_id, department_id, file_server_root) VALUES
    ('内設計',
     (SELECT id FROM document_kinds WHERE code = '内'),
     (SELECT id FROM departments WHERE code = '設計'),
     '/files/internal/design'),
    ('仕機設',
     (SELECT id FROM document_kinds WHERE code = '仕'),
     (SELECT id FROM departments WHERE code = '機設'),
     '/files/specs/mech'),
    ('仕電設',
     (SELECT id FROM document_kinds WHERE code = '仕'),
     (SELECT id FROM departments WHERE code = '電設'),
     '/files/specs/elec'),
    ('議品管',
     (SELECT id FROM document_kinds WHERE code = '議'),
     (SELECT id FROM departments WHERE code = '品管'),
     '/files/minutes/qa'),
    ('外機設',
     (SELECT id FROM document_kinds WHERE code = '外'),
     (SELECT id FROM departments WHERE code = '機設'),
     '/files/external/mech'),
    ('手保全',
     (SELECT id FROM document_kinds WHERE code = '手'),
     (SELECT id FROM departments WHERE code = '保全'),
     '/files/procedures/maint'),
    ('内品管',
     (SELECT id FROM document_kinds WHERE code = '内'),
     (SELECT id FROM departments WHERE code = '品管'),
     '/files/internal/qa');

--------------------------------------------------------------------------------
-- projects (20件: 複数年度・ステータス・分野、同一年度・分野に複数件)
--------------------------------------------------------------------------------
\echo '  → projects: プロジェクト (20件)'
INSERT INTO projects (name, status, start_date, end_date, wbs_code, discipline_id, manager_id, created_at) VALUES
    -- ── 2025年度 (2025-04〜2026-03) ──
    -- MECH: 3件（ソート確認用）
    ('新型ポンプ開発', 'active', '2026-01-15', NULL, 'DV-2025-003',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2026-01-15'),
    ('圧縮機改修', 'active', '2025-09-01', NULL, 'DV-2025-002',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2025-09-01'),
    ('配管更新工事', 'completed', '2025-04-01', '2025-09-30', 'DV-2025-001',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2025-04-01'),
    -- ELEC: 2件
    ('制御盤更新', 'active', '2026-02-01', NULL, 'IN-2025-002',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2026-02-01'),
    ('電気設備更新', 'completed', '2025-06-01', '2025-11-30', 'IN-2025-001',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2025-06-01'),
    -- INST: 2件
    ('計装システム刷新', 'active', '2026-01-10', NULL, 'IN-2025-004',
     (SELECT id FROM disciplines WHERE code = 'INST'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2026-01-10'),
    ('省エネ診断', 'cancelled', '2025-07-01', NULL, 'IN-2025-003',
     (SELECT id FROM disciplines WHERE code = 'INST'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2025-07-01'),
    -- QA: 1件
    ('品質改善活動', 'planning', '2026-01-01', NULL, 'MN-2025-002',
     (SELECT id FROM disciplines WHERE code = 'QA'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2026-01-01'),
    -- MAINT: 2件
    ('予防保全強化', 'planning', '2026-01-01', NULL, 'MN-2025-003',
     (SELECT id FROM disciplines WHERE code = 'MAINT'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2026-01-01'),
    ('定期点検2025', 'completed', '2025-10-01', '2025-12-20', 'MN-2025-001',
     (SELECT id FROM disciplines WHERE code = 'MAINT'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2025-10-01'),

    -- ── 2024年度 (2024-04〜2025-03) ──
    -- MECH: 2件
    ('ポンプ基礎調査', 'completed', '2024-10-01', '2025-02-28', 'DV-2024-002',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2024-10-01'),
    ('熱交換器点検', 'completed', '2024-04-15', '2024-09-30', 'DV-2024-001',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2024-04-15'),
    -- ELEC: 2件
    ('受変電設備更新', 'completed', '2024-11-01', '2025-03-31', 'IN-2024-002',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2024-11-01'),
    ('照明LED化', 'completed', '2024-06-01', '2024-12-31', 'IN-2024-001',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2024-06-01'),
    -- QA: 1件
    ('品質マネジメント体制構築', 'completed', '2024-04-01', '2025-03-31', 'MN-2024-001',
     (SELECT id FROM disciplines WHERE code = 'QA'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2024-04-01'),
    -- MAINT: 1件
    ('設備台帳整備', 'completed', '2024-07-01', '2025-01-31', 'MN-2024-002',
     (SELECT id FROM disciplines WHERE code = 'MAINT'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2024-07-01'),

    -- ── 2023年度 (2023-04〜2024-03) ──
    -- MECH: 2件
    ('配管耐震補強', 'completed', '2023-10-01', '2024-03-31', 'DV-2023-002',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2023-10-01'),
    ('回転機振動調査', 'completed', '2023-04-01', '2023-09-30', 'DV-2023-001',
     (SELECT id FROM disciplines WHERE code = 'MECH'),
     (SELECT id FROM employees WHERE employee_code = 'PM001'),
     '2023-04-01'),
    -- ELEC: 1件
    ('非常用発電機更新', 'completed', '2023-08-01', '2024-02-29', 'IN-2023-001',
     (SELECT id FROM disciplines WHERE code = 'ELEC'),
     (SELECT id FROM employees WHERE employee_code = 'PM003'),
     '2023-08-01'),
    -- MAINT: 1件
    ('予備品管理システム導入', 'completed', '2023-06-01', '2024-01-31', 'MN-2023-001',
     (SELECT id FROM disciplines WHERE code = 'MAINT'),
     (SELECT id FROM employees WHERE employee_code = 'PM002'),
     '2023-06-01');
