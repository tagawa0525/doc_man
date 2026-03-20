use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /documents filters ───────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_dept_codes(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    helpers::insert_document_with_dept(&pool, "DOC-001", "設計文書", admin.id, kind, proj, "設計")
        .await;
    helpers::insert_document_with_dept(&pool, "DOC-002", "製造文書", admin.id, kind, proj, "製造")
        .await;
    helpers::insert_document_with_dept(&pool, "DOC-003", "品質文書", admin.id, kind, proj, "品質")
        .await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?dept_codes=%E8%A8%AD%E8%A8%88,%E5%93%81%E8%B3%AA")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 2);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_doc_kind_id(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind_a = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let kind_b = helpers::insert_document_kind(&pool, "外", "社外", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    helpers::insert_document(&pool, "DOC-001", "社内文書", admin.id, kind_a, proj).await;
    helpers::insert_document(&pool, "DOC-002", "社外文書", admin.id, kind_b, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents?doc_kind_id={kind_a}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["doc_kind"]["id"], kind_a.to_string());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_fiscal_year(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    // 2025年度 (2025-04-01 〜 2026-03-31)
    helpers::insert_document_with_created_at(
        &pool, "DOC-001", "2025年度文書", admin.id, kind, proj, "設計", "2025-06-15T00:00:00Z",
    )
    .await;
    // 2024年度
    helpers::insert_document_with_created_at(
        &pool, "DOC-002", "2024年度文書", admin.id, kind, proj, "設計", "2024-06-15T00:00:00Z",
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?fiscal_year=2025")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "2025年度文書");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_multiple_fiscal_years(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    helpers::insert_document_with_created_at(
        &pool, "DOC-001", "2025年度文書", admin.id, kind, proj, "設計", "2025-06-15T00:00:00Z",
    )
    .await;
    helpers::insert_document_with_created_at(
        &pool, "DOC-002", "2024年度文書", admin.id, kind, proj, "設計", "2024-06-15T00:00:00Z",
    )
    .await;
    helpers::insert_document_with_created_at(
        &pool, "DOC-003", "2023年度文書", admin.id, kind, proj, "設計", "2023-06-15T00:00:00Z",
    )
    .await;

    // fiscal_years=2024,2025 → 2件
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?fiscal_years=2024,2025")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 2);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_project_name(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj_a = helpers::insert_project(&pool, "火力発電プロジェクト", disc, None).await;
    let proj_b = helpers::insert_project(&pool, "水力発電プロジェクト", disc, None).await;

    helpers::insert_document(&pool, "DOC-001", "文書A", admin.id, kind, proj_a).await;
    helpers::insert_document(&pool, "DOC-002", "文書B", admin.id, kind, proj_b).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?project_name=%E7%81%AB%E5%8A%9B")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "文書A");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_author_name(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let other = helpers::insert_employee(&pool, "OTHER001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    helpers::insert_document(&pool, "DOC-001", "文書A", admin.id, kind, proj).await;
    helpers::insert_document(&pool, "DOC-002", "文書B", other.id, kind, proj).await;

    // "OTHER001" の名前は "Test OTHER001"
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?author_name=OTHER001")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "文書B");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_filters_by_wbs_code(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj_a =
        helpers::insert_project_with_wbs(&pool, "PJ-A", disc, None, "WBS-001-A").await;
    let proj_b =
        helpers::insert_project_with_wbs(&pool, "PJ-B", disc, None, "WBS-002-B").await;

    helpers::insert_document(&pool, "DOC-001", "文書A", admin.id, kind, proj_a).await;
    helpers::insert_document(&pool, "DOC-002", "文書B", admin.id, kind, proj_b).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?wbs_code=001-A")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "文書A");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_combines_multiple_filters(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind_a = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let kind_b = helpers::insert_document_kind(&pool, "外", "社外", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    helpers::insert_document_with_dept(&pool, "DOC-001", "設計社内", admin.id, kind_a, proj, "設計")
        .await;
    helpers::insert_document_with_dept(&pool, "DOC-002", "設計社外", admin.id, kind_b, proj, "設計")
        .await;
    helpers::insert_document_with_dept(&pool, "DOC-003", "製造社内", admin.id, kind_a, proj, "製造")
        .await;

    // dept_codes=設計 AND doc_kind_id=kind_a → DOC-001のみ
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/documents?dept_codes=%E8%A8%AD%E8%A8%88&doc_kind_id={kind_a}"
                ))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "設計社内");
}

// ── POST /documents ─────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_assigns_doc_number_and_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テストプロジェクト", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "title": "テスト文書",
                        "doc_kind_id": kind,
                        "project_id": proj
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    let doc_number = body["doc_number"].as_str().unwrap();
    assert!(doc_number.starts_with("内設計-"));
    assert_eq!(body["status"], "draft");
    assert_eq!(body["revision"], 0);
    let file_path = body["file_path"].as_str().unwrap();
    assert!(file_path.ends_with("/0"));
    assert_eq!(body["title"], "テスト文書");
    assert_eq!(body["author"]["id"], admin.id.to_string());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_with_tags(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_tag(&pool, "外形図").await;
    helpers::insert_tag(&pool, "設備").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "title": "テスト文書",
                        "doc_kind_id": kind,
                        "project_id": proj,
                        "tags": ["外形図", "設備"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    let tags = body["tags"].as_array().unwrap();
    assert_eq!(tags.len(), 2);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_viewer_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "title": "テスト",
                        "doc_kind_id": kind,
                        "project_id": proj
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── GET /documents ──────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_returns_paginated_list(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_document(&pool, "内設計-2603001", "テスト1", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["doc_number"], "内設計-2603001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_with_project_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj_a = helpers::insert_project(&pool, "プロジェクトA", disc, None).await;
    let proj_b = helpers::insert_project(&pool, "プロジェクトB", disc, None).await;
    helpers::insert_document(&pool, "内設計-2603001", "文書A", admin.id, kind, proj_a).await;
    helpers::insert_document(&pool, "内設計-2603002", "文書B", admin.id, kind, proj_b).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents?project_id={proj_a}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "文書A");
}

// ── GET /documents?q= ───────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_with_q_filters_by_title(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_document(&pool, "内設計-2603001", "配管設計書", admin.id, kind, proj).await;
    helpers::insert_document(&pool, "内設計-2603002", "電気回路図", admin.id, kind, proj).await;
    helpers::insert_document(
        &pool,
        "内設計-2603003",
        "配管施工要領",
        admin.id,
        kind,
        proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?q=%E9%85%8D%E7%AE%A1") // q=配管
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 2);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_with_q_filters_by_doc_number(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_document(&pool, "内設計-2603001", "文書A", admin.id, kind, proj).await;
    helpers::insert_document(&pool, "内設計-2603002", "文書B", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?q=2603001")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["doc_number"], "内設計-2603001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_with_q_is_case_insensitive(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_document(&pool, "ABC-001", "Design Report", admin.id, kind, proj).await;
    helpers::insert_document(&pool, "DEF-002", "Test Plan", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?q=design")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["doc_number"], "ABC-001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_with_q_escapes_like_wildcards(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::insert_document(
        &pool,
        "内設計-2603001",
        "100%完了報告",
        admin.id,
        kind,
        proj,
    )
    .await;
    helpers::insert_document(&pool, "内設計-2603002", "通常文書", admin.id, kind, proj).await;

    // q=100% — % はリテラルとして扱われるべき
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents?q=100%25") // %25 = URL-encoded %
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["title"], "100%完了報告");
}

// ── GET /documents/{id} ─────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト文書", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["doc_number"], "内設計-2603001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /documents/{id} ─────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_updates_title_without_incrementing_revision(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "旧タイトル", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "title": "新タイトル" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["title"], "新タイトル");
    assert_eq!(body["revision"], 0); // revision は PUT で変わらない
    assert_eq!(body["doc_number"], "内設計-2603001"); // doc_number不変
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_doc_number_change_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "doc_number": "変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_status_change_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "status": "approved" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── DELETE /documents/{id} ──────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_document_admin_returns_204(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "削除対象", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_document_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_document_with_distributions_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient = helpers::insert_employee(&pool, "GEN001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    // 配布レコードを直接挿入
    sqlx::query(
        "INSERT INTO distributions (document_id, recipient_id, distributed_by)
         VALUES ($1, $2, $3)",
    )
    .bind(doc_id)
    .bind(recipient.id)
    .bind(admin.id)
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── POST /documents/{id}/revise ─────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn revise_approved_document_creates_new_revision(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    // approved 状態にする
    sqlx::query("UPDATE documents SET status = 'approved' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/revise"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "reason": "設計変更のため" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["revision"], 1);
    assert_eq!(body["status"], "draft");
    let file_path = body["file_path"].as_str().unwrap();
    assert!(file_path.ends_with("/1"));
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn revise_draft_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/revise"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "reason": "テスト" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn revise_requires_reason(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    sqlx::query("UPDATE documents SET status = 'approved' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/revise"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(json!({ "reason": "" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn revise_viewer_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    sqlx::query("UPDATE documents SET status = 'approved' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/revise"))
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "reason": "テスト" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── GET /documents/{id}/revisions ───────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_revisions_for_new_document_returns_one(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{doc_id}/revisions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let revisions = body.as_array().unwrap();
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0]["revision"], 0);
    assert!(revisions[0]["reason"].is_null());
}
