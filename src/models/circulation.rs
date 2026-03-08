use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CirculationResponse {
    pub id: Uuid,
    pub recipient: RecipientBrief,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct RecipientBrief {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCirculationRequest {
    pub recipient_ids: Vec<Uuid>,
}
