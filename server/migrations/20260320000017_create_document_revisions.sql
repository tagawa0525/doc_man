CREATE TABLE document_revisions (
    id             UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id    UUID        NOT NULL REFERENCES documents(id),
    revision       INTEGER     NOT NULL CHECK (revision >= 0),
    file_path      VARCHAR(500) NOT NULL,
    reason         TEXT,
    created_by     UUID        NOT NULL REFERENCES employees(id),
    effective_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_to   TIMESTAMPTZ,
    UNIQUE (document_id, revision)
);

CREATE INDEX idx_document_revisions_document_id ON document_revisions(document_id);

ALTER TABLE documents DROP COLUMN file_path;
ALTER TABLE documents ALTER COLUMN revision SET DEFAULT 0;
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_revision_check;
ALTER TABLE documents ADD CONSTRAINT documents_revision_check CHECK (revision >= 0);
