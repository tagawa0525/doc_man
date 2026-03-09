use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::DepartmentBrief;

/// DB行型（projects + disciplines + departments + employees JOIN）
#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub wbs_code: Option<String>,
    pub disc_id: Uuid,
    pub disc_code: String,
    pub disc_name: String,
    pub dept_id: Uuid,
    pub dept_code: String,
    pub dept_name: String,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DisciplineBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub department: DepartmentBrief,
}

#[derive(Debug, Serialize)]
pub struct ManagerBrief {
    pub id: Uuid,
    pub name: String,
}

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline: DisciplineBrief,
    pub manager: Option<ManagerBrief>,
}

impl From<ProjectRow> for ProjectResponse {
    fn from(row: ProjectRow) -> Self {
        ProjectResponse {
            id: row.id,
            name: row.name,
            status: row.status,
            start_date: row.start_date,
            end_date: row.end_date,
            wbs_code: row.wbs_code,
            discipline: DisciplineBrief {
                id: row.disc_id,
                code: row.disc_code,
                name: row.disc_name,
                department: DepartmentBrief {
                    id: row.dept_id,
                    code: row.dept_code,
                    name: row.dept_name,
                },
            },
            manager: row.manager_id.map(|id| ManagerBrief {
                id,
                name: row.manager_name.unwrap_or_default(),
            }),
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub status: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline_id: Uuid,
    pub manager_id: Option<Uuid>,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
}
