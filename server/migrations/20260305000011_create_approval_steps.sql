CREATE TABLE approval_steps (
    id                UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id       UUID           NOT NULL REFERENCES documents(id),
    route_revision    INTEGER        NOT NULL,
    document_revision INTEGER        NOT NULL,
    step_order        INTEGER        NOT NULL,
    approver_id       UUID           NOT NULL REFERENCES employees(id),
    status            VARCHAR(20)    NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending', 'approved', 'rejected')),
    approved_at       TIMESTAMPTZ,
    comment           TEXT,
    created_at        TIMESTAMPTZ    NOT NULL DEFAULT now(),
    CONSTRAINT approval_steps_unique UNIQUE (document_id, route_revision, step_order)
);

CREATE INDEX idx_approval_steps_doc_route_order
    ON approval_steps(document_id, route_revision, step_order);
CREATE INDEX idx_approval_steps_approver_status
    ON approval_steps(approver_id, status);
