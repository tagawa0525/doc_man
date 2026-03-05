-- gen_random_uuid() は PostgreSQL 13+ では組み込みだが、
-- 古い環境との互換性のため pgcrypto を明示的に有効化する
CREATE EXTENSION IF NOT EXISTS pgcrypto;
