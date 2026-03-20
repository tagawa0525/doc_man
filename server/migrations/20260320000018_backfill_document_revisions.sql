-- 既存文書を document_revisions に Rev.0 として記録
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from, effective_to)
SELECT id, 0, doc_number || '/0', NULL, author_id, created_at, NULL
FROM documents
WHERE NOT EXISTS (
    SELECT 1 FROM document_revisions dr WHERE dr.document_id = documents.id AND dr.revision = 0
);

-- 既存文書の revision を 0 にリセット
UPDATE documents SET revision = 0 WHERE revision >= 1;
