use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

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
async fn put_document_updates_title_and_increments_revision(pool: PgPool) {
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
