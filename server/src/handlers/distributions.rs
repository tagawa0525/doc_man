use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::authorization;
use crate::error::AppError;
use crate::models::distribution::{
    CreateDistributionRequest, DistributedByBrief, DistributionResponse, RecipientBrief,
};
use crate::services::mail::{DistributionMailContext, MailRecipient};
use crate::state::AppState;

/// GET /api/v1/documents/{doc_id}/distributions
pub async fn list_distributions(
    _user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DistributionResponse>>, AppError> {
    let rows = sqlx::query(
        "SELECT d.id, d.distributed_at,
                r.id AS recipient_id, r.name AS recipient_name, r.email AS recipient_email,
                db.id AS distributed_by_id, db.name AS distributed_by_name
         FROM distributions d
         JOIN employees r ON r.id = d.recipient_id
         JOIN employees db ON db.id = d.distributed_by
         WHERE d.document_id = $1
         ORDER BY d.distributed_at DESC, r.name",
    )
    .bind(doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data = rows
        .into_iter()
        .map(|r| DistributionResponse {
            id: r.get("id"),
            recipient: RecipientBrief {
                id: r.get("recipient_id"),
                name: r.get("recipient_name"),
                email: r.get("recipient_email"),
            },
            distributed_by: DistributedByBrief {
                id: r.get("distributed_by_id"),
                name: r.get("distributed_by_name"),
            },
            distributed_at: r.get("distributed_at"),
        })
        .collect();

    Ok(Json(data))
}

/// POST /api/v1/documents/{doc_id}/distributions
pub async fn create_distributions(
    user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<CreateDistributionRequest>,
) -> Result<(StatusCode, Json<Vec<DistributionResponse>>), AppError> {
    if !matches!(user.role, Role::Admin | Role::ProjectManager) {
        return Err(AppError::Forbidden(
            "admin or project_manager role required".to_string(),
        ));
    }

    let dept_id = authorization::get_document_department_id(&state.db, doc_id).await?;
    authorization::check_department_access(&user, dept_id)?;

    // 重複排除
    let unique_ids: Vec<Uuid> = {
        let mut seen = std::collections::HashSet::new();
        req.recipient_ids
            .into_iter()
            .filter(|id| seen.insert(*id))
            .collect()
    };

    if unique_ids.is_empty() {
        return Err(AppError::InvalidRequest(
            "recipient_ids must not be empty".to_string(),
        ));
    }

    // 文書の存在確認
    let doc_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM documents WHERE id = $1)")
            .bind(doc_id)
            .fetch_one(&state.db)
            .await
            .map_err(AppError::Database)?;

    if !doc_exists {
        return Err(AppError::NotFound(format!("document {doc_id} not found")));
    }

    // 同一タイムスタンプでバッチ挿入（全成功 or 全ロールバック）
    let now = chrono::Utc::now();
    let mut inserted_ids: Vec<Uuid> = Vec::with_capacity(unique_ids.len());

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;
    for recipient_id in &unique_ids {
        let row = sqlx::query(
            "INSERT INTO distributions (document_id, recipient_id, distributed_at, distributed_by)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
        )
        .bind(doc_id)
        .bind(recipient_id)
        .bind(now)
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23503")
            {
                return AppError::InvalidRequest(format!("invalid recipient_id: {recipient_id}"));
            }
            AppError::Database(e)
        })?;

        inserted_ids.push(row.get("id"));
    }
    tx.commit().await.map_err(AppError::Database)?;

    // メール送信（stub ではログ出力のみ）
    let doc_info = sqlx::query(
        "SELECT d.doc_number, d.title, dr.file_path, e.name AS author_name
         FROM documents d
         JOIN employees e ON e.id = $2
         JOIN document_revisions dr ON dr.document_id = d.id AND dr.effective_to IS NULL
         WHERE d.id = $1",
    )
    .bind(doc_id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let recipients_for_mail: Vec<MailRecipient> =
        sqlx::query("SELECT name, email FROM employees WHERE id = ANY($1) AND email IS NOT NULL")
            .bind(&unique_ids)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|r| MailRecipient {
                name: r.get("name"),
                email: r.get("email"),
            })
            .collect();

    if !recipients_for_mail.is_empty() {
        let file_path: String = doc_info.get("file_path");
        let directory_path = std::path::Path::new(&file_path)
            .parent()
            .map_or_else(|| file_path.clone(), |p| p.to_string_lossy().to_string());

        let context = DistributionMailContext {
            doc_number: doc_info.get("doc_number"),
            title: doc_info.get("title"),
            directory_path,
            distributed_by_name: doc_info.get("author_name"),
        };

        if let Err(e) = state
            .mail_sender
            .send_distribution(&recipients_for_mail, &context)
            .await
        {
            tracing::warn!("failed to send distribution mail: {e}");
        }
    }

    // 挿入したレコードを取得して返す
    let rows = sqlx::query(
        "SELECT d.id, d.distributed_at,
                r.id AS recipient_id, r.name AS recipient_name, r.email AS recipient_email,
                db.id AS distributed_by_id, db.name AS distributed_by_name
         FROM distributions d
         JOIN employees r ON r.id = d.recipient_id
         JOIN employees db ON db.id = d.distributed_by
         WHERE d.id = ANY($1)
         ORDER BY r.name",
    )
    .bind(&inserted_ids)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data = rows
        .into_iter()
        .map(|r| DistributionResponse {
            id: r.get("id"),
            recipient: RecipientBrief {
                id: r.get("recipient_id"),
                name: r.get("recipient_name"),
                email: r.get("recipient_email"),
            },
            distributed_by: DistributedByBrief {
                id: r.get("distributed_by_id"),
                name: r.get("distributed_by_name"),
            },
            distributed_at: r.get("distributed_at"),
        })
        .collect();

    Ok((StatusCode::CREATED, Json(data)))
}
