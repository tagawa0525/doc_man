use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /document-kinds ───────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_kinds_returns_paginated_list(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    helpers::insert_document_kind(&pool, "議", "議事録", 2).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-kinds")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 2);
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
}

// ── POST /document-kinds ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_kind_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-kinds")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "code": "内", "name": "社内", "seq_digits": 3 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "内");
    assert_eq!(body["seq_digits"], 3);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_kind_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-kinds")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "code": "内", "name": "社内", "seq_digits": 3 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_kind_duplicate_code_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-kinds")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "code": "内", "name": "別名", "seq_digits": 3 }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── GET /document-kinds/{id} ──────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_kind_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let id = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/document-kinds/{id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "内");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_document_kind_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/document-kinds/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /document-kinds/{id} ──────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_kind_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let id = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-kinds/{id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "社内文書" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "社内文書");
    assert_eq!(body["code"], "内"); // code変わらない
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_kind_code_change_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let id = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-kinds/{id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(json!({ "code": "外" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_kind_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let id = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-kinds/{id}"))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
