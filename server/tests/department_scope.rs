use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── projects: 部署スコープ ───────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn pm_can_create_project_in_own_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    helpers::assign_department(&pool, pm.id, dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "自部署プロジェクト", "discipline_id": disc, "start_date": "2025-04-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn pm_cannot_create_project_in_other_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let own_dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let other_dept = helpers::insert_department(&pool, "品管", "品質管理部", None).await;
    let other_disc = helpers::insert_discipline(&pool, "QA", "品質管理", other_dept).await;
    helpers::assign_department(&pool, pm.id, own_dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "他部署プロジェクト", "discipline_id": other_disc, "start_date": "2025-04-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn admin_can_create_project_in_any_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "品管", "品質管理部", None).await;
    let disc = helpers::insert_discipline(&pool, "QA", "品質管理", dept).await;
    // admin はどの部署にも所属していないがバイパスできる

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "管理者プロジェクト", "discipline_id": disc, "start_date": "2025-04-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

// ── documents: 部署スコープ ──────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn general_can_create_document_in_own_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "テスト", disc, None).await;
    helpers::assign_department(&pool, general.id, dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"title": "自部署文書", "doc_kind_id": kind, "project_id": proj})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn general_cannot_create_document_in_other_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let own_dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let other_dept = helpers::insert_department(&pool, "品管", "品質管理部", None).await;
    let other_disc = helpers::insert_discipline(&pool, "QA", "品質管理", other_dept).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(&pool, "他部署プロジェクト", other_disc, None).await;
    helpers::assign_department(&pool, general.id, own_dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"title": "他部署文書", "doc_kind_id": kind, "project_id": proj})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── 複数部署所属 ─────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn user_with_multiple_departments_can_access_both(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept_a = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let dept_b = helpers::insert_department(&pool, "品管", "品質管理部", None).await;
    let disc_a = helpers::insert_discipline(&pool, "MECH", "機械", dept_a).await;
    let disc_b = helpers::insert_discipline(&pool, "QA", "品質管理", dept_b).await;
    helpers::assign_department(&pool, pm.id, dept_a, true).await;
    helpers::assign_department(&pool, pm.id, dept_b, false).await;

    // 部署Aのプロジェクト作成 → 成功
    let resp_a = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "設計プロジェクト", "discipline_id": disc_a, "start_date": "2025-04-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp_a.status(), StatusCode::CREATED);

    // 部署Bのプロジェクト作成 → 成功
    let resp_b = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "品管プロジェクト", "discipline_id": disc_b, "start_date": "2025-04-01"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp_b.status(), StatusCode::CREATED);
}
