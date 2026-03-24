-- 文書番号を構成要素に分解し、doc_number を生成列にする
--
-- Before: doc_number VARCHAR(30) — 手動で構築した文字列
-- After:  frozen_kind_code, frozen_dept_code, doc_period, doc_seq, frozen_seq_digits
--         → doc_number GENERATED ALWAYS AS (...) STORED

-- 1. 構成要素カラムを追加（nullable で追加し、バックフィル後に NOT NULL にする）
ALTER TABLE documents
    ADD COLUMN frozen_kind_code  VARCHAR(5),
    ADD COLUMN doc_period        TEXT,
    ADD COLUMN doc_seq           INT,
    ADD COLUMN frozen_seq_digits SMALLINT;

-- 2. 既存データのバックフィル
--    doc_number = '{kind_code}{dept_code}-{period}{seq_padded}'
--    split_part(doc_number, '-', 2) = '{period}{seq_padded}'
--    right(..., seq_digits) = seq_padded → ::INT = seq
--    left(..., len - seq_digits) = period
UPDATE documents d SET
    frozen_kind_code  = dk.code,
    frozen_seq_digits = dk.seq_digits::SMALLINT,
    doc_period = left(
        split_part(d.doc_number, '-', 2),
        length(split_part(d.doc_number, '-', 2)) - dk.seq_digits
    ),
    doc_seq = right(
        split_part(d.doc_number, '-', 2),
        dk.seq_digits
    )::INT
FROM document_kinds dk
WHERE dk.id = d.doc_kind_id;

-- 3. NOT NULL 制約を設定
ALTER TABLE documents
    ALTER COLUMN frozen_kind_code  SET NOT NULL,
    ALTER COLUMN doc_period        SET NOT NULL,
    ALTER COLUMN doc_seq           SET NOT NULL,
    ALTER COLUMN frozen_seq_digits SET NOT NULL;

-- 4. 旧 doc_number カラムとその制約・インデックスを削除
DROP INDEX IF EXISTS idx_documents_doc_number;
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_doc_number_unique;
ALTER TABLE documents DROP COLUMN doc_number;

-- 5. doc_number を生成列として再作成
ALTER TABLE documents ADD COLUMN doc_number VARCHAR(30) GENERATED ALWAYS AS (
    frozen_kind_code || frozen_dept_code || '-'
    || doc_period
    || lpad(doc_seq::text, frozen_seq_digits, '0')
) STORED;

-- 6. 制約・インデックスを再作成
ALTER TABLE documents ADD CONSTRAINT documents_doc_number_unique UNIQUE (doc_number);
CREATE INDEX idx_documents_doc_number ON documents(doc_number);

-- 7. 複合ユニーク制約（採番の一意性保証）
ALTER TABLE documents ADD CONSTRAINT documents_composite_unique
    UNIQUE (frozen_kind_code, frozen_dept_code, doc_period, doc_seq);

-- 8. document_revisions.file_path を再計算
--    旧 doc_number と生成列の doc_number が不一致の行（frozen_dept_code の不整合修正）で
--    file_path が古い doc_number を含んでいるため、生成列の値で再構築する
UPDATE document_revisions dr SET
    file_path = d.doc_number || '/' || dr.revision
FROM documents d
WHERE d.id = dr.document_id
  AND dr.file_path <> d.doc_number || '/' || dr.revision;
