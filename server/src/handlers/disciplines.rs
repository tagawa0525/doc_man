use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::discipline::{
    CreateDisciplineRequest, DisciplineResponse, DisciplineRow, UpdateDisciplineRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DisciplineListQuery {
    pub department_id: Option<Uuid>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// GET /api/v1/disciplines
pub async fn list_disciplines(
    _user: AuthenticatedUser,
    Query(params): Query<DisciplineListQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<DisciplineResponse>>, AppError> {
    if let Err(e) = params.pagination.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM disciplines
         WHERE ($1::uuid IS NULL OR department_id = $1)",
    )
    .bind(params.department_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let rows = sqlx::query(
        "SELECT di.id, di.code, di.name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name
         FROM disciplines di
         JOIN departments d ON d.id = di.department_id
         WHERE ($1::uuid IS NULL OR di.department_id = $1)
         ORDER BY di.code
         LIMIT $2 OFFSET $3",
    )
    .bind(params.department_id)
    .bind(params.pagination.limit())
    .bind(params.pagination.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<DisciplineResponse> = rows
        .into_iter()
        .map(|r| {
            DisciplineRow {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                dept_id: r.get("dept_id"),
                dept_code: r.get("dept_code"),
                dept_name: r.get("dept_name"),
            }
            .into()
        })
        .collect();

    Ok(Json(PaginatedResponse::new(
        data,
        total,
        params.pagination.page,
        params.pagination.per_page,
    )))
}

/// POST /api/v1/disciplines
pub async fn create_discipline(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDisciplineRequest>,
) -> Result<(StatusCode, Json<DisciplineResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO disciplines (code, name, department_id)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&req.code)
    .bind(&req.name)
    .bind(req.department_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.constraint() {
            Some("disciplines_code_unique") => {
                AppError::Conflict(format!("discipline code '{}' already exists", req.code))
            }
            _ => {
                if db_err.code().as_deref() == Some("23503") {
                    AppError::InvalidRequest(format!(
                        "department_id '{}' does not exist",
                        req.department_id
                    ))
                } else {
                    AppError::Database(e)
                }
            }
        },
        _ => AppError::Database(e),
    })?;

    let id: Uuid = row.get("id");
    let disc = fetch_discipline_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch created discipline".to_string()))?;

    Ok((StatusCode::CREATED, Json(disc)))
}

/// GET /api/v1/disciplines/{id}
pub async fn get_discipline(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DisciplineResponse>, AppError> {
    let disc = fetch_discipline_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("discipline {id} not found")))?;

    Ok(Json(disc))
}

/// PUT /api/v1/disciplines/{id}
pub async fn update_discipline(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDisciplineRequest>,
) -> Result<Json<DisciplineResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    // code 変更は不可
    if req.code.is_some() {
        return Err(AppError::Unprocessable(
            "code cannot be changed".to_string(),
        ));
    }

    let existing = sqlx::query("SELECT name, department_id FROM disciplines WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("discipline {id} not found")))?;

    let current_name: String = existing.get("name");
    let current_dept_id: Uuid = existing.get("department_id");

    let new_name = req.name.unwrap_or(current_name);
    let new_dept_id = req.department_id.unwrap_or(current_dept_id);

    sqlx::query(
        "UPDATE disciplines SET name = $1, department_id = $2, updated_at = now() WHERE id = $3",
    )
    .bind(&new_name)
    .bind(new_dept_id)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23503") => {
            AppError::InvalidRequest(format!("department_id '{new_dept_id}' does not exist"))
        }
        _ => AppError::Database(e),
    })?;

    let disc = fetch_discipline_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch updated discipline".to_string()))?;

    Ok(Json(disc))
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_discipline_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<Option<DisciplineResponse>, AppError> {
    let row = sqlx::query(
        "SELECT di.id, di.code, di.name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name
         FROM disciplines di
         JOIN departments d ON d.id = di.department_id
         WHERE di.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(row.map(|r| {
        DisciplineRow {
            id: r.get("id"),
            code: r.get("code"),
            name: r.get("name"),
            dept_id: r.get("dept_id"),
            dept_code: r.get("dept_code"),
            dept_name: r.get("dept_name"),
        }
        .into()
    }))
}
