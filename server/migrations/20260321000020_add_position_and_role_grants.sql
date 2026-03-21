-- 1. department_role_grants テーブル
CREATE TABLE department_role_grants (
    id            UUID         NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    department_id UUID         NOT NULL UNIQUE REFERENCES departments(id),
    role          VARCHAR(20)  NOT NULL
                  CHECK (role IN ('admin', 'project_manager', 'general', 'viewer')),
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- 2. employees に position_id 追加
ALTER TABLE employees ADD COLUMN position_id UUID REFERENCES positions(id);

-- 3. 既存データのバックフィル（現在の role から職位を推定）
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '課長')   WHERE role = 'admin';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '総合職') WHERE role = 'project_manager';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '一般職') WHERE role = 'general';
UPDATE employees SET position_id = (SELECT id FROM positions WHERE name = '嘱託')   WHERE role = 'viewer';

-- 4. NOT NULL 制約追加
ALTER TABLE employees ALTER COLUMN position_id SET NOT NULL;

-- 5. role を nullable に変更（NULL = 上書きなし、職位/部署のデフォルトを使用）
ALTER TABLE employees ALTER COLUMN role DROP NOT NULL;
ALTER TABLE employees ALTER COLUMN role DROP DEFAULT;
