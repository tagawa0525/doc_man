use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::handlers::departments;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/me", get(me))
        .route(
            "/api/v1/departments",
            get(departments::list_departments).post(departments::create_department),
        )
        .route(
            "/api/v1/departments/{id}",
            get(departments::get_department).put(departments::update_department),
        )
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(serde::Serialize)]
struct MeResponse {
    id: Uuid,
    role: serde_json::Value,
}

async fn me(user: AuthenticatedUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: user.id,
        role: json!(user.role),
    })
}
