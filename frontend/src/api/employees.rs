use super::client::{self, ApiError};
use super::types::*;
use uuid::Uuid;

pub async fn list(
    page: u32,
    per_page: u32,
) -> Result<PaginatedResponse<EmployeeResponse>, ApiError> {
    client::get(&format!(
        "/api/v1/employees?page={page}&per_page={per_page}"
    ))
    .await
}

pub async fn list_active() -> Result<PaginatedResponse<EmployeeResponse>, ApiError> {
    client::get("/api/v1/employees?per_page=100").await
}

pub async fn get(id: Uuid) -> Result<EmployeeResponse, ApiError> {
    client::get(&format!("/api/v1/employees/{id}")).await
}

pub async fn create(req: &CreateEmployeeRequest) -> Result<EmployeeResponse, ApiError> {
    client::post("/api/v1/employees", req).await
}

pub async fn update(id: Uuid, req: &UpdateEmployeeRequest) -> Result<EmployeeResponse, ApiError> {
    client::put(&format!("/api/v1/employees/{id}"), req).await
}
