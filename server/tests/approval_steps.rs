use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /documents/{id}/approval-steps ─────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_approval_steps_returns_empty_array(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト文書", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
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

// ── POST /documents/{id}/approval-steps ────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_approval_steps_sets_route_and_changes_status(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver1 = helpers::insert_employee(&pool, "APR001", "general").await;
    let approver2 = helpers::insert_employee(&pool, "APR002", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト文書", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "steps": [
                            { "step_order": 1, "approver_id": approver1.id },
                            { "step_order": 2, "approver_id": approver2.id }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    let steps = body.as_array().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["route_revision"], 1);
    assert_eq!(steps[0]["step_order"], 1);
    assert_eq!(steps[0]["status"], "pending");
    assert_eq!(steps[1]["step_order"], 2);

    // 文書のステータスが under_review になっていることを確認
    let doc_status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_status, "under_review");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_approval_steps_viewer_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "steps": [
                            { "step_order": 1, "approver_id": approver.id }
                        ]
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
async fn post_approval_steps_on_approved_doc_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;

    // ステータスを approved に変更
    sqlx::query("UPDATE documents SET status = 'approved' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "steps": [
                            { "step_order": 1, "approver_id": approver.id }
                        ]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── POST /documents/{id}/approval-steps/{step_id}/approve ──────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn approve_step_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;
    let step_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver.id).await;

    // ステータスを under_review に変更
    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step_id}/approve"
                ))
                .header(
                    "Authorization",
                    format!("Bearer {}", approver.employee_code),
                )
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "確認しました" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["status"], "approved");
    assert!(body["approved_at"].is_string());
    assert_eq!(body["comment"], "確認しました");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn approve_last_step_changes_doc_to_approved(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;

    // 1ステップのみの承認ルート
    let step_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver.id).await;
    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step_id}/approve"
                ))
                .header(
                    "Authorization",
                    format!("Bearer {}", approver.employee_code),
                )
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "OK" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 文書のステータスが approved になっている
    let doc_status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_status, "approved");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn approve_by_non_approver_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let other = helpers::insert_employee(&pool, "OTH001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;
    let step_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver.id).await;

    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step_id}/approve"
                ))
                .header("Authorization", format!("Bearer {}", other.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "OK" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn approve_non_active_step_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver1 = helpers::insert_employee(&pool, "APR001", "general").await;
    let approver2 = helpers::insert_employee(&pool, "APR002", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;

    // 2ステップの承認ルート
    helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver1.id).await;
    let step2_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 2, approver2.id).await;

    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    // step2（非アクティブ）に対して承認を試みる
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step2_id}/approve"
                ))
                .header(
                    "Authorization",
                    format!("Bearer {}", approver2.employee_code),
                )
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "OK" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── POST /documents/{id}/approval-steps/{step_id}/reject ───────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn reject_step_changes_doc_to_rejected(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver1 = helpers::insert_employee(&pool, "APR001", "general").await;
    let approver2 = helpers::insert_employee(&pool, "APR002", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;

    let step1_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver1.id).await;
    helpers::insert_approval_step(&pool, doc_id, 1, 1, 2, approver2.id).await;

    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step1_id}/reject"
                ))
                .header(
                    "Authorization",
                    format!("Bearer {}", approver1.employee_code),
                )
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "寸法を修正してください" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["status"], "rejected");
    assert_eq!(body["comment"], "寸法を修正してください");

    // 文書のステータスが rejected になっている
    let doc_status: String = sqlx::query_scalar("SELECT status FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(doc_status, "rejected");

    // 残りの pending ステップも rejected になっている
    let remaining_pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM approval_steps WHERE document_id = $1 AND status = 'pending'",
    )
    .bind(doc_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(remaining_pending, 0);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn reject_by_non_approver_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let approver = helpers::insert_employee(&pool, "APR001", "general").await;
    let other = helpers::insert_employee(&pool, "OTH001", "general").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    let doc_id = helpers::insert_document(&pool, 1, "テスト", admin.id, kind, proj).await;
    let step_id = helpers::insert_approval_step(&pool, doc_id, 1, 1, 1, approver.id).await;

    sqlx::query("UPDATE documents SET status = 'under_review' WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/documents/{doc_id}/approval-steps/{step_id}/reject"
                ))
                .header("Authorization", format!("Bearer {}", other.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "comment": "差戻し" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
