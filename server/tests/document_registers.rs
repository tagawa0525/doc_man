use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /document-registers ───────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_registers_returns_paginated_list(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-registers")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["register_code"], "内技術");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_registers_with_doc_kind_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind_a = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let kind_b = helpers::insert_document_kind(&pool, "議", "議事録", 2).await;
    helpers::insert_document_register(&pool, "内技術", kind_a, dept).await;
    helpers::insert_document_register(&pool, "議技術", kind_b, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/document-registers?doc_kind_id={kind_a}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["register_code"], "内技術");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_registers_with_department_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept_a = helpers::insert_department(&pool, "001", "技術部", None).await;
    let dept_b = helpers::insert_department(&pool, "002", "営業部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    helpers::insert_document_register(&pool, "内技術", kind, dept_a).await;

    // kind_b for dept_b requires a different kind or different (kind, dept) pair
    let kind_b = helpers::insert_document_kind(&pool, "議", "議事録", 2).await;
    helpers::insert_document_register(&pool, "議営業", kind_b, dept_b).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/document-registers?department_id={dept_a}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["register_code"], "内技術");
}

// ── POST /document-registers ──────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_register_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-registers")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "register_code": "内技術",
                        "doc_kind_id": kind,
                        "department_id": dept,
                        "file_server_root": "/nas/tech/design"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["register_code"], "内技術");
    assert_eq!(body["doc_kind"]["code"], "内");
    assert_eq!(body["department"]["code"], "001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_register_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-registers")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "register_code": "内技術",
                        "doc_kind_id": kind,
                        "department_id": dept,
                        "file_server_root": "/nas/tech/design"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_register_duplicate_code_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    // 同じdepartment_idだが別kindを使い register_code のみ重複させる
    let kind_b = helpers::insert_document_kind(&pool, "議", "議事録", 2).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-registers")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "register_code": "内技術",
                        "doc_kind_id": kind_b,
                        "department_id": dept,
                        "file_server_root": "/nas/tech/design"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── GET /document-registers/{id} ──────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_register_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let reg_id = helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/document-registers/{reg_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["register_code"], "内技術");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_register_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-registers/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /document-registers/{id} ──────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_register_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let reg_id = helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-registers/{reg_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "file_server_root": "/new/path" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["file_server_root"], "/new/path");
    assert_eq!(body["register_code"], "内技術"); // register_code変わらない
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_register_register_code_change_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let reg_id = helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-registers/{reg_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "register_code": "変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_register_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let reg_id = helpers::insert_document_register(&pool, "内技術", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-registers/{reg_id}"))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "file_server_root": "/new" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
