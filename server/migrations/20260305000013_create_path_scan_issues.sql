CREATE TABLE path_scan_issues (
    id          UUID           NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    document_id UUID           REFERENCES documents(id),
    found_path  VARCHAR(500)   NOT NULL,
    issue_kind  VARCHAR(20)    NOT NULL
                CHECK (issue_kind IN ('no_match', 'multiple_match')),
    resolved_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ    NOT NULL DEFAULT now()
);

CREATE INDEX idx_path_scan_issues_resolved_at ON path_scan_issues(resolved_at);
