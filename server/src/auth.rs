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
    pub name: String,
    pub role: Role,
    pub is_active: bool,
    pub department_ids: Vec<Uuid>,
}

impl AuthenticatedUser {
    /// admin はバイパス、それ以外は所属部署に含まれるかチェック
    pub fn can_access_department(&self, department_id: Uuid) -> bool {
        self.role == Role::Admin || self.department_ids.contains(&department_id)
    }
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

            let row = sqlx::query(
                "SELECT
                    e.id,
                    e.name,
                    e.is_active,
                    COALESCE(
                        e.role,
                        drg.role,
                        p.default_role
                    ) AS effective_role,
                    ARRAY(
                        SELECT ed2.department_id
                        FROM employee_departments ed2
                        WHERE ed2.employee_id = e.id AND ed2.effective_to IS NULL
                    ) AS department_ids
                 FROM employees e
                 JOIN positions p ON p.id = e.position_id
                 LEFT JOIN employee_departments ed
                     ON ed.employee_id = e.id
                     AND ed.effective_to IS NULL
                     AND ed.is_primary = true
                 LEFT JOIN department_role_grants drg
                     ON drg.department_id = ed.department_id
                 WHERE e.employee_code = $1",
            )
            .bind(token)
            .fetch_optional(&app_state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

            let is_active: bool = row.get("is_active");
            if !is_active {
                return Err(AppError::Unauthorized);
            }

            let role: Role = row.get::<String, _>("effective_role").parse()?;
            let department_ids: Vec<Uuid> = row.get("department_ids");

            Ok(AuthenticatedUser {
                id: row.get("id"),
                name: row.get("name"),
                role,
                is_active,
                department_ids,
            })
        }
    }
}
