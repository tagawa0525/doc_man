use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn valid_token_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());

    sqlx::query(
        "INSERT INTO employees (name, employee_code, role, is_active) VALUES ($1, $2, $3, $4)",
    )
    .bind("Test User")
    .bind("E001")
    .bind("general")
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", "Bearer E001")
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

    sqlx::query(
        "INSERT INTO employees (name, employee_code, role, is_active) VALUES ($1, $2, $3, $4)",
    )
    .bind("Retired User")
    .bind("E002")
    .bind("general")
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", "Bearer E002")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
