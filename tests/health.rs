use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn health_check_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool);
    let request = Request::builder()
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}
