CREATE TABLE document_kinds (
    id         UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    code       VARCHAR(10)    NOT NULL,
    name       VARCHAR(100)   NOT NULL,
    seq_digits INTEGER        NOT NULL CHECK (seq_digits IN (2, 3)),
    created_at TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT document_kinds_code_unique UNIQUE (code)
);
