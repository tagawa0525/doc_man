use super::client::{self, ApiError};
use super::types::{CirculationResponse, CreateCirculationRequest};
use uuid::Uuid;

pub async fn list(doc_id: Uuid) -> Result<Vec<CirculationResponse>, ApiError> {
    client::get(&format!("/api/v1/documents/{doc_id}/circulations")).await
}

pub async fn create(
    doc_id: Uuid,
    req: &CreateCirculationRequest,
) -> Result<Vec<CirculationResponse>, ApiError> {
    client::post(&format!("/api/v1/documents/{doc_id}/circulations"), req).await
}

pub async fn confirm(doc_id: Uuid) -> Result<CirculationResponse, ApiError> {
    client::post(
        &format!("/api/v1/documents/{doc_id}/circulations/confirm"),
        &serde_json::json!({}),
    )
    .await
}
