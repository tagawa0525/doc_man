use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct DistributionResponse {
    pub id: Uuid,
    pub recipient: RecipientBrief,
    pub distributed_by: DistributedByBrief,
    pub distributed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RecipientBrief {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DistributedByBrief {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDistributionRequest {
    pub recipient_ids: Vec<Uuid>,
}
