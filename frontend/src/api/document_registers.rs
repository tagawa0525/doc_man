use super::client::{self, ApiError};
use super::types::{
    CreateDocumentRegisterRequest, DocumentRegisterResponse, PaginatedResponse,
    UpdateDocumentRegisterRequest,
};
use uuid::Uuid;

pub async fn list(
    page: u32,
    per_page: u32,
) -> Result<PaginatedResponse<DocumentRegisterResponse>, ApiError> {
    client::get(&format!(
        "/api/v1/document-registers?page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn get(id: Uuid) -> Result<DocumentRegisterResponse, ApiError> {
    client::get(&format!("/api/v1/document-registers/{id}")).await
}

pub async fn create(
    req: &CreateDocumentRegisterRequest,
) -> Result<DocumentRegisterResponse, ApiError> {
    client::post("/api/v1/document-registers", req).await
}

pub async fn update(
    id: Uuid,
    req: &UpdateDocumentRegisterRequest,
) -> Result<DocumentRegisterResponse, ApiError> {
    client::put(&format!("/api/v1/document-registers/{id}"), req).await
}
