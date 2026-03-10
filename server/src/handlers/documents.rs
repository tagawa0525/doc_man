use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::DocKindBrief;
use crate::models::document::{
    AuthorBrief, CreateDocumentRequest, DocumentResponse, ProjectBrief, UpdateDocumentRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::services::document_numbering::assign_doc_number;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DocumentListQuery {
    pub project_id: Option<Uuid>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// GET /api/v1/documents
pub async fn list_documents(
    _user: AuthenticatedUser,
    Query(params): Query<DocumentListQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<DocumentResponse>>, AppError> {
    if let Err(e) = params.pagination.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documents
         WHERE ($1::uuid IS NULL OR project_id = $1)",
    )
    .bind(params.project_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let rows = sqlx::query(
        "SELECT d.id, d.doc_number, d.revision, d.title, d.file_path,
                d.status, d.confidentiality, d.frozen_dept_code,
                d.created_at, d.updated_at,
                e.id AS author_id, e.name AS author_name,
                dk.id AS doc_kind_id, dk.code AS doc_kind_code, dk.name AS doc_kind_name,
                p.id AS project_id, p.name AS project_name
         FROM documents d
         JOIN employees e ON e.id = d.author_id
         JOIN document_kinds dk ON dk.id = d.doc_kind_id
         JOIN projects p ON p.id = d.project_id
         WHERE ($1::uuid IS NULL OR d.project_id = $1)
         ORDER BY d.created_at DESC
         LIMIT $2 OFFSET $3",
    )
    .bind(params.project_id)
    .bind(params.pagination.limit())
    .bind(params.pagination.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    // 全文書IDに対するタグを一括取得（N+1回避）
    let doc_ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();
    let tags_map = fetch_tags_batch(&state.db, &doc_ids).await?;

    let data: Vec<DocumentResponse> = rows
        .into_iter()
        .map(|r| {
            let doc_id: Uuid = r.get("id");
            let tags = tags_map.get(&doc_id).cloned().unwrap_or_default();
            DocumentResponse {
                id: doc_id,
                doc_number: r.get("doc_number"),
                revision: r.get("revision"),
                title: r.get("title"),
                file_path: r.get("file_path"),
                status: r.get("status"),
                confidentiality: r.get("confidentiality"),
                frozen_dept_code: r.get("frozen_dept_code"),
                author: AuthorBrief {
                    id: r.get("author_id"),
                    name: r.get("author_name"),
                },
                doc_kind: DocKindBrief {
                    id: r.get("doc_kind_id"),
                    code: r.get("doc_kind_code"),
                    name: r.get("doc_kind_name"),
                },
                project: ProjectBrief {
                    id: r.get("project_id"),
                    name: r.get("project_name"),
                },
                tags,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(PaginatedResponse::new(
        data,
        total,
        params.pagination.page,
        params.pagination.per_page,
    )))
}

/// POST /api/v1/documents
pub async fn create_document(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<(StatusCode, Json<DocumentResponse>), AppError> {
    if user.role == Role::Viewer {
        return Err(AppError::Forbidden(
            "viewer role cannot create documents".to_string(),
        ));
    }

    // doc_kind の code, seq_digits を取得
    let dk_row = sqlx::query("SELECT code, seq_digits FROM document_kinds WHERE id = $1")
        .bind(req.doc_kind_id)
        .fetch_optional(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| {
            AppError::InvalidRequest(format!("doc_kind_id {} not found", req.doc_kind_id))
        })?;

    let doc_kind_code: String = dk_row.get("code");
    let seq_digits: i32 = dk_row.get("seq_digits");

    // project → discipline → department の code を取得 (= frozen_dept_code)
    let dept_row = sqlx::query(
        "SELECT d.code AS dept_code
         FROM projects p
         JOIN disciplines di ON di.id = p.discipline_id
         JOIN departments d ON d.id = di.department_id
         WHERE p.id = $1",
    )
    .bind(req.project_id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::InvalidRequest(format!("project_id {} not found", req.project_id)))?;

    let dept_code: String = dept_row.get("dept_code");

    // JST の現在日時を取得
    let jst = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
    let now_jst = chrono::Utc::now().with_timezone(&jst).naive_local();

    // トランザクション開始
    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 文書番号の採番
    let doc_number =
        assign_doc_number(tx.as_mut(), &doc_kind_code, &dept_code, seq_digits, now_jst).await?;

    let confidentiality = req.confidentiality.as_deref().unwrap_or("internal");

    let doc_row = sqlx::query(
        "INSERT INTO documents (doc_number, title, file_path, author_id, doc_kind_id, frozen_dept_code, confidentiality, project_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id",
    )
    .bind(&doc_number)
    .bind(&req.title)
    .bind(&req.file_path)
    .bind(user.id)
    .bind(req.doc_kind_id)
    .bind(&dept_code)
    .bind(confidentiality)
    .bind(req.project_id)
    .fetch_one(tx.as_mut())
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
            Some("23514") => {
                AppError::InvalidRequest("invalid document data (check constraint violated)".to_string())
            }
            Some("23503") => {
                AppError::InvalidRequest("referenced entity does not exist".to_string())
            }
            _ => AppError::Database(e),
        },
        _ => AppError::Database(e),
    })?;

    let doc_id: Uuid = doc_row.get("id");

    // タグの関連付け（重複排除）
    if let Some(ref tag_names) = req.tags {
        let unique_tags: Vec<&String> = {
            let mut seen = std::collections::HashSet::new();
            tag_names
                .iter()
                .filter(|t| seen.insert(t.as_str()))
                .collect()
        };
        for tag_name in unique_tags {
            let tag_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
                .bind(tag_name)
                .fetch_optional(tx.as_mut())
                .await
                .map_err(AppError::Database)?;

            if let Some(tid) = tag_id {
                sqlx::query("INSERT INTO document_tags (document_id, tag_id) VALUES ($1, $2)")
                    .bind(doc_id)
                    .bind(tid)
                    .execute(tx.as_mut())
                    .await
                    .map_err(AppError::Database)?;
            }
        }
    }

    tx.commit().await.map_err(AppError::Database)?;

    let doc = fetch_document_by_id(&state, doc_id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch created document".to_string()))?;

    Ok((StatusCode::CREATED, Json(doc)))
}

/// GET /api/v1/documents/{id}
pub async fn get_document(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DocumentResponse>, AppError> {
    let doc = fetch_document_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;

    Ok(Json(doc))
}

/// PUT /api/v1/documents/{id}
pub async fn update_document(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDocumentRequest>,
) -> Result<Json<DocumentResponse>, AppError> {
    if user.role == Role::Viewer {
        return Err(AppError::Forbidden(
            "viewer role cannot update documents".to_string(),
        ));
    }

    // 変更不可フィールドのチェック
    if req.doc_number.is_some() {
        return Err(AppError::Unprocessable(
            "doc_number cannot be changed".to_string(),
        ));
    }
    if req.status.is_some() {
        return Err(AppError::Unprocessable(
            "status cannot be changed via PUT".to_string(),
        ));
    }
    if req.frozen_dept_code.is_some() {
        return Err(AppError::Unprocessable(
            "frozen_dept_code cannot be changed".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    let existing = sqlx::query(
        "SELECT title, file_path, confidentiality, status, revision
         FROM documents WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;

    let current_title: String = existing.get("title");
    let current_file_path: String = existing.get("file_path");
    let current_confidentiality: String = existing.get("confidentiality");
    let current_status: String = existing.get("status");
    let current_revision: i32 = existing.get("revision");

    let new_title = req.title.unwrap_or(current_title.clone());
    let new_file_path = req.file_path.unwrap_or(current_file_path.clone());
    let new_confidentiality = req
        .confidentiality
        .unwrap_or(current_confidentiality.clone());

    // タグの変更を実際に比較
    let tags_changed = if let Some(ref new_tags) = req.tags {
        let current_tags = fetch_tags(&state.db, id).await?;
        let mut sorted_new: Vec<&str> = new_tags.iter().map(std::string::String::as_str).collect();
        sorted_new.sort_unstable();
        sorted_new.dedup();
        let mut sorted_current: Vec<&str> = current_tags
            .iter()
            .map(std::string::String::as_str)
            .collect();
        sorted_current.sort_unstable();
        sorted_new != sorted_current
    } else {
        false
    };

    // draft/rejected 状態で内容変更がある場合に revision +1
    let content_changed = new_title != current_title
        || new_file_path != current_file_path
        || new_confidentiality != current_confidentiality
        || tags_changed;

    let new_revision =
        if content_changed && (current_status == "draft" || current_status == "rejected") {
            current_revision + 1
        } else {
            current_revision
        };

    sqlx::query(
        "UPDATE documents
         SET title = $1, file_path = $2, confidentiality = $3, revision = $4, updated_at = now()
         WHERE id = $5",
    )
    .bind(&new_title)
    .bind(&new_file_path)
    .bind(&new_confidentiality)
    .bind(new_revision)
    .bind(id)
    .execute(tx.as_mut())
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
            Some("23514") => AppError::InvalidRequest(
                "invalid document data (check constraint violated)".to_string(),
            ),
            Some("23503") => {
                AppError::InvalidRequest("referenced entity does not exist".to_string())
            }
            _ => AppError::Database(e),
        },
        _ => AppError::Database(e),
    })?;

    // タグの更新（指定されていれば全件差し替え、重複排除）
    if let Some(ref tag_names) = req.tags {
        sqlx::query("DELETE FROM document_tags WHERE document_id = $1")
            .bind(id)
            .execute(tx.as_mut())
            .await
            .map_err(AppError::Database)?;

        let unique_tags: Vec<&String> = {
            let mut seen = std::collections::HashSet::new();
            tag_names
                .iter()
                .filter(|t| seen.insert(t.as_str()))
                .collect()
        };
        for tag_name in unique_tags {
            let tag_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM tags WHERE name = $1")
                .bind(tag_name)
                .fetch_optional(tx.as_mut())
                .await
                .map_err(AppError::Database)?;

            if let Some(tid) = tag_id {
                sqlx::query("INSERT INTO document_tags (document_id, tag_id) VALUES ($1, $2)")
                    .bind(id)
                    .bind(tid)
                    .execute(tx.as_mut())
                    .await
                    .map_err(AppError::Database)?;
            }
        }
    }

    tx.commit().await.map_err(AppError::Database)?;

    let doc = fetch_document_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch updated document".to_string()))?;

    Ok(Json(doc))
}

/// DELETE /api/v1/documents/{id}
pub async fn delete_document(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    // approval_steps の存在チェック
    let approval_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM approval_steps WHERE document_id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(AppError::Database)?;

    if approval_count > 0 {
        return Err(AppError::Conflict(
            "cannot delete document with approval steps".to_string(),
        ));
    }

    // document_tags を先に削除（FK制約のため）
    sqlx::query("DELETE FROM document_tags WHERE document_id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("document {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_tags(db: &sqlx::PgPool, document_id: Uuid) -> Result<Vec<String>, AppError> {
    let tags: Vec<String> = sqlx::query_scalar(
        "SELECT t.name FROM document_tags dt
         JOIN tags t ON t.id = dt.tag_id
         WHERE dt.document_id = $1
         ORDER BY t.name",
    )
    .bind(document_id)
    .fetch_all(db)
    .await
    .map_err(AppError::Database)?;

    Ok(tags)
}

async fn fetch_tags_batch(
    db: &sqlx::PgPool,
    doc_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<String>>, AppError> {
    if doc_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows = sqlx::query(
        "SELECT dt.document_id, t.name
         FROM document_tags dt
         JOIN tags t ON t.id = dt.tag_id
         WHERE dt.document_id = ANY($1)
         ORDER BY dt.document_id, t.name",
    )
    .bind(doc_ids)
    .fetch_all(db)
    .await
    .map_err(AppError::Database)?;

    let mut map: std::collections::HashMap<Uuid, Vec<String>> = std::collections::HashMap::new();
    for r in rows {
        let doc_id: Uuid = r.get("document_id");
        let tag_name: String = r.get("name");
        map.entry(doc_id).or_default().push(tag_name);
    }

    Ok(map)
}

async fn fetch_document_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<Option<DocumentResponse>, AppError> {
    let row = sqlx::query(
        "SELECT d.id, d.doc_number, d.revision, d.title, d.file_path,
                d.status, d.confidentiality, d.frozen_dept_code,
                d.created_at, d.updated_at,
                e.id AS author_id, e.name AS author_name,
                dk.id AS doc_kind_id, dk.code AS doc_kind_code, dk.name AS doc_kind_name,
                p.id AS project_id, p.name AS project_name
         FROM documents d
         JOIN employees e ON e.id = d.author_id
         JOIN document_kinds dk ON dk.id = d.doc_kind_id
         JOIN projects p ON p.id = d.project_id
         WHERE d.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    match row {
        Some(r) => {
            let doc_id: Uuid = r.get("id");
            let tags = fetch_tags(&state.db, doc_id).await?;
            Ok(Some(DocumentResponse {
                id: doc_id,
                doc_number: r.get("doc_number"),
                revision: r.get("revision"),
                title: r.get("title"),
                file_path: r.get("file_path"),
                status: r.get("status"),
                confidentiality: r.get("confidentiality"),
                frozen_dept_code: r.get("frozen_dept_code"),
                author: AuthorBrief {
                    id: r.get("author_id"),
                    name: r.get("author_name"),
                },
                doc_kind: DocKindBrief {
                    id: r.get("doc_kind_id"),
                    code: r.get("doc_kind_code"),
                    name: r.get("doc_kind_name"),
                },
                project: ProjectBrief {
                    id: r.get("project_id"),
                    name: r.get("project_name"),
                },
                tags,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        }
        None => Ok(None),
    }
}
