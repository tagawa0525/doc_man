use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::position::{CreatePositionRequest, PositionResponse, UpdatePositionRequest};
use crate::state::AppState;

/// GET /api/v1/positions
pub async fn list_positions(
    _user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<PositionResponse>>, AppError> {
    let rows =
        sqlx::query("SELECT id, name, default_role, sort_order FROM positions ORDER BY sort_order")
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?;

    let data: Vec<PositionResponse> = rows
        .into_iter()
        .map(|r| PositionResponse {
            id: r.get("id"),
            name: r.get("name"),
            default_role: r.get("default_role"),
            sort_order: r.get("sort_order"),
        })
        .collect();

    Ok(Json(data))
}

/// GET /api/v1/positions/{id}
pub async fn get_position(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<PositionResponse>, AppError> {
    let row = sqlx::query("SELECT id, name, default_role, sort_order FROM positions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("position {id} not found")))?;

    Ok(Json(PositionResponse {
        id: row.get("id"),
        name: row.get("name"),
        default_role: row.get("default_role"),
        sort_order: row.get("sort_order"),
    }))
}

/// POST /api/v1/positions
pub async fn create_position(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreatePositionRequest>,
) -> Result<(StatusCode, Json<PositionResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO positions (name, default_role, sort_order)
         VALUES ($1, $2, $3)
         RETURNING id, name, default_role, sort_order",
    )
    .bind(&req.name)
    .bind(&req.default_role)
    .bind(req.sort_order)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.constraint() {
            Some("positions_name_key") => {
                AppError::Conflict(format!("position '{}' already exists", req.name))
            }
            _ => {
                if db_err.code().as_deref() == Some("23514") {
                    AppError::InvalidRequest(format!(
                        "invalid default_role '{}'",
                        req.default_role
                    ))
                } else {
                    AppError::Database(e)
                }
            }
        },
        _ => AppError::Database(e),
    })?;

    Ok((
        StatusCode::CREATED,
        Json(PositionResponse {
            id: row.get("id"),
            name: row.get("name"),
            default_role: row.get("default_role"),
            sort_order: row.get("sort_order"),
        }),
    ))
}

/// PUT /api/v1/positions/{id}
pub async fn update_position(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdatePositionRequest>,
) -> Result<Json<PositionResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let existing =
        sqlx::query("SELECT name, default_role, sort_order FROM positions WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("position {id} not found")))?;

    let new_name = req.name.unwrap_or_else(|| existing.get("name"));
    let new_default_role = req
        .default_role
        .unwrap_or_else(|| existing.get("default_role"));
    let new_sort_order = req.sort_order.unwrap_or_else(|| existing.get("sort_order"));

    let row = sqlx::query(
        "UPDATE positions SET name = $1, default_role = $2, sort_order = $3, updated_at = now()
         WHERE id = $4
         RETURNING id, name, default_role, sort_order",
    )
    .bind(&new_name)
    .bind(&new_default_role)
    .bind(new_sort_order)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.constraint() {
            Some("positions_name_key") => {
                AppError::Conflict(format!("position '{new_name}' already exists"))
            }
            _ => {
                if db_err.code().as_deref() == Some("23514") {
                    AppError::InvalidRequest(format!(
                        "invalid default_role '{new_default_role}'"
                    ))
                } else {
                    AppError::Database(e)
                }
            }
        },
        _ => AppError::Database(e),
    })?;

    Ok(Json(PositionResponse {
        id: row.get("id"),
        name: row.get("name"),
        default_role: row.get("default_role"),
        sort_order: row.get("sort_order"),
    }))
}
