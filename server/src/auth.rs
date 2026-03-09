use axum::extract::FromRef;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "project_manager")]
    ProjectManager,
    #[serde(rename = "general")]
    General,
    #[serde(rename = "viewer")]
    Viewer,
}

impl std::str::FromStr for Role {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "project_manager" => Ok(Role::ProjectManager),
            "general" => Ok(Role::General),
            "viewer" => Ok(Role::Viewer),
            _ => Err(AppError::Unauthorized),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: Role,
    pub is_active: bool,
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let app_state = AppState::from_ref(state);
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(std::borrow::ToOwned::to_owned);

        async move {
            let auth_header = auth_header.ok_or(AppError::Unauthorized)?;
            if !auth_header.starts_with("Bearer ") {
                return Err(AppError::Unauthorized);
            }
            let token = &auth_header[7..];

            let row =
                sqlx::query("SELECT id, role, is_active FROM employees WHERE employee_code = $1")
                    .bind(token)
                    .fetch_optional(&app_state.db)
                    .await
                    .map_err(AppError::Database)?
                    .ok_or(AppError::Unauthorized)?;

            let is_active: bool = row.get("is_active");
            if !is_active {
                return Err(AppError::Unauthorized);
            }

            let role: Role = row.get::<String, _>("role").parse()?;

            Ok(AuthenticatedUser {
                id: row.get("id"),
                role,
                is_active,
            })
        }
    }
}
