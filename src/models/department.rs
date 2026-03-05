use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB行型
#[derive(Debug, Clone)]
pub struct DepartmentRow {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub merged_into_id: Option<Uuid>,
}

/// レスポンス型（GET /departments）ツリー構造
#[derive(Debug, Serialize)]
pub struct DepartmentTree {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub children: Vec<DepartmentTree>,
}

/// レスポンス型（GET/POST/PUT /departments/:id）フラット
#[derive(Debug, Serialize)]
pub struct DepartmentResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}

impl From<DepartmentRow> for DepartmentResponse {
    fn from(row: DepartmentRow) -> Self {
        DepartmentResponse {
            id: row.id,
            code: row.code,
            name: row.name,
            parent_id: row.parent_id,
            effective_from: row.effective_from,
            effective_to: row.effective_to,
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub effective_to: Option<NaiveDate>,
    pub merged_into_id: Option<Uuid>,
}
