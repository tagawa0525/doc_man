use std::fmt::Write;

use super::client::{self, ApiError};
use super::types::{
    CreateProjectRequest, PaginatedResponse, ProjectResponse, UpdateProjectRequest,
};
use uuid::Uuid;

pub async fn list(
    page: u32,
    per_page: u32,
    q: &str,
) -> Result<PaginatedResponse<ProjectResponse>, ApiError> {
    let mut url = format!("/api/v1/projects?page={page}&per_page={per_page}");
    if !q.is_empty() {
        let _ = write!(url, "&q={}", super::encode_query(q));
    }
    client::get(&url).await
}

pub async fn get(id: Uuid) -> Result<ProjectResponse, ApiError> {
    client::get(&format!("/api/v1/projects/{id}")).await
}

pub async fn create(req: &CreateProjectRequest) -> Result<ProjectResponse, ApiError> {
    client::post("/api/v1/projects", req).await
}

pub async fn update(id: Uuid, req: &UpdateProjectRequest) -> Result<ProjectResponse, ApiError> {
    client::put(&format!("/api/v1/projects/{id}"), req).await
}

pub async fn delete(id: Uuid) -> Result<(), ApiError> {
    client::delete(&format!("/api/v1/projects/{id}")).await
}
