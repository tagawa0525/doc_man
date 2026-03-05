use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use doc_man::app;
use tower::ServiceExt;

#[tokio::test]
async fn health_check_returns_200() {
    let app = app();
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
