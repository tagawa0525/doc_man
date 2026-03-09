use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::document_kind::{
    CreateDocumentKindRequest, DocumentKindResponse, DocumentKindRow, UpdateDocumentKindRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

/// GET /api/v1/document-kinds
pub async fn list_document_kinds(
    _user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<DocumentKindResponse>>, AppError> {
    if let Err(e) = params.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM document_kinds")
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let rows = sqlx::query(
        "SELECT id, code, name, seq_digits
         FROM document_kinds
         ORDER BY code
         LIMIT $1 OFFSET $2",
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<DocumentKindResponse> = rows
        .into_iter()
        .map(|r| {
            DocumentKindRow {
                id: r.get("id"),
                code: r.get("code"),
                name: r.get("name"),
                seq_digits: r.get("seq_digits"),
            }
            .into()
        })
        .collect();

    Ok(Json(PaginatedResponse::new(
        data,
        total,
        params.page,
        params.per_page,
    )))
}

/// POST /api/v1/document-kinds
pub async fn create_document_kind(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentKindRequest>,
) -> Result<(StatusCode, Json<DocumentKindResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO document_kinds (code, name, seq_digits)
         VALUES ($1, $2, $3)
         RETURNING id, code, name, seq_digits",
    )
    .bind(&req.code)
    .bind(&req.name)
    .bind(req.seq_digits)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23514") {
                return AppError::InvalidRequest(
                    "invalid document kind data (check constraint violated)".to_string(),
                );
            }
            match db_err.constraint() {
                Some("document_kinds_code_unique") => {
                    AppError::Conflict(format!("document kind code '{}' already exists", req.code))
                }
                _ => AppError::Database(e),
            }
        }
        _ => AppError::Database(e),
    })?;

    let resp = DocumentKindResponse::from(DocumentKindRow {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        seq_digits: row.get("seq_digits"),
    });

    Ok((StatusCode::CREATED, Json(resp)))
}

/// GET /api/v1/document-kinds/{id}
pub async fn get_document_kind(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DocumentKindResponse>, AppError> {
    let row = sqlx::query(
        "SELECT id, code, name, seq_digits
         FROM document_kinds
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("document kind {id} not found")))?;

    let resp = DocumentKindResponse::from(DocumentKindRow {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        seq_digits: row.get("seq_digits"),
    });

    Ok(Json(resp))
}

/// PUT /api/v1/document-kinds/{id}
pub async fn update_document_kind(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDocumentKindRequest>,
) -> Result<Json<DocumentKindResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    if req.code.is_some() {
        return Err(AppError::Unprocessable(
            "code cannot be changed".to_string(),
        ));
    }

    let existing = sqlx::query("SELECT name, seq_digits FROM document_kinds WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("document kind {id} not found")))?;

    let current_name: String = existing.get("name");
    let current_seq_digits: i32 = existing.get("seq_digits");

    let new_name = req.name.unwrap_or(current_name);
    let new_seq_digits = req.seq_digits.unwrap_or(current_seq_digits);

    sqlx::query(
        "UPDATE document_kinds SET name = $1, seq_digits = $2, updated_at = now() WHERE id = $3",
    )
    .bind(&new_name)
    .bind(new_seq_digits)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23514") => {
            AppError::InvalidRequest(
                "invalid document kind data (check constraint violated)".to_string(),
            )
        }
        _ => AppError::Database(e),
    })?;

    let row = sqlx::query(
        "SELECT id, code, name, seq_digits
         FROM document_kinds
         WHERE id = $1",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let resp = DocumentKindResponse::from(DocumentKindRow {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        seq_digits: row.get("seq_digits"),
    });

    Ok(Json(resp))
}
