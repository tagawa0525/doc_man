use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::handlers::approval_steps;
use crate::handlers::departments;
use crate::handlers::disciplines;
use crate::handlers::distributions;
use crate::handlers::document_kinds;
use crate::handlers::document_registers;
use crate::handlers::documents;
use crate::handlers::employees;
use crate::handlers::projects;
use crate::handlers::tags;
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
        .route(
            "/api/v1/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/api/v1/projects/{id}",
            get(projects::get_project)
                .put(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/api/v1/documents",
            get(documents::list_documents).post(documents::create_document),
        )
        .route(
            "/api/v1/documents/{id}",
            get(documents::get_document)
                .put(documents::update_document)
                .delete(documents::delete_document),
        )
        .route(
            "/api/v1/documents/{id}/revise",
            post(documents::revise_document),
        )
        .route(
            "/api/v1/documents/{id}/revisions",
            get(documents::list_document_revisions),
        )
        .route(
            "/api/v1/documents/{doc_id}/approval-steps",
            get(approval_steps::list_approval_steps).post(approval_steps::create_approval_route),
        )
        .route(
            "/api/v1/documents/{doc_id}/approval-steps/{step_id}/approve",
            post(approval_steps::approve_step),
        )
        .route(
            "/api/v1/documents/{doc_id}/approval-steps/{step_id}/reject",
            post(approval_steps::reject_step),
        )
        .route(
            "/api/v1/documents/{doc_id}/distributions",
            get(distributions::list_distributions).post(distributions::create_distributions),
        )
        .route("/api/v1/tags", get(tags::list_tags).post(tags::create_tag))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(serde::Serialize)]
struct MeDepartment {
    id: Uuid,
    code: String,
    name: String,
}

#[derive(serde::Serialize)]
struct MeResponse {
    id: Uuid,
    role: serde_json::Value,
    departments: Vec<MeDepartment>,
}

async fn me(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<MeResponse>, crate::error::AppError> {
    let rows = sqlx::query(
        "SELECT d.id, d.code, d.name
         FROM employee_departments ed
         JOIN departments d ON d.id = ed.department_id
         WHERE ed.employee_id = $1 AND ed.effective_to IS NULL
         ORDER BY d.code",
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(crate::error::AppError::Database)?;

    let departments = rows
        .into_iter()
        .map(|r| MeDepartment {
            id: r.get("id"),
            code: r.get("code"),
            name: r.get("name"),
        })
        .collect();

    Ok(Json(MeResponse {
        id: user.id,
        role: json!(user.role),
        departments,
    }))
}
