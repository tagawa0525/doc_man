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
use crate::models::document_revision::{
    CreatedByBrief, DocumentRevisionResponse, ReviseDocumentRequest,
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
        "SELECT d.id, d.doc_number, d.revision, d.title,
                COALESCE(dr.file_path, '') AS file_path,
                d.status, d.confidentiality, d.frozen_dept_code,
                d.created_at, d.updated_at,
                e.id AS author_id, e.name AS author_name,
                dk.id AS doc_kind_id, dk.code AS doc_kind_code, dk.name AS doc_kind_name,
                p.id AS project_id, p.name AS project_name
         FROM documents d
         JOIN employees e ON e.id = d.author_id
         JOIN document_kinds dk ON dk.id = d.doc_kind_id
         JOIN projects p ON p.id = d.project_id
         LEFT JOIN document_revisions dr ON dr.document_id = d.id AND dr.effective_to IS NULL
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
        "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, confidentiality, project_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(&doc_number)
    .bind(&req.title)
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

    // document_revisions に Rev.0 を作成
    let file_path = format!("{doc_number}/0");
    sqlx::query(
        "INSERT INTO document_revisions (document_id, revision, file_path, created_by)
         VALUES ($1, 0, $2, $3)",
    )
    .bind(doc_id)
    .bind(&file_path)
    .bind(user.id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

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
        "SELECT title, confidentiality
         FROM documents WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;

    let current_title: String = existing.get("title");
    let current_confidentiality: String = existing.get("confidentiality");

    let new_title = req.title.unwrap_or(current_title);
    let new_confidentiality = req.confidentiality.unwrap_or(current_confidentiality);

    sqlx::query(
        "UPDATE documents
         SET title = $1, confidentiality = $2, updated_at = now()
         WHERE id = $3",
    )
    .bind(&new_title)
    .bind(&new_confidentiality)
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

    // distributions の存在チェック
    let dist_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM distributions WHERE document_id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .map_err(AppError::Database)?;

    if dist_count > 0 {
        return Err(AppError::Conflict(
            "cannot delete document with distributions".to_string(),
        ));
    }

    // document_revisions を先に削除（FK制約のため）
    sqlx::query("DELETE FROM document_revisions WHERE document_id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

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

/// POST /api/v1/documents/{id}/revise
pub async fn revise_document(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<ReviseDocumentRequest>,
) -> Result<Json<DocumentResponse>, AppError> {
    if user.role == Role::Viewer {
        return Err(AppError::Forbidden(
            "viewer role cannot revise documents".to_string(),
        ));
    }

    if req.reason.trim().is_empty() {
        return Err(AppError::InvalidRequest("reason is required".to_string()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    let doc =
        sqlx::query("SELECT status, revision, doc_number FROM documents WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(tx.as_mut())
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("document {id} not found")))?;

    let status: String = doc.get("status");
    if status != "approved" {
        return Err(AppError::Unprocessable(
            "only approved documents can be revised".to_string(),
        ));
    }

    let current_revision: i32 = doc.get("revision");
    let doc_number: String = doc.get("doc_number");
    let new_revision = current_revision + 1;

    // 旧改訂の effective_to を閉じる
    sqlx::query(
        "UPDATE document_revisions SET effective_to = now()
         WHERE document_id = $1 AND effective_to IS NULL",
    )
    .bind(id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // 新改訂レコードを作成
    let file_path = format!("{doc_number}/{new_revision}");
    sqlx::query(
        "INSERT INTO document_revisions (document_id, revision, file_path, reason, created_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(new_revision)
    .bind(&file_path)
    .bind(&req.reason)
    .bind(user.id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // documents の revision と status を更新
    sqlx::query(
        "UPDATE documents SET revision = $1, status = 'draft', updated_at = now() WHERE id = $2",
    )
    .bind(new_revision)
    .bind(id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    let doc = fetch_document_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch revised document".to_string()))?;

    Ok(Json(doc))
}

/// GET /api/v1/documents/{id}/revisions
pub async fn list_document_revisions(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DocumentRevisionResponse>>, AppError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if !exists {
        return Err(AppError::NotFound(format!("document {id} not found")));
    }

    let rows = sqlx::query(
        "SELECT dr.id, dr.document_id, dr.revision, dr.file_path, dr.reason,
                dr.effective_from, dr.effective_to,
                e.id AS created_by_id, e.name AS created_by_name
         FROM document_revisions dr
         JOIN employees e ON e.id = dr.created_by
         WHERE dr.document_id = $1
         ORDER BY dr.revision DESC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<DocumentRevisionResponse> = rows
        .into_iter()
        .map(|r| DocumentRevisionResponse {
            id: r.get("id"),
            document_id: r.get("document_id"),
            revision: r.get("revision"),
            file_path: r.get("file_path"),
            reason: r.get("reason"),
            created_by: CreatedByBrief {
                id: r.get("created_by_id"),
                name: r.get("created_by_name"),
            },
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
        })
        .collect();

    Ok(Json(data))
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
        "SELECT d.id, d.doc_number, d.revision, d.title,
                COALESCE(dr.file_path, '') AS file_path,
                d.status, d.confidentiality, d.frozen_dept_code,
                d.created_at, d.updated_at,
                e.id AS author_id, e.name AS author_name,
                dk.id AS doc_kind_id, dk.code AS doc_kind_code, dk.name AS doc_kind_name,
                p.id AS project_id, p.name AS project_name
         FROM documents d
         JOIN employees e ON e.id = d.author_id
         JOIN document_kinds dk ON dk.id = d.doc_kind_id
         JOIN projects p ON p.id = d.project_id
         LEFT JOIN document_revisions dr ON dr.document_id = d.id AND dr.effective_to IS NULL
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
