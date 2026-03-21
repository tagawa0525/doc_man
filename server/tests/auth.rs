use axum::http::{Request, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn valid_token_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let user = helpers::insert_employee(&pool, "E001", "general").await;

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", format!("Bearer {}", user.employee_code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn invalid_token_returns_401(pool: PgPool) {
    let app = helpers::build_test_app(pool);

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", "Bearer INVALID_CODE")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn inactive_user_returns_401(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let user = helpers::insert_employee_inactive(&pool, "E002", "general").await;

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", format!("Bearer {}", user.employee_code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ── GET /me departments ─────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn me_returns_departments(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let user = helpers::insert_general(&pool).await;
    let dept_a = helpers::insert_department(&pool, "D001", "設計部", None).await;
    let dept_b = helpers::insert_department(&pool, "D002", "製造部", None).await;
    helpers::assign_department(&pool, user.id, dept_a, true).await;
    helpers::assign_department(&pool, user.id, dept_b, false).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header("Authorization", format!("Bearer {}", user.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;

    let departments = body["departments"].as_array().unwrap();
    assert_eq!(departments.len(), 2);

    // code順でソートされること
    assert_eq!(departments[0]["code"], "D001");
    assert_eq!(departments[0]["name"], "設計部");
    assert_eq!(departments[1]["code"], "D002");
    assert_eq!(departments[1]["name"], "製造部");

    // id が UUID として返ること
    assert!(!departments[0]["id"].as_str().unwrap().is_empty());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn me_returns_empty_departments_when_none_assigned(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let user = helpers::insert_general(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header("Authorization", format!("Bearer {}", user.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let departments = body["departments"].as_array().unwrap();
    assert_eq!(departments.len(), 0);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn me_excludes_expired_department_assignments(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let user = helpers::insert_general(&pool).await;
    let dept_active = helpers::insert_department(&pool, "D001", "設計部", None).await;
    let dept_expired = helpers::insert_department(&pool, "D002", "旧製造部", None).await;

    helpers::assign_department(&pool, user.id, dept_active, true).await;
    // 期限切れの所属
    sqlx::query(
        "INSERT INTO employee_departments (employee_id, department_id, is_primary, effective_from, effective_to)
         VALUES ($1, $2, false, '2020-01-01', '2024-12-31')",
    )
    .bind(user.id)
    .bind(dept_expired)
    .execute(&pool)
    .await
    .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/me")
                .header("Authorization", format!("Bearer {}", user.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let departments = body["departments"].as_array().unwrap();
    assert_eq!(departments.len(), 1);
    assert_eq!(departments[0]["code"], "D001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn missing_auth_header_returns_401(pool: PgPool) {
    let app = helpers::build_test_app(pool);

    let request = Request::builder()
        .uri("/api/v1/me")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
