CREATE TABLE disciplines (
    id            UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    code          VARCHAR(10)    NOT NULL,
    name          VARCHAR(100)   NOT NULL,
    department_id UUID           NOT NULL REFERENCES departments(id),
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT disciplines_code_unique UNIQUE (code)
);

CREATE INDEX idx_disciplines_department_id ON disciplines(department_id);
