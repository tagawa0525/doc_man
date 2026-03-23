-- 01_master.sql: 職位・部署・従業員・タグ・文書種別
-- 他テーブルに依存しない基盤データ

--------------------------------------------------------------------------------
-- positions（マイグレーションで初期データ投入済み、冪等に追加）
--------------------------------------------------------------------------------
\echo '  → positions: 職位マスタ (7件)'
INSERT INTO positions (name, default_role, sort_order) VALUES
    ('社長',   'admin',           10),
    ('部長',   'admin',           20),
    ('課長',   'admin',           30),
    ('総合職', 'project_manager', 40),
    ('一般職', 'general',         50),
    ('嘱託',   'viewer',          60),
    ('派遣',   'viewer',          70)
ON CONFLICT (name) DO NOTHING;

--------------------------------------------------------------------------------
-- departments (10件、3階層: 本社→部→課)
--------------------------------------------------------------------------------
\echo '  → departments: 部署 (10件, 3階層)'
INSERT INTO departments (code, name, effective_from) VALUES
    ('HQ', '本社', '2020-01-01');
INSERT INTO departments (code, name, parent_id, effective_from) VALUES
    ('設計', '設計部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('品管', '品質管理部', (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('保全', '保全部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('管理', '管理部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01'),
    ('製造', '製造部',     (SELECT id FROM departments WHERE code = 'HQ'), '2020-01-01');
INSERT INTO departments (code, name, parent_id, effective_from) VALUES
    ('機設', '機械設計課', (SELECT id FROM departments WHERE code = '設計'), '2020-01-01'),
    ('電設', '電気設計課', (SELECT id FROM departments WHERE code = '設計'), '2020-01-01'),
    ('計設', '計装設計課', (SELECT id FROM departments WHERE code = '設計'), '2020-01-01'),
    ('製1', '製造1課',    (SELECT id FROM departments WHERE code = '製造'), '2020-01-01');

--------------------------------------------------------------------------------
-- employees (16件、全ロール・部署を網羅)
--------------------------------------------------------------------------------
\echo '  → employees: 従業員 (16件)'
INSERT INTO employees (name, employee_code, email, role, position_id, is_active) VALUES
    -- admin
    ('管理太郎', 'ADM001', 'kanri@example.com',    'admin',
     (SELECT id FROM positions WHERE name = '社長'),   true),
    -- 部長クラス（admin、各部の長）
    ('設計部長', 'MGR001', 'sekkei-bucho@example.com', NULL,
     (SELECT id FROM positions WHERE name = '部長'),   true),
    ('品管部長', 'MGR002', 'hinkan-bucho@example.com', NULL,
     (SELECT id FROM positions WHERE name = '部長'),   true),
    -- 課長クラス（project_manager）
    ('山田花子', 'PM001',  'yamada@example.com',   'project_manager',
     (SELECT id FROM positions WHERE name = '課長'),   true),
    ('佐藤次郎', 'PM002',  'sato@example.com',    'project_manager',
     (SELECT id FROM positions WHERE name = '課長'),   true),
    ('渡辺直樹', 'PM003',  'watanabe@example.com', 'project_manager',
     (SELECT id FROM positions WHERE name = '課長'),   true),
    -- 総合職（general）
    ('鈴木一郎', 'GEN001', 'suzuki@example.com',  NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('田中美咲', 'GEN002', 'tanaka@example.com',  NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('高橋健太', 'GEN003', 'takahashi@example.com',NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('小林真理', 'GEN004', 'kobayashi@example.com', NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    ('加藤裕也', 'GEN005', 'kato@example.com',     NULL,
     (SELECT id FROM positions WHERE name = '総合職'), true),
    -- 一般職
    ('吉田恵子', 'STF001', 'yoshida@example.com',  NULL,
     (SELECT id FROM positions WHERE name = '一般職'), true),
    ('松本大輔', 'STF002', 'matsumoto@example.com', NULL,
     (SELECT id FROM positions WHERE name = '一般職'), true),
    -- 嘱託・派遣（viewer）
    ('中村由紀', 'VW001',  'nakamura@example.com', NULL,
     (SELECT id FROM positions WHERE name = '嘱託'),   true),
    ('木村太一', 'VW002',  'kimura@example.com',   NULL,
     (SELECT id FROM positions WHERE name = '派遣'),   true),
    -- 退職者
    ('伊藤誠',   'EX001',  'ito@example.com',     NULL,
     (SELECT id FROM positions WHERE name = '一般職'), false);

--------------------------------------------------------------------------------
-- department_role_grants
--------------------------------------------------------------------------------
\echo '  → department_role_grants: 部署ロール付与 (1件)'
INSERT INTO department_role_grants (department_id, role) VALUES
    ((SELECT id FROM departments WHERE code = '管理'), 'admin');

--------------------------------------------------------------------------------
-- employee_departments (18件: 兼務含む)
--------------------------------------------------------------------------------
\echo '  → employee_departments: 所属 (18件)'
INSERT INTO employee_departments (employee_id, department_id, is_primary, effective_from) VALUES
    -- 管理部
    ((SELECT id FROM employees WHERE employee_code = 'ADM001'),
     (SELECT id FROM departments WHERE code = '管理'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'VW001'),
     (SELECT id FROM departments WHERE code = '管理'), true, '2023-04-01'),
    -- 設計部
    ((SELECT id FROM employees WHERE employee_code = 'MGR001'),
     (SELECT id FROM departments WHERE code = '設計'), true, '2020-04-01'),
    -- 機械設計課
    ((SELECT id FROM employees WHERE employee_code = 'PM001'),
     (SELECT id FROM departments WHERE code = '機設'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM departments WHERE code = '機設'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN004'),
     (SELECT id FROM departments WHERE code = '機設'), true, '2022-04-01'),
    -- 電気設計課
    ((SELECT id FROM employees WHERE employee_code = 'PM003'),
     (SELECT id FROM departments WHERE code = '電設'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN002'),
     (SELECT id FROM departments WHERE code = '電設'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'VW002'),
     (SELECT id FROM departments WHERE code = '電設'), true, '2024-04-01'),
    -- 計装設計課
    ((SELECT id FROM employees WHERE employee_code = 'GEN005'),
     (SELECT id FROM departments WHERE code = '計設'), true, '2022-04-01'),
    -- 品質管理部
    ((SELECT id FROM employees WHERE employee_code = 'MGR002'),
     (SELECT id FROM departments WHERE code = '品管'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'PM002'),
     (SELECT id FROM departments WHERE code = '品管'), true, '2020-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'GEN003'),
     (SELECT id FROM departments WHERE code = '品管'), true, '2021-04-01'),
    -- 保全部
    ((SELECT id FROM employees WHERE employee_code = 'STF001'),
     (SELECT id FROM departments WHERE code = '保全'), true, '2021-04-01'),
    ((SELECT id FROM employees WHERE employee_code = 'STF002'),
     (SELECT id FROM departments WHERE code = '保全'), true, '2022-04-01'),
    -- 製造1課
    ((SELECT id FROM employees WHERE employee_code = 'EX001'),
     (SELECT id FROM departments WHERE code = '製1'), true, '2021-04-01'),
    -- 兼務: PM002 は保全部も兼務
    ((SELECT id FROM employees WHERE employee_code = 'PM002'),
     (SELECT id FROM departments WHERE code = '保全'), false, '2022-04-01'),
    -- 兼務: GEN001 は品管も兼務
    ((SELECT id FROM employees WHERE employee_code = 'GEN001'),
     (SELECT id FROM departments WHERE code = '品管'), false, '2024-04-01');

--------------------------------------------------------------------------------
-- tags (8件)
--------------------------------------------------------------------------------
\echo '  → tags: タグ (8件)'
INSERT INTO tags (name) VALUES
    ('安全'), ('環境'), ('品質'), ('設計変更'), ('緊急'), ('機密'), ('コスト削減'), ('法規制');

--------------------------------------------------------------------------------
-- document_kinds (5件)
--------------------------------------------------------------------------------
\echo '  → document_kinds: 文書種別 (5件)'
INSERT INTO document_kinds (code, name, seq_digits) VALUES
    ('内', '社内文書', 3),
    ('外', '外部文書', 3),
    ('議', '議事録',   2),
    ('仕', '仕様書',   3),
    ('手', '手順書',   3);
