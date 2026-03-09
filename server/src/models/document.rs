use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DocKindBrief;

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub doc_number: String,
    pub revision: i32,
    pub title: String,
    pub file_path: String,
    pub status: String,
    pub confidentiality: String,
    pub frozen_dept_code: String,
    pub author: AuthorBrief,
    pub doc_kind: DocKindBrief,
    pub project: ProjectBrief,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthorBrief {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectBrief {
    pub id: Uuid,
    pub name: String,
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub file_path: String,
    pub confidentiality: Option<String>,
    pub doc_kind_id: Uuid,
    pub project_id: Uuid,
    pub tags: Option<Vec<String>>,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub doc_number: Option<String>,
    pub frozen_dept_code: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub file_path: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Vec<String>>,
}
