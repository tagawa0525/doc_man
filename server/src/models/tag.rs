use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}
