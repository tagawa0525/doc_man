CREATE TABLE positions (
    id           UUID         NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name         VARCHAR(100) NOT NULL UNIQUE,
    default_role VARCHAR(20)  NOT NULL DEFAULT 'viewer'
                 CHECK (default_role IN ('admin', 'project_manager', 'general', 'viewer')),
    sort_order   INT          NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_positions_sort_order ON positions(sort_order);

-- 初期データ（PR 2 のバックフィルに必要なためマイグレーションに含める）
INSERT INTO positions (name, default_role, sort_order) VALUES
    ('社長',   'admin',           1),
    ('部長',   'admin',           2),
    ('課長',   'admin',           3),
    ('総合職', 'project_manager', 4),
    ('一般職', 'general',         5),
    ('嘱託',   'viewer',          6),
    ('派遣',   'viewer',          7);
