-- start_date を NOT NULL に変更
-- 既存の NULL 行は created_at の日付で埋める
UPDATE projects SET start_date = created_at::date WHERE start_date IS NULL;
ALTER TABLE projects ALTER COLUMN start_date SET NOT NULL;
