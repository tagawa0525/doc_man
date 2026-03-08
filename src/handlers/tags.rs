use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::tag::{CreateTagRequest, TagResponse};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

/// GET /api/v1/tags
pub async fn list_tags(
    _user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<TagResponse>>, AppError> {
    if let Err(e) = params.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let rows = sqlx::query("SELECT id, name FROM tags ORDER BY name LIMIT $1 OFFSET $2")
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    use sqlx::Row;
    let data: Vec<TagResponse> = rows
        .into_iter()
        .map(|r| TagResponse {
            id: r.get("id"),
            name: r.get("name"),
        })
        .collect();

    Ok(Json(PaginatedResponse::new(
        data,
        total,
        params.page,
        params.per_page,
    )))
}

/// POST /api/v1/tags
pub async fn create_tag(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagResponse>), AppError> {
    if user.role == Role::Viewer {
        return Err(AppError::Forbidden(
            "viewer role cannot create tags".to_string(),
        ));
    }

    let row = sqlx::query("INSERT INTO tags (name) VALUES ($1) RETURNING id, name")
        .bind(&req.name)
        .fetch_one(&state.db)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => match db_err.constraint() {
                Some("tags_name_unique") => {
                    AppError::Conflict(format!("tag '{}' already exists", req.name))
                }
                _ => AppError::Database(e),
            },
            _ => AppError::Database(e),
        })?;

    use sqlx::Row;
    Ok((
        StatusCode::CREATED,
        Json(TagResponse {
            id: row.get("id"),
            name: row.get("name"),
        }),
    ))
}
