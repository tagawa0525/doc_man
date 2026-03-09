use super::client::{self, ApiError};
use super::types::*;
use uuid::Uuid;

pub async fn list(page: u32, per_page: u32) -> Result<PaginatedResponse<DocumentResponse>, ApiError> {
    client::get(&format!("/api/v1/documents?page={page}&per_page={per_page}")).await
}

pub async fn list_by_project(project_id: Uuid, page: u32, per_page: u32) -> Result<PaginatedResponse<DocumentResponse>, ApiError> {
    client::get(&format!("/api/v1/documents?project_id={project_id}&page={page}&per_page={per_page}")).await
}

pub async fn get(id: Uuid) -> Result<DocumentResponse, ApiError> {
    client::get(&format!("/api/v1/documents/{id}")).await
}

pub async fn create(req: &CreateDocumentRequest) -> Result<DocumentResponse, ApiError> {
    client::post("/api/v1/documents", req).await
}

pub async fn update(id: Uuid, req: &UpdateDocumentRequest) -> Result<DocumentResponse, ApiError> {
    client::put(&format!("/api/v1/documents/{id}"), req).await
}

pub async fn delete(id: Uuid) -> Result<(), ApiError> {
    client::delete(&format!("/api/v1/documents/{id}")).await
}
