use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /projects ───────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_projects_returns_paginated_list(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    helpers::insert_project(&pool, "プロジェクトA", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["name"], "プロジェクトA");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_projects_with_status_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    helpers::insert_project(&pool, "プロジェクトA", disc, None).await;
    helpers::insert_project_with_status(&pool, "プロジェクトB", disc, None, "completed").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects?status=completed")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["name"], "プロジェクトB");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_projects_with_discipline_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc_a = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let disc_b = helpers::insert_discipline(&pool, "ELEC", "電気", dept).await;
    helpers::insert_project(&pool, "プロジェクトA", disc_a, None).await;
    helpers::insert_project(&pool, "プロジェクトB", disc_b, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/projects?discipline_id={}", disc_a))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["name"], "プロジェクトA");
}

// ── POST /projects ──────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_project_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "新プロジェクト",
                        "discipline_id": disc,
                        "wbs_code": "P-2026-001"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "新プロジェクト");
    assert_eq!(body["status"], "planning");
    assert_eq!(body["discipline"]["code"], "MECH");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_project_project_manager_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "PMプロジェクト",
                        "discipline_id": disc
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_project_general_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "テスト",
                        "discipline_id": disc
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
async fn post_project_duplicate_wbs_code_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    helpers::insert_project_with_wbs(&pool, "既存", disc, None, "P-2026-001").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "重複",
                        "discipline_id": disc,
                        "wbs_code": "P-2026-001"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── GET /projects/{id} ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_project_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "プロジェクトA", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "プロジェクトA");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_project_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /projects/{id} ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_project_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "プロジェクトA", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "変更後名称", "status": "active" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "変更後名称");
    assert_eq!(body["status"], "active");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_project_manager_own_project_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "PMプロジェクト", disc, Some(pm.id)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "PM変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "PM変更");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_project_manager_other_project_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let other_pm = helpers::insert_employee(&pool, "PM002", "project_manager").await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "他PMプロジェクト", disc, Some(other_pm.id)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "不正変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_project_general_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "テスト", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "不正" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── DELETE /projects/{id} ───────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_project_admin_returns_204(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "削除対象", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_project_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let proj_id = helpers::insert_project(&pool, "テスト", disc, None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/projects/{}", proj_id))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_project_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/v1/projects/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
