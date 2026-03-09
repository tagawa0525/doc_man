CREATE TABLE document_tags (
    document_id UUID NOT NULL REFERENCES documents(id),
    tag_id      UUID NOT NULL REFERENCES tags(id),
    PRIMARY KEY (document_id, tag_id)
);
