CREATE TABLE employees (
    id            UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name          VARCHAR(100)   NOT NULL,
    employee_code VARCHAR(20)    UNIQUE,
    ad_account    VARCHAR(100)   UNIQUE,
    role          VARCHAR(20)    NOT NULL DEFAULT 'viewer'
                  CHECK (role IN ('admin', 'project_manager', 'general', 'viewer')),
    is_active     BOOLEAN        NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_employees_employee_code ON employees(employee_code);
CREATE INDEX idx_employees_ad_account    ON employees(ad_account);
CREATE INDEX idx_employees_is_active     ON employees(is_active);
