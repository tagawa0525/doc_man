CREATE TABLE distributions (
    id              UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id     UUID        NOT NULL REFERENCES documents(id),
    recipient_id    UUID        NOT NULL REFERENCES employees(id),
    distributed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    distributed_by  UUID        NOT NULL REFERENCES employees(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_distributions_document_id ON distributions(document_id);
CREATE INDEX idx_distributions_recipient_id ON distributions(recipient_id);
