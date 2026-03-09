use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /documents/{id}/circulations ───────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_circulations_returns_empty_array(pool: PgPool) {
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
                .uri(format!("/api/v1/documents/{doc_id}/circulations"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert!(body.as_array().unwrap().is_empty());
}

// ── POST /documents/{id}/circulations ──────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_circulations_creates_and_changes_status(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient1 = helpers::insert_employee(&pool, "RCP001", "general").await;
    let recipient2 = helpers::insert_employee(&pool, "RCP002", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト文書", admin.id, kind, proj).await;

    // 文書を approved に変更
    sqlx::query("UPDATE documents SET status = 'approved' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/circulations"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "recipient_ids": [recipient1.id, recipient2.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    let circs = body.as_array().unwrap();
    assert_eq!(circs.len(), 2);
    for circ in circs {
        assert!(circ["confirmed_at"].is_null());
    }

    // 文書ステータスが circulating になっている
    let doc_status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_status, "circulating");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_circulations_on_draft_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient = helpers::insert_employee(&pool, "RCP001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    // ステータスは draft のまま
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/circulations"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "recipient_ids": [recipient.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_circulations_viewer_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;
    let recipient = helpers::insert_employee(&pool, "RCP001", "general").await;
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
                .uri(format!("/api/v1/documents/{doc_id}/circulations"))
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "recipient_ids": [recipient.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── POST /documents/{id}/circulations/confirm ──────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn confirm_circulation_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient = helpers::insert_employee(&pool, "RCP001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    helpers::insert_circulation(&pool, doc_id, recipient.id).await;
    sqlx::query("UPDATE documents SET status = 'circulating' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/circulations/confirm"))
                .header(
                    "Authorization",
                    format!("Bearer {}", recipient.employee_code),
                )
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert!(body["confirmed_at"].is_string());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn confirm_all_changes_doc_to_completed(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient = helpers::insert_employee(&pool, "RCP001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    // 1人だけの回覧
    helpers::insert_circulation(&pool, doc_id, recipient.id).await;
    sqlx::query("UPDATE documents SET status = 'circulating' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/circulations/confirm"))
                .header(
                    "Authorization",
                    format!("Bearer {}", recipient.employee_code),
                )
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 文書ステータスが completed になっている
    let doc_status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_status, "completed");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn confirm_non_recipient_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let recipient = helpers::insert_employee(&pool, "RCP001", "general").await;
    let other = helpers::insert_employee(&pool, "OTH001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id =
        helpers::insert_document(&pool, "内設計-2603001", "テスト", admin.id, kind, proj).await;

    helpers::insert_circulation(&pool, doc_id, recipient.id).await;
    sqlx::query("UPDATE documents SET status = 'circulating' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/circulations/confirm"))
                .header("Authorization", format!("Bearer {}", other.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
