use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct PositionResponse {
    pub id: Uuid,
    pub name: String,
    pub default_role: String,
    pub sort_order: i32,
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreatePositionRequest {
    pub name: String,
    pub default_role: String,
    pub sort_order: i32,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdatePositionRequest {
    pub name: Option<String>,
    pub default_role: Option<String>,
    pub sort_order: Option<i32>,
}
