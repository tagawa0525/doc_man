-- 05_historical.sql: 過去15年分の履歴データ（2011〜2024）
-- 各分野・各年度にプロジェクト、各文書種別に2文書ずつ生成

\echo '  → 過去プロジェクト・文書を生成 (2011〜2024)'

DO $$
DECLARE
    v_year         INT;
    v_fy_start     DATE;
    v_proj_id      UUID;
    v_doc_id       UUID;
    v_doc_num      TEXT;
    -- 専門分野（部署コード付き）
    v_disc_ids     UUID[];
    v_disc_names   TEXT[];
    v_disc_depts   TEXT[];   -- 分野が属する部署コード
    -- マネージャー
    v_mgr_ids      UUID[];
    -- 作成者
    v_author_ids   UUID[];
    -- 文書種別
    v_dk_ids       UUID[];
    v_dk_codes     TEXT[];
    -- ループ変数
    v_di           INT;  -- discipline index
    v_ki           INT;  -- kind index
    v_ni           INT;  -- doc number within kind (1..2)
    v_seq          INT;
    v_doc_created  TIMESTAMPTZ;
    v_yymm         TEXT;
BEGIN
    -- 分野を部署コード順・分野コード順で取得
    v_disc_ids   := ARRAY(
        SELECT di.id FROM disciplines di
        JOIN departments d ON d.id = di.department_id
        ORDER BY d.code, di.code
    );
    v_disc_names := ARRAY(
        SELECT di.name FROM disciplines di
        JOIN departments d ON d.id = di.department_id
        ORDER BY d.code, di.code
    );
    v_disc_depts := ARRAY(
        SELECT d.code FROM disciplines di
        JOIN departments d ON d.id = di.department_id
        ORDER BY d.code, di.code
    );

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

        -- 各分野に1プロジェクト
        FOR v_di IN 1..array_length(v_disc_ids, 1) LOOP
            INSERT INTO projects (
                name, status, start_date, end_date, wbs_code,
                discipline_id, manager_id, created_at
            ) VALUES (
                format('%s年度 %s案件', v_year, v_disc_names[v_di]),
                'completed',
                v_fy_start + ((v_di - 1) * 15 || ' days')::INTERVAL,
                v_fy_start + ((v_di * 30 + 90) || ' days')::INTERVAL,
                format('HI-%s-%s', v_year, v_di),
                v_disc_ids[v_di],
                v_mgr_ids[((v_year + v_di) % 3) + 1],
                v_fy_start + ((v_di - 1) * 15 || ' days')::INTERVAL + '09:00'::INTERVAL
            )
            RETURNING id INTO v_proj_id;

            -- 各文書種別に2文書ずつ
            FOR v_ki IN 1..array_length(v_dk_ids, 1) LOOP
                FOR v_ni IN 1..2 LOOP
                    v_seq := (v_di - 1) * 100 + (v_ki - 1) * 10 + v_ni;

                    v_doc_created := v_fy_start
                        + (((v_di - 1) * 20 + v_ki * 5 + v_ni * 2) || ' days')::INTERVAL
                        + '10:00'::INTERVAL;
                    v_yymm := to_char(v_doc_created, 'YYMM');

                    v_doc_num := format('%s%s-%s%03s',
                        v_dk_codes[v_ki], v_disc_depts[v_di],
                        v_yymm, v_seq);

                    INSERT INTO documents (
                        doc_number, title, author_id, doc_kind_id,
                        frozen_dept_code, status, confidentiality,
                        project_id, created_at
                    ) VALUES (
                        v_doc_num,
                        format('%s年度 %s %s #%s',
                            v_year, v_disc_names[v_di],
                            CASE v_dk_codes[v_ki]
                                WHEN '内' THEN '報告書'
                                WHEN '外' THEN '外部文書'
                                WHEN '議' THEN '議事録'
                                WHEN '仕' THEN '仕様書'
                                WHEN '手' THEN '手順書'
                            END,
                            v_ni),
                        v_author_ids[((v_seq + v_year) % array_length(v_author_ids, 1)) + 1],
                        v_dk_ids[v_ki],
                        v_disc_depts[v_di],
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
                        v_author_ids[((v_seq + v_year) % array_length(v_author_ids, 1)) + 1],
                        v_doc_created
                    );
                END LOOP;
            END LOOP;
        END LOOP;
    END LOOP;
END $$;
