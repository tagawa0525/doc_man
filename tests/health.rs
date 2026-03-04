use axum::http::StatusCode;
use axum_test::TestServer;
use doc_man::app;

#[tokio::test]
async fn health_check_returns_200() {
    let server = TestServer::new(app()).unwrap();
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}
