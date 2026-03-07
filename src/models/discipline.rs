use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB行型（disciplines + departments JOIN）
#[derive(Debug, Clone)]
pub struct DisciplineRow {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub dept_id: Uuid,
    pub dept_code: String,
    pub dept_name: String,
}

#[derive(Debug, Serialize)]
pub struct DepartmentBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct DisciplineResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub department: DepartmentBrief,
}

impl From<DisciplineRow> for DisciplineResponse {
    fn from(row: DisciplineRow) -> Self {
        DisciplineResponse {
            id: row.id,
            code: row.code,
            name: row.name,
            department: DepartmentBrief {
                id: row.dept_id,
                code: row.dept_code,
                name: row.dept_name,
            },
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDisciplineRequest {
    pub code: String,
    pub name: String,
    pub department_id: Uuid,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateDisciplineRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub department_id: Option<Uuid>,
}
