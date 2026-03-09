use super::client::{self, ApiError};
use super::types::*;

pub async fn list(page: u32, per_page: u32) -> Result<PaginatedResponse<TagResponse>, ApiError> {
    client::get(&format!("/api/v1/tags?page={page}&per_page={per_page}")).await
}

pub async fn list_all() -> Result<PaginatedResponse<TagResponse>, ApiError> {
    client::get("/api/v1/tags?per_page=100").await
}

pub async fn create(req: &CreateTagRequest) -> Result<TagResponse, ApiError> {
    client::post("/api/v1/tags", req).await
}
