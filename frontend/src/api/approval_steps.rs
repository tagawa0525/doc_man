use super::client::{self, ApiError};
use super::types::*;
use uuid::Uuid;

pub async fn list(doc_id: Uuid) -> Result<Vec<ApprovalStepResponse>, ApiError> {
    client::get(&format!("/api/v1/documents/{doc_id}/approval-steps")).await
}

pub async fn create_route(
    doc_id: Uuid,
    req: &CreateApprovalRouteRequest,
) -> Result<Vec<ApprovalStepResponse>, ApiError> {
    client::post(&format!("/api/v1/documents/{doc_id}/approval-steps"), req).await
}

pub async fn approve(
    doc_id: Uuid,
    step_id: Uuid,
    req: &ApprovalActionRequest,
) -> Result<ApprovalStepResponse, ApiError> {
    client::post(
        &format!("/api/v1/documents/{doc_id}/approval-steps/{step_id}/approve"),
        req,
    )
    .await
}

pub async fn reject(
    doc_id: Uuid,
    step_id: Uuid,
    req: &ApprovalActionRequest,
) -> Result<ApprovalStepResponse, ApiError> {
    client::post(
        &format!("/api/v1/documents/{doc_id}/approval-steps/{step_id}/reject"),
        req,
    )
    .await
}
