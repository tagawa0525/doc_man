use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::circulation::{CirculationResponse, CreateCirculationRequest, RecipientBrief};
use crate::state::AppState;

/// GET /api/v1/documents/{doc_id}/circulations
pub async fn list_circulations(
    _user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<CirculationResponse>>, AppError> {
    use sqlx::Row;

    let rows = sqlx::query(
        "SELECT c.id, c.confirmed_at,
                e.id AS recipient_id, e.name AS recipient_name
         FROM circulations c
         JOIN employees e ON e.id = c.recipient_id
         WHERE c.document_id = $1
         ORDER BY e.name",
    )
    .bind(doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<CirculationResponse> = rows
        .into_iter()
        .map(|r| CirculationResponse {
            id: r.get("id"),
            recipient: RecipientBrief {
                id: r.get("recipient_id"),
                name: r.get("recipient_name"),
            },
            confirmed_at: r.get("confirmed_at"),
        })
        .collect();

    Ok(Json(data))
}

/// POST /api/v1/documents/{doc_id}/circulations
pub async fn create_circulations(
    user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<CreateCirculationRequest>,
) -> Result<(StatusCode, Json<Vec<CirculationResponse>>), AppError> {
    if user.role != Role::Admin && user.role != Role::ProjectManager {
        return Err(AppError::Forbidden(
            "admin or project_manager role required".to_string(),
        ));
    }

    if req.recipient_ids.is_empty() {
        return Err(AppError::InvalidRequest(
            "recipient_ids must not be empty".to_string(),
        ));
    }

    use sqlx::Row;

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 文書のステータスを確認
    let doc = sqlx::query("SELECT status FROM documents WHERE id = $1 FOR UPDATE")
        .bind(doc_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("document {} not found", doc_id)))?;

    let doc_status: String = doc.get("status");
    if doc_status != "approved" {
        return Err(AppError::Unprocessable(format!(
            "circulation can only be started on approved documents, current status: {}",
            doc_status
        )));
    }

    // 重複排除
    let unique_ids: Vec<Uuid> = {
        let mut seen = std::collections::HashSet::new();
        req.recipient_ids
            .iter()
            .filter(|id| seen.insert(**id))
            .copied()
            .collect()
    };

    for recipient_id in &unique_ids {
        sqlx::query("INSERT INTO circulations (document_id, recipient_id) VALUES ($1, $2)")
            .bind(doc_id)
            .bind(recipient_id)
            .execute(tx.as_mut())
            .await
            .map_err(|e| match &e {
                sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                    Some("23503") => AppError::InvalidRequest(
                        "referenced recipient_id does not exist".to_string(),
                    ),
                    Some("23505") => {
                        AppError::Conflict("duplicate recipient in circulation".to_string())
                    }
                    _ => AppError::Database(e),
                },
                _ => AppError::Database(e),
            })?;
    }

    // 文書ステータスを circulating に変更
    sqlx::query(
        "UPDATE documents SET status = 'circulating', updated_at = now()
         WHERE id = $1 AND status = 'approved'",
    )
    .bind(doc_id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    // レスポンス用にデータを取得
    let rows = sqlx::query(
        "SELECT c.id, c.confirmed_at,
                e.id AS recipient_id, e.name AS recipient_name
         FROM circulations c
         JOIN employees e ON e.id = c.recipient_id
         WHERE c.document_id = $1
         ORDER BY e.name",
    )
    .bind(doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<CirculationResponse> = rows
        .into_iter()
        .map(|r| CirculationResponse {
            id: r.get("id"),
            recipient: RecipientBrief {
                id: r.get("recipient_id"),
                name: r.get("recipient_name"),
            },
            confirmed_at: r.get("confirmed_at"),
        })
        .collect();

    Ok((StatusCode::CREATED, Json(data)))
}

/// POST /api/v1/documents/{doc_id}/circulations/confirm
pub async fn confirm_circulation(
    user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<CirculationResponse>, AppError> {
    use sqlx::Row;

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 文書の存在とステータスを確認（行ロックで並行確認を直列化）
    let doc = sqlx::query("SELECT status FROM documents WHERE id = $1 FOR UPDATE")
        .bind(doc_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("document {} not found", doc_id)))?;

    let doc_status: String = doc.get("status");
    if doc_status != "circulating" {
        return Err(AppError::Unprocessable(format!(
            "document is not in circulating status, current status: {}",
            doc_status
        )));
    }

    // ユーザーの回覧レコードを取得
    let circ = sqlx::query(
        "SELECT id, confirmed_at FROM circulations
         WHERE document_id = $1 AND recipient_id = $2
         FOR UPDATE",
    )
    .bind(doc_id)
    .bind(user.id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| {
        AppError::Forbidden("you are not a recipient of this circulation".to_string())
    })?;

    let circ_id: Uuid = circ.get("id");

    // confirmed_at が未設定の場合のみ更新（冪等性を保証）
    sqlx::query(
        "UPDATE circulations SET confirmed_at = now() WHERE id = $1 AND confirmed_at IS NULL",
    )
    .bind(circ_id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // 全員確認済みなら文書を completed に（NOT EXISTS で原子的に判定）
    sqlx::query(
        "UPDATE documents SET status = 'completed', updated_at = now()
         WHERE id = $1
           AND status = 'circulating'
           AND NOT EXISTS (
               SELECT 1 FROM circulations c
               WHERE c.document_id = $1
                 AND c.confirmed_at IS NULL
           )",
    )
    .bind(doc_id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    // レスポンス用にデータを取得
    let row = sqlx::query(
        "SELECT c.id, c.confirmed_at,
                e.id AS recipient_id, e.name AS recipient_name
         FROM circulations c
         JOIN employees e ON e.id = c.recipient_id
         WHERE c.id = $1",
    )
    .bind(circ_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(CirculationResponse {
        id: row.get("id"),
        recipient: RecipientBrief {
            id: row.get("recipient_id"),
            name: row.get("recipient_name"),
        },
        confirmed_at: row.get("confirmed_at"),
    }))
}
