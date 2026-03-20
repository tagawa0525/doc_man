use super::client::{self, ApiError};
use super::types::{CreatePositionRequest, PositionResponse, UpdatePositionRequest};
use uuid::Uuid;

pub async fn list() -> Result<Vec<PositionResponse>, ApiError> {
    client::get("/api/v1/positions").await
}

pub async fn get(id: Uuid) -> Result<PositionResponse, ApiError> {
    client::get(&format!("/api/v1/positions/{id}")).await
}

pub async fn create(req: &CreatePositionRequest) -> Result<PositionResponse, ApiError> {
    client::post("/api/v1/positions", req).await
}

pub async fn update(id: Uuid, req: &UpdatePositionRequest) -> Result<PositionResponse, ApiError> {
    client::put(&format!("/api/v1/positions/{id}"), req).await
}
