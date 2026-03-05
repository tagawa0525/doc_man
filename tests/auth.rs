use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// テスト用エンドポイント: GET /api/v1/me
// → 認証済みユーザーのIDを返す (PR-1で /api/v1/me を追加する)

#[sqlx::test(migrations = "migrations")]
async fn valid_token_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());

    // テスト用社員を挿入
    let employee_code = "E001";
    sqlx::query!(
        "INSERT INTO employees (name, employee_code, role, is_active) VALUES ($1, $2, $3, $4)",
        "Test User",
        employee_code,
        "general",
        true
    )
    .execute(&pool)
    .await
    .unwrap();

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", format!("Bearer {}", employee_code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "migrations")]
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

#[sqlx::test(migrations = "migrations")]
async fn inactive_user_returns_401(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());

    let employee_code = "E002";
    sqlx::query!(
        "INSERT INTO employees (name, employee_code, role, is_active) VALUES ($1, $2, $3, $4)",
        "Retired User",
        employee_code,
        "general",
        false
    )
    .execute(&pool)
    .await
    .unwrap();

    let request = Request::builder()
        .uri("/api/v1/me")
        .header("Authorization", format!("Bearer {}", employee_code))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "migrations")]
async fn missing_auth_header_returns_401(pool: PgPool) {
    let app = helpers::build_test_app(pool);

    let request = Request::builder()
        .uri("/api/v1/me")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
