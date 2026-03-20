use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::AppError;

/// discipline_id から所属部署IDを解決
pub async fn get_discipline_department_id(
    pool: &PgPool,
    discipline_id: Uuid,
) -> Result<Uuid, AppError> {
    let row = sqlx::query("SELECT department_id FROM disciplines WHERE id = $1")
        .bind(discipline_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::InvalidRequest(format!("discipline_id '{discipline_id}' does not exist"))
        })?;

    Ok(row.get("department_id"))
}

/// project_id → discipline → department_id を解決
pub async fn get_project_department_id(pool: &PgPool, project_id: Uuid) -> Result<Uuid, AppError> {
    let row = sqlx::query(
        "SELECT d.department_id
         FROM projects p
         JOIN disciplines d ON d.id = p.discipline_id
         WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("project {project_id} not found")))?;

    Ok(row.get("department_id"))
}

/// document_id → project → discipline → department_id を解決
pub async fn get_document_department_id(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Uuid, AppError> {
    let row = sqlx::query(
        "SELECT di.department_id
         FROM documents doc
         JOIN projects p ON p.id = doc.project_id
         JOIN disciplines di ON di.id = p.discipline_id
         WHERE doc.id = $1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("document {document_id} not found")))?;

    Ok(row.get("department_id"))
}

/// ユーザーがリソースの所属部署にアクセスできるかチェック
pub fn check_department_access(
    user: &AuthenticatedUser,
    department_id: Uuid,
) -> Result<(), AppError> {
    if user.can_access_department(department_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "you do not have access to this department's resources".to_string(),
        ))
    }
}
