use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

pub fn build_router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
