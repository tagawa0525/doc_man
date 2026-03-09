use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- 共通 ---

/// `{ id, code, name }` の共通Brief型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

/// `{ id, name }` の共通Brief型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameBrief {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub id: Uuid,
    pub role: String,
}

// --- Tags ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTagRequest {
    pub name: String,
}

// --- Departments ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentTree {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub children: Vec<DepartmentTree>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub merged_into_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDepartmentRequest {
    pub code: String,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub effective_from: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub effective_to: Option<NaiveDate>,
    pub merged_into_id: Option<Uuid>,
}

// --- Employees ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeResponse {
    pub id: Uuid,
    pub name: String,
    pub employee_code: Option<String>,
    pub ad_account: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub current_department: Option<NameBrief>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub employee_code: Option<String>,
    pub ad_account: Option<String>,
    pub role: Option<String>,
    pub department_id: Uuid,
    pub effective_from: NaiveDate,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub ad_account: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

// --- Disciplines ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisciplineResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub department: CodeBrief,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDisciplineRequest {
    pub code: String,
    pub name: String,
    pub department_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDisciplineRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub department_id: Option<Uuid>,
}

// --- Document Kinds ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentKindResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub seq_digits: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDocumentKindRequest {
    pub code: String,
    pub name: String,
    pub seq_digits: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDocumentKindRequest {
    pub code: Option<String>,
    pub name: Option<String>,
    pub seq_digits: Option<i32>,
}

// --- Document Registers ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRegisterResponse {
    pub id: Uuid,
    pub register_code: String,
    pub file_server_root: String,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
    pub doc_kind: CodeBrief,
    pub department: CodeBrief,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDocumentRegisterRequest {
    pub register_code: String,
    pub doc_kind_id: Uuid,
    pub department_id: Uuid,
    pub file_server_root: String,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDocumentRegisterRequest {
    pub register_code: Option<String>,
    pub file_server_root: Option<String>,
    pub new_doc_sub_path: Option<String>,
    pub doc_number_pattern: Option<String>,
}

// --- Projects ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDisciplineBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub department: CodeBrief,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline: ProjectDisciplineBrief,
    pub manager: Option<NameBrief>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline_id: Uuid,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub wbs_code: Option<String>,
    pub discipline_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
}

// --- Documents ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub doc_number: String,
    pub revision: i32,
    pub title: String,
    pub file_path: String,
    pub status: String,
    pub confidentiality: String,
    pub frozen_dept_code: String,
    pub author: NameBrief,
    pub doc_kind: CodeBrief,
    pub project: NameBrief,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub file_path: String,
    pub confidentiality: Option<String>,
    pub doc_kind_id: Uuid,
    pub project_id: Uuid,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDocumentRequest {
    pub doc_number: Option<String>,
    pub frozen_dept_code: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub file_path: Option<String>,
    pub confidentiality: Option<String>,
    pub tags: Option<Vec<String>>,
}

// --- Approval Steps ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStepResponse {
    pub id: Uuid,
    pub route_revision: i32,
    pub document_revision: i32,
    pub step_order: i32,
    pub approver: NameBrief,
    pub status: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepInput {
    pub step_order: i32,
    pub approver_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateApprovalRouteRequest {
    pub steps: Vec<StepInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApprovalActionRequest {
    pub comment: Option<String>,
}

// --- Circulations ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CirculationResponse {
    pub id: Uuid,
    pub recipient: NameBrief,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCirculationRequest {
    pub recipient_ids: Vec<Uuid>,
}

/// `DepartmentTree` をフラットな `(id, label)` リストに変換する。
/// ラベルは階層を ` > ` で連結し、ルートにはコードを付与する。
pub fn flatten_dept_tree(
    depts: &[DepartmentTree],
    result: &mut Vec<(String, String)>,
    prefix: &str,
) {
    for d in depts {
        let label = if prefix.is_empty() {
            format!("{} ({})", d.name, d.code)
        } else {
            format!("{prefix} > {}", d.name)
        };
        result.push((d.id.to_string(), label));
        let next_prefix = if prefix.is_empty() {
            d.name.clone()
        } else {
            format!("{prefix} > {}", d.name)
        };
        flatten_dept_tree(&d.children, result, &next_prefix);
    }
}
