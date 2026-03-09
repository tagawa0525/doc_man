CREATE TABLE employee_departments (
    id              UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    employee_id     UUID        NOT NULL REFERENCES employees(id),
    department_id   UUID        NOT NULL REFERENCES departments(id),
    is_primary      BOOLEAN     NOT NULL DEFAULT false,
    effective_from  DATE        NOT NULL,
    effective_to    DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_employee_departments_employee_id
    ON employee_departments(employee_id, effective_to);
CREATE INDEX idx_employee_departments_department_id
    ON employee_departments(department_id, effective_to);
CREATE INDEX idx_employee_departments_primary
    ON employee_departments(employee_id, is_primary, effective_to);
