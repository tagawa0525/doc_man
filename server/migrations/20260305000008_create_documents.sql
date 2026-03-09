CREATE TABLE documents (
    id               UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    doc_number       VARCHAR(30)    NOT NULL,
    revision         INTEGER        NOT NULL DEFAULT 1 CHECK (revision >= 1),
    title            VARCHAR(300)   NOT NULL,
    file_path        VARCHAR(500)   NOT NULL,
    author_id        UUID           NOT NULL REFERENCES employees(id),
    doc_kind_id      UUID           NOT NULL REFERENCES document_kinds(id),
    frozen_dept_code VARCHAR(10)    NOT NULL,
    confidentiality  VARCHAR(20)    NOT NULL DEFAULT 'internal'
                     CHECK (confidentiality IN ('public', 'internal', 'restricted', 'confidential')),
    status           VARCHAR(20)    NOT NULL DEFAULT 'draft'
                     CHECK (status IN ('draft', 'under_review', 'approved', 'rejected', 'circulating', 'completed')),
    project_id       UUID           NOT NULL REFERENCES projects(id),
    created_at       TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT documents_doc_number_unique UNIQUE (doc_number)
);

CREATE INDEX idx_documents_doc_number       ON documents(doc_number);
CREATE INDEX idx_documents_project_id       ON documents(project_id);
CREATE INDEX idx_documents_author_id        ON documents(author_id);
CREATE INDEX idx_documents_doc_kind_id      ON documents(doc_kind_id);
CREATE INDEX idx_documents_confidentiality  ON documents(confidentiality);
CREATE INDEX idx_documents_status           ON documents(status);
