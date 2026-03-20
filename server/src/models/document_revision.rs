use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// POST /documents/{id}/revise リクエスト
#[derive(Debug, Deserialize)]
pub struct ReviseDocumentRequest {
    pub reason: String,
}

/// 改訂履歴レスポンス
#[derive(Debug, Serialize)]
pub struct DocumentRevisionResponse {
    pub id: Uuid,
    pub document_id: Uuid,
    pub revision: i32,
    pub file_path: String,
    pub reason: Option<String>,
    pub created_by: CreatedByBrief,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct CreatedByBrief {
    pub id: Uuid,
    pub name: String,
}
