use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::document_register::{
    CreateDocumentRegisterRequest, DocumentRegisterResponse, DocumentRegisterRow,
    UpdateDocumentRegisterRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DocumentRegisterListQuery {
    pub doc_kind_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// GET /api/v1/document-registers
pub async fn list_document_registers(
    _user: AuthenticatedUser,
    Query(params): Query<DocumentRegisterListQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<DocumentRegisterResponse>>, AppError> {
    if let Err(e) = params.pagination.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM document_registers
         WHERE ($1::uuid IS NULL OR doc_kind_id = $1)
           AND ($2::uuid IS NULL OR department_id = $2)",
    )
    .bind(params.doc_kind_id)
    .bind(params.department_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let rows = sqlx::query(
        "SELECT dr.id, dr.register_code, dr.file_server_root,
                dr.new_doc_sub_path, dr.doc_number_pattern,
                dk.id as doc_kind_id, dk.code as doc_kind_code, dk.name as doc_kind_name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name
         FROM document_registers dr
         JOIN document_kinds dk ON dk.id = dr.doc_kind_id
         JOIN departments d ON d.id = dr.department_id
         WHERE ($1::uuid IS NULL OR dr.doc_kind_id = $1)
           AND ($2::uuid IS NULL OR dr.department_id = $2)
         ORDER BY dr.register_code
         LIMIT $3 OFFSET $4",
    )
    .bind(params.doc_kind_id)
    .bind(params.department_id)
    .bind(params.pagination.limit())
    .bind(params.pagination.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<DocumentRegisterResponse> = rows
        .into_iter()
        .map(|r| {
            DocumentRegisterRow {
                id: r.get("id"),
                register_code: r.get("register_code"),
                file_server_root: r.get("file_server_root"),
                new_doc_sub_path: r.get("new_doc_sub_path"),
                doc_number_pattern: r.get("doc_number_pattern"),
                doc_kind_id: r.get("doc_kind_id"),
                doc_kind_code: r.get("doc_kind_code"),
                doc_kind_name: r.get("doc_kind_name"),
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

/// POST /api/v1/document-registers
pub async fn create_document_register(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRegisterRequest>,
) -> Result<(StatusCode, Json<DocumentRegisterResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO document_registers (register_code, doc_kind_id, department_id, file_server_root, new_doc_sub_path, doc_number_pattern)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(&req.register_code)
    .bind(req.doc_kind_id)
    .bind(req.department_id)
    .bind(&req.file_server_root)
    .bind(&req.new_doc_sub_path)
    .bind(&req.doc_number_pattern)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            match db_err.constraint() {
                Some("document_registers_register_code_unique") => {
                    AppError::Conflict(format!(
                        "register code '{}' already exists",
                        req.register_code
                    ))
                }
                Some("document_registers_kind_dept_unique") => {
                    AppError::Conflict(
                        "this doc_kind_id and department_id combination already exists".to_string(),
                    )
                }
                _ => {
                    if db_err.code().as_deref() == Some("23503") {
                        AppError::InvalidRequest(
                            "referenced doc_kind_id or department_id does not exist".to_string(),
                        )
                    } else {
                        AppError::Database(e)
                    }
                }
            }
        }
        _ => AppError::Database(e),
    })?;

    let id: Uuid = row.get("id");
    let reg = fetch_register_by_id(&state, id).await?.ok_or_else(|| {
        AppError::Internal("failed to fetch created document register".to_string())
    })?;

    Ok((StatusCode::CREATED, Json(reg)))
}

/// GET /api/v1/document-registers/{id}
pub async fn get_document_register(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DocumentRegisterResponse>, AppError> {
    let reg = fetch_register_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("document register {id} not found")))?;

    Ok(Json(reg))
}

/// PUT /api/v1/document-registers/{id}
pub async fn update_document_register(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDocumentRegisterRequest>,
) -> Result<Json<DocumentRegisterResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    if req.register_code.is_some() {
        return Err(AppError::Unprocessable(
            "register_code cannot be changed".to_string(),
        ));
    }

    let existing = sqlx::query(
        "SELECT file_server_root, new_doc_sub_path, doc_number_pattern
         FROM document_registers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("document register {id} not found")))?;

    let current_root: String = existing.get("file_server_root");
    let current_sub: Option<String> = existing.get("new_doc_sub_path");
    let current_pattern: Option<String> = existing.get("doc_number_pattern");

    let new_root = req.file_server_root.unwrap_or(current_root);
    let new_sub = req.new_doc_sub_path.or(current_sub);
    let new_pattern = req.doc_number_pattern.or(current_pattern);

    sqlx::query(
        "UPDATE document_registers
         SET file_server_root = $1, new_doc_sub_path = $2, doc_number_pattern = $3, updated_at = now()
         WHERE id = $4",
    )
    .bind(&new_root)
    .bind(&new_sub)
    .bind(&new_pattern)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    let reg = fetch_register_by_id(&state, id).await?.ok_or_else(|| {
        AppError::Internal("failed to fetch updated document register".to_string())
    })?;

    Ok(Json(reg))
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_register_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<Option<DocumentRegisterResponse>, AppError> {
    let row = sqlx::query(
        "SELECT dr.id, dr.register_code, dr.file_server_root,
                dr.new_doc_sub_path, dr.doc_number_pattern,
                dk.id as doc_kind_id, dk.code as doc_kind_code, dk.name as doc_kind_name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name
         FROM document_registers dr
         JOIN document_kinds dk ON dk.id = dr.doc_kind_id
         JOIN departments d ON d.id = dr.department_id
         WHERE dr.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(row.map(|r| {
        DocumentRegisterRow {
            id: r.get("id"),
            register_code: r.get("register_code"),
            file_server_root: r.get("file_server_root"),
            new_doc_sub_path: r.get("new_doc_sub_path"),
            doc_number_pattern: r.get("doc_number_pattern"),
            doc_kind_id: r.get("doc_kind_id"),
            doc_kind_code: r.get("doc_kind_code"),
            doc_kind_name: r.get("doc_kind_name"),
            dept_id: r.get("dept_id"),
            dept_code: r.get("dept_code"),
            dept_name: r.get("dept_name"),
        }
        .into()
    }))
}
