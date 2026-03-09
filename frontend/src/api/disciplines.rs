use super::client::{self, ApiError};
use super::types::*;
use uuid::Uuid;

pub async fn list(page: u32, per_page: u32) -> Result<PaginatedResponse<DisciplineResponse>, ApiError> {
    client::get(&format!("/api/v1/disciplines?page={page}&per_page={per_page}")).await
}

pub async fn list_all() -> Result<PaginatedResponse<DisciplineResponse>, ApiError> {
    client::get("/api/v1/disciplines?per_page=100").await
}

pub async fn get(id: Uuid) -> Result<DisciplineResponse, ApiError> {
    client::get(&format!("/api/v1/disciplines/{id}")).await
}

pub async fn create(req: &CreateDisciplineRequest) -> Result<DisciplineResponse, ApiError> {
    client::post("/api/v1/disciplines", req).await
}

pub async fn update(id: Uuid, req: &UpdateDisciplineRequest) -> Result<DisciplineResponse, ApiError> {
    client::put(&format!("/api/v1/disciplines/{id}"), req).await
}
