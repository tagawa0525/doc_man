use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB行型
#[derive(Debug, Clone)]
pub struct DocumentKindRow {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub seq_digits: i32,
}

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct DocumentKindResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub seq_digits: i32,
}

impl From<DocumentKindRow> for DocumentKindResponse {
    fn from(row: DocumentKindRow) -> Self {
        DocumentKindResponse {
            id: row.id,
            code: row.code,
            name: row.name,
            seq_digits: row.seq_digits,
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDocumentKindRequest {
    pub code: String,
    pub name: String,
    pub seq_digits: i32,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentKindRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub seq_digits: Option<i32>,
}
