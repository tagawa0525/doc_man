CREATE TABLE document_registers (
    id                  UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    register_code       VARCHAR(15)    NOT NULL,
    doc_kind_id         UUID           NOT NULL REFERENCES document_kinds(id),
    department_id       UUID           NOT NULL REFERENCES departments(id),
    file_server_root    VARCHAR(300)   NOT NULL,
    new_doc_sub_path    VARCHAR(300),
    doc_number_pattern  VARCHAR(200),
    created_at          TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT document_registers_register_code_unique UNIQUE (register_code),
    CONSTRAINT document_registers_kind_dept_unique UNIQUE (doc_kind_id, department_id)
);

CREATE INDEX idx_document_registers_kind_dept
    ON document_registers(doc_kind_id, department_id);
CREATE INDEX idx_document_registers_department_id
    ON document_registers(department_id);
