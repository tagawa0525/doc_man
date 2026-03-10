DELETE FROM circulations;
DROP TABLE IF EXISTS circulations;

UPDATE documents SET status = 'approved', updated_at = now()
WHERE status IN ('circulating', 'completed');

ALTER TABLE documents
    DROP CONSTRAINT documents_status_check,
    ADD CONSTRAINT documents_status_check
        CHECK (status IN ('draft', 'under_review', 'approved', 'rejected'));
