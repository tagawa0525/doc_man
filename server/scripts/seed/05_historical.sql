-- 05_historical.sql: 過去15年分の履歴データ（2011〜2024）
-- 全部署・全文書種別・全年度に文書を生成

\echo '  → 過去プロジェクト・文書を生成 (2011〜2024)'

DO $$
DECLARE
    v_year         INT;
    v_fy_start     DATE;
    v_proj_id      UUID;
    v_doc_id       UUID;
    v_doc_num      TEXT;
    -- 専門分野
    v_disc_ids     UUID[];
    v_disc_names   TEXT[];
    -- マネージャー
    v_mgr_ids      UUID[];
    -- 作成者
    v_author_ids   UUID[];
    -- 文書種別
    v_dk_ids       UUID[];
    v_dk_codes     TEXT[];
    -- 部署コード（frozen_dept_code として使用）
    v_dept_codes   TEXT[] := ARRAY['設計', '機設', '電設', '計設', '品管', '保全'];
    -- ループ変数
    v_di           INT;  -- discipline index
    v_ki           INT;  -- kind index
    v_dci          INT;  -- dept_code index
    v_seq          INT;
    v_doc_created  TIMESTAMPTZ;
    v_yymm         TEXT;
BEGIN
    -- ID を配列に取得
    v_disc_ids   := ARRAY(SELECT id FROM disciplines ORDER BY code);
    v_disc_names := ARRAY(SELECT name FROM disciplines ORDER BY code);
    v_mgr_ids    := ARRAY(
        SELECT id FROM employees
        WHERE employee_code IN ('PM001','PM002','PM003')
        ORDER BY employee_code
    );
    v_author_ids := ARRAY(
        SELECT id FROM employees
        WHERE employee_code IN ('GEN001','GEN002','GEN003','GEN004','GEN005','STF001','STF002')
        ORDER BY employee_code
    );
    v_dk_ids   := ARRAY(SELECT id FROM document_kinds ORDER BY code);
    v_dk_codes := ARRAY(SELECT code FROM document_kinds ORDER BY code);
    -- dk ORDER BY code: 外, 仕, 手, 内, 議

    FOR v_year IN 2011..2024 LOOP
        v_fy_start := make_date(v_year, 4, 1);

        -- 各年度に5プロジェクト（専門分野ごとに1つ）
        FOR v_di IN 1..array_length(v_disc_ids, 1) LOOP
            INSERT INTO projects (
                name, status, start_date, end_date, wbs_code,
                discipline_id, manager_id, created_at
            ) VALUES (
                format('%s年度 %s案件', v_year, v_disc_names[v_di]),
                'completed',
                v_fy_start + ((v_di - 1) * 30 || ' days')::INTERVAL,
                v_fy_start + ((v_di * 60 + 90) || ' days')::INTERVAL,
                format('HI-%s-%s-%s', v_year, v_di,
                    substr(v_disc_names[v_di], 1, 2)),
                v_disc_ids[v_di],
                v_mgr_ids[((v_year + v_di) % 3) + 1],
                v_fy_start + ((v_di - 1) * 30 || ' days')::INTERVAL + '09:00'::INTERVAL
            )
            RETURNING id INTO v_proj_id;

            -- 各プロジェクトに文書を生成:
            -- 全部署(6) × 部署ごとに種別を変えて、各部署に最低1文書
            -- → 各プロジェクト6文書、年間30文書
            FOR v_dci IN 1..array_length(v_dept_codes, 1) LOOP
                -- 文書種別: 部署+分野+年のローテーション
                v_ki := ((v_dci + v_di + v_year) % array_length(v_dk_ids, 1)) + 1;
                v_seq := (v_di - 1) * 10 + v_dci;

                v_doc_created := v_fy_start
                    + (((v_di - 1) * 40 + v_dci * 7) || ' days')::INTERVAL
                    + '10:00'::INTERVAL;
                v_yymm := to_char(v_doc_created, 'YYMM');

                v_doc_num := format('%s%s-%sH%03s',
                    v_dk_codes[v_ki], v_dept_codes[v_dci],
                    v_yymm, v_seq);

                INSERT INTO documents (
                    doc_number, title, author_id, doc_kind_id,
                    frozen_dept_code, status, confidentiality,
                    project_id, created_at
                ) VALUES (
                    v_doc_num,
                    format('%s年度 %s %s',
                        v_year, v_dept_codes[v_dci],
                        CASE v_dk_codes[v_ki]
                            WHEN '内' THEN '報告書'
                            WHEN '外' THEN '外部文書'
                            WHEN '議' THEN '議事録'
                            WHEN '仕' THEN '仕様書'
                            WHEN '手' THEN '手順書'
                        END),
                    v_author_ids[((v_dci + v_di + v_year) % array_length(v_author_ids, 1)) + 1],
                    v_dk_ids[v_ki],
                    v_dept_codes[v_dci],
                    'approved',
                    'internal',
                    v_proj_id,
                    v_doc_created
                )
                RETURNING id INTO v_doc_id;

                INSERT INTO document_revisions (
                    document_id, revision, file_path, created_by,
                    effective_from
                ) VALUES (
                    v_doc_id, 0, v_doc_num || '/0',
                    v_author_ids[((v_dci + v_di + v_year) % array_length(v_author_ids, 1)) + 1],
                    v_doc_created
                );
            END LOOP;
        END LOOP;
    END LOOP;
END $$;
