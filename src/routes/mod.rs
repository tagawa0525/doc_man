use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/me", get(me))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(serde::Serialize)]
struct MeResponse {
    id: Uuid,
    role: String,
}

async fn me(user: AuthenticatedUser) -> Json<MeResponse> {
    Json(MeResponse {
        id: user.id,
        role: format!("{:?}", user.role).to_lowercase(),
    })
}
