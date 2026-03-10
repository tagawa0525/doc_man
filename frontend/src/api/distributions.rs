use uuid::Uuid;

use super::client::{self, ApiError};
use super::types::{CreateDistributionRequest, DistributionResponse};

pub async fn list(doc_id: Uuid) -> Result<Vec<DistributionResponse>, ApiError> {
    client::get(&format!("/api/v1/documents/{doc_id}/distributions")).await
}

pub async fn create(
    doc_id: Uuid,
    req: &CreateDistributionRequest,
) -> Result<Vec<DistributionResponse>, ApiError> {
    client::post(&format!("/api/v1/documents/{doc_id}/distributions"), req).await
}
