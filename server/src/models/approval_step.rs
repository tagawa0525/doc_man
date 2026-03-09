use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApprovalStepResponse {
    pub id: Uuid,
    pub route_revision: i32,
    pub document_revision: i32,
    pub step_order: i32,
    pub approver: ApproverBrief,
    pub status: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApproverBrief {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApprovalRouteRequest {
    pub steps: Vec<StepInput>,
}

#[derive(Debug, Deserialize)]
pub struct StepInput {
    pub step_order: i32,
    pub approver_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalActionRequest {
    pub comment: Option<String>,
}
