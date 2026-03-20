-- 既存文書の revision (1始まり) を document_revisions に展開
-- revision=N の文書に対し Rev.0〜Rev.N-1 を作成
-- Rev.N-1 が現行改訂 (effective_to = NULL)、それ以前は閉じる
INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by, effective_from, effective_to)
SELECT
    d.id,
    gs.rev,
    d.doc_number || '/' || gs.rev,
    NULL,
    d.author_id,
    d.created_at,
    CASE WHEN gs.rev < d.revision - 1 THEN d.created_at ELSE NULL END
FROM documents d
CROSS JOIN LATERAL generate_series(0, d.revision - 1) AS gs(rev)
WHERE NOT EXISTS (
    SELECT 1 FROM document_revisions dr WHERE dr.document_id = d.id AND dr.revision = gs.rev
);

-- 1始まり → 0始まりに変換 (revision N → N-1)
UPDATE documents SET revision = revision - 1 WHERE revision >= 1;
