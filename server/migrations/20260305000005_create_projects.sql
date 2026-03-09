CREATE TABLE projects (
    id            UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name          VARCHAR(200)   NOT NULL,
    status        VARCHAR(20)    NOT NULL DEFAULT 'planning'
                  CHECK (status IN ('planning', 'active', 'completed', 'cancelled')),
    start_date    DATE,
    end_date      DATE,
    wbs_code      VARCHAR(50)    UNIQUE,
    discipline_id UUID           NOT NULL REFERENCES disciplines(id),
    manager_id    UUID           REFERENCES employees(id),
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_projects_discipline_id ON projects(discipline_id);
CREATE INDEX idx_projects_status        ON projects(status);
CREATE INDEX idx_projects_wbs_code      ON projects(wbs_code);
