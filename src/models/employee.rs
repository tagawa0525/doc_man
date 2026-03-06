use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB行型（employees + employee_departments JOIN）
#[derive(Debug, Clone)]
pub struct EmployeeRow {
    pub id: Uuid,
    pub name: String,
    pub employee_code: Option<String>,
    pub ad_account: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub dept_id: Option<Uuid>,
    pub dept_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentSummary {
    pub id: Uuid,
    pub name: String,
}

/// レスポンス型
#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub id: Uuid,
    pub name: String,
    pub employee_code: Option<String>,
    pub ad_account: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub current_department: Option<DepartmentSummary>,
}

impl From<EmployeeRow> for EmployeeResponse {
    fn from(row: EmployeeRow) -> Self {
        EmployeeResponse {
            id: row.id,
            name: row.name,
            employee_code: row.employee_code,
            ad_account: row.ad_account,
            role: row.role,
            is_active: row.is_active,
            current_department: row
                .dept_id
                .zip(row.dept_name)
                .map(|(id, name)| DepartmentSummary { id, name }),
        }
    }
}

/// POST リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub employee_code: Option<String>,
    pub ad_account: Option<String>,
    pub role: Option<String>,
    pub department_id: Uuid,
    pub effective_from: NaiveDate,
}

/// PUT リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub ad_account: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}
