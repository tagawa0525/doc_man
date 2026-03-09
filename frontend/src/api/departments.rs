use super::client::{self, ApiError};
use super::types::{
    CreateDepartmentRequest, DepartmentResponse, DepartmentTree, UpdateDepartmentRequest,
};
use uuid::Uuid;

pub async fn list() -> Result<Vec<DepartmentTree>, ApiError> {
    client::get("/api/v1/departments").await
}

pub async fn list_include_inactive() -> Result<Vec<DepartmentTree>, ApiError> {
    client::get("/api/v1/departments?include_inactive=true").await
}

pub async fn get(id: Uuid) -> Result<DepartmentResponse, ApiError> {
    client::get(&format!("/api/v1/departments/{id}")).await
}

pub async fn create(req: &CreateDepartmentRequest) -> Result<DepartmentResponse, ApiError> {
    client::post("/api/v1/departments", req).await
}

pub async fn update(
    id: Uuid,
    req: &UpdateDepartmentRequest,
) -> Result<DepartmentResponse, ApiError> {
    client::put(&format!("/api/v1/departments/{id}"), req).await
}
