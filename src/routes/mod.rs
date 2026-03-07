use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::handlers::departments;
use crate::handlers::disciplines;
use crate::handlers::document_kinds;
use crate::handlers::document_registers;
use crate::handlers::employees;
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
        .route(
            "/api/v1/employees",
            get(employees::list_employees).post(employees::create_employee),
        )
        .route(
            "/api/v1/employees/{id}",
            get(employees::get_employee).put(employees::update_employee),
        )
        .route(
            "/api/v1/disciplines",
            get(disciplines::list_disciplines).post(disciplines::create_discipline),
        )
        .route(
            "/api/v1/disciplines/{id}",
            get(disciplines::get_discipline).put(disciplines::update_discipline),
        )
        .route(
            "/api/v1/document-kinds",
            get(document_kinds::list_document_kinds).post(document_kinds::create_document_kind),
        )
        .route(
            "/api/v1/document-kinds/{id}",
            get(document_kinds::get_document_kind).put(document_kinds::update_document_kind),
        )
        .route(
            "/api/v1/document-registers",
            get(document_registers::list_document_registers)
                .post(document_registers::create_document_register),
        )
        .route(
            "/api/v1/document-registers/{id}",
            get(document_registers::get_document_register)
                .put(document_registers::update_document_register),
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
