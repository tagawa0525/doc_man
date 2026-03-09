CREATE TABLE circulations (
    id           UUID        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id  UUID        NOT NULL REFERENCES documents(id),
    recipient_id UUID        NOT NULL REFERENCES employees(id),
    confirmed_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT circulations_unique UNIQUE (document_id, recipient_id)
);

CREATE INDEX idx_circulations_doc_confirmed
    ON circulations(document_id, confirmed_at);
CREATE INDEX idx_circulations_recipient_confirmed
    ON circulations(recipient_id, confirmed_at);
