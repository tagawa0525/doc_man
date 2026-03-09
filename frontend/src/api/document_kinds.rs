use super::client::{self, ApiError};
use super::types::{
    CreateDocumentKindRequest, DocumentKindResponse, PaginatedResponse, UpdateDocumentKindRequest,
};
use uuid::Uuid;

pub async fn list(
    page: u32,
    per_page: u32,
) -> Result<PaginatedResponse<DocumentKindResponse>, ApiError> {
    client::get(&format!(
        "/api/v1/document-kinds?page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn list_all() -> Result<PaginatedResponse<DocumentKindResponse>, ApiError> {
    client::get("/api/v1/document-kinds?per_page=100").await
}

pub async fn get(id: Uuid) -> Result<DocumentKindResponse, ApiError> {
    client::get(&format!("/api/v1/document-kinds/{id}")).await
}

pub async fn create(req: &CreateDocumentKindRequest) -> Result<DocumentKindResponse, ApiError> {
    client::post("/api/v1/document-kinds", req).await
}

pub async fn update(
    id: Uuid,
    req: &UpdateDocumentKindRequest,
) -> Result<DocumentKindResponse, ApiError> {
    client::put(&format!("/api/v1/document-kinds/{id}"), req).await
}
