CREATE TABLE departments (
    id               UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    code             VARCHAR(10)    NOT NULL,
    name             VARCHAR(100)   NOT NULL,
    parent_id        UUID           REFERENCES departments(id),
    effective_from   DATE           NOT NULL,
    effective_to     DATE,
    merged_into_id   UUID           REFERENCES departments(id),
    created_at       TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT departments_code_unique UNIQUE (code)
);

CREATE INDEX idx_departments_code        ON departments(code);
CREATE INDEX idx_departments_parent_id   ON departments(parent_id);
