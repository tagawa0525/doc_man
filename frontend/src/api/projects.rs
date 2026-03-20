use std::fmt::Write;

use super::client::{self, ApiError};
use super::types::{
    CreateProjectRequest, PaginatedResponse, ProjectResponse, UpdateProjectRequest,
};
use uuid::Uuid;

#[derive(Default)]
pub struct ProjectListParams {
    pub page: u32,
    pub per_page: u32,
    pub q: String,
    pub dept_ids: String,
    pub fiscal_years: String,
    pub manager_name: String,
}

pub async fn list_filtered(
    params: &ProjectListParams,
) -> Result<PaginatedResponse<ProjectResponse>, ApiError> {
    let mut url = format!(
        "/api/v1/projects?page={}&per_page={}",
        params.page, params.per_page
    );
    if !params.q.is_empty() {
        let _ = write!(url, "&q={}", super::encode_query(&params.q));
    }
    if !params.dept_ids.is_empty() {
        let _ = write!(url, "&dept_ids={}", params.dept_ids);
    }
    if !params.fiscal_years.is_empty() {
        let _ = write!(url, "&fiscal_years={}", params.fiscal_years);
    }
    if !params.manager_name.is_empty() {
        let _ = write!(
            url,
            "&manager_name={}",
            super::encode_query(&params.manager_name)
        );
    }
    client::get(&url).await
}

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
