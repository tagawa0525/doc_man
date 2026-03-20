use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::authorization;
use crate::error::AppError;
use crate::models::approval_step::{
    ApprovalActionRequest, ApprovalStepResponse, ApproverBrief, CreateApprovalRouteRequest,
};
use crate::state::AppState;

/// GET /api/v1/documents/{doc_id}/approval-steps
pub async fn list_approval_steps(
    _user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ApprovalStepResponse>>, AppError> {
    let rows = sqlx::query(
        "SELECT a.id, a.route_revision, a.document_revision, a.step_order,
                a.status, a.approved_at, a.comment, a.created_at,
                e.id AS approver_id, e.name AS approver_name
         FROM approval_steps a
         JOIN employees e ON e.id = a.approver_id
         WHERE a.document_id = $1
         ORDER BY a.route_revision, a.step_order",
    )
    .bind(doc_id)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<ApprovalStepResponse> = rows
        .into_iter()
        .map(|r| ApprovalStepResponse {
            id: r.get("id"),
            route_revision: r.get("route_revision"),
            document_revision: r.get("document_revision"),
            step_order: r.get("step_order"),
            approver: ApproverBrief {
                id: r.get("approver_id"),
                name: r.get("approver_name"),
            },
            status: r.get("status"),
            approved_at: r.get("approved_at"),
            comment: r.get("comment"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(data))
}

/// POST /api/v1/documents/{doc_id}/approval-steps
pub async fn create_approval_route(
    user: AuthenticatedUser,
    Path(doc_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<CreateApprovalRouteRequest>,
) -> Result<(StatusCode, Json<Vec<ApprovalStepResponse>>), AppError> {
    if user.role != Role::Admin && user.role != Role::ProjectManager {
        return Err(AppError::Forbidden(
            "admin or project_manager role required".to_string(),
        ));
    }

    let dept_id = authorization::get_document_department_id(&state.db, doc_id).await?;
    authorization::check_department_access(&user, dept_id)?;

    if req.steps.is_empty() {
        return Err(AppError::InvalidRequest(
            "steps must not be empty".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 文書のステータスと revision を取得
    let doc = sqlx::query("SELECT status, revision FROM documents WHERE id = $1 FOR UPDATE")
        .bind(doc_id)
        .fetch_optional(tx.as_mut())
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound(format!("document {doc_id} not found")))?;

    let doc_status: String = doc.get("status");
    let doc_revision: i32 = doc.get("revision");

    if doc_status != "draft" && doc_status != "rejected" {
        return Err(AppError::Unprocessable(format!(
            "approval route can only be set on draft or rejected documents, current status: {doc_status}"
        )));
    }

    // 現在の最大 route_revision を取得
    let max_route_rev: Option<i32> =
        sqlx::query_scalar("SELECT MAX(route_revision) FROM approval_steps WHERE document_id = $1")
            .bind(doc_id)
            .fetch_one(tx.as_mut())
            .await
            .map_err(AppError::Database)?;

    let new_route_revision = max_route_rev.unwrap_or(0) + 1;

    // 承認ステップを挿入
    for step in &req.steps {
        sqlx::query(
            "INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(doc_id)
        .bind(new_route_revision)
        .bind(doc_revision)
        .bind(step.step_order)
        .bind(step.approver_id)
        .execute(tx.as_mut())
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) => match db_err.code().as_deref() {
                Some("23503") => {
                    AppError::InvalidRequest("referenced approver_id does not exist".to_string())
                }
                Some("23505") => AppError::Conflict(
                    "duplicate step_order in approval route".to_string(),
                ),
                _ => AppError::Database(e),
            },
            _ => AppError::Database(e),
        })?;
    }

    // 文書のステータスを under_review に変更
    sqlx::query("UPDATE documents SET status = 'under_review', updated_at = now() WHERE id = $1")
        .bind(doc_id)
        .execute(tx.as_mut())
        .await
        .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    // レスポンス用にデータを取得
    let rows = sqlx::query(
        "SELECT a.id, a.route_revision, a.document_revision, a.step_order,
                a.status, a.approved_at, a.comment, a.created_at,
                e.id AS approver_id, e.name AS approver_name
         FROM approval_steps a
         JOIN employees e ON e.id = a.approver_id
         WHERE a.document_id = $1 AND a.route_revision = $2
         ORDER BY a.step_order",
    )
    .bind(doc_id)
    .bind(new_route_revision)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let data: Vec<ApprovalStepResponse> = rows
        .into_iter()
        .map(|r| ApprovalStepResponse {
            id: r.get("id"),
            route_revision: r.get("route_revision"),
            document_revision: r.get("document_revision"),
            step_order: r.get("step_order"),
            approver: ApproverBrief {
                id: r.get("approver_id"),
                name: r.get("approver_name"),
            },
            status: r.get("status"),
            approved_at: r.get("approved_at"),
            comment: r.get("comment"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok((StatusCode::CREATED, Json(data)))
}

/// POST /api/v1/documents/{doc_id}/approval-steps/{step_id}/approve
pub async fn approve_step(
    user: AuthenticatedUser,
    Path((doc_id, step_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(req): Json<ApprovalActionRequest>,
) -> Result<Json<ApprovalStepResponse>, AppError> {
    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 対象ステップを取得
    let step = sqlx::query(
        "SELECT a.id, a.document_id, a.route_revision, a.document_revision,
                a.step_order, a.approver_id, a.status
         FROM approval_steps a
         WHERE a.id = $1 AND a.document_id = $2
         FOR UPDATE",
    )
    .bind(step_id)
    .bind(doc_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("approval step {step_id} not found")))?;

    let approver_id: Uuid = step.get("approver_id");
    let route_revision: i32 = step.get("route_revision");

    // 承認者チェック
    if user.id != approver_id {
        return Err(AppError::Forbidden(
            "only the assigned approver can approve this step".to_string(),
        ));
    }

    // アクティブステップチェック（最新route_revisionの最小step_order pending）
    let active_step_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM approval_steps
         WHERE document_id = $1
           AND route_revision = (SELECT MAX(route_revision) FROM approval_steps WHERE document_id = $1)
           AND status = 'pending'
         ORDER BY step_order
         LIMIT 1",
    )
    .bind(doc_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    if active_step_id != Some(step_id) {
        return Err(AppError::Unprocessable(
            "this step is not the active step".to_string(),
        ));
    }

    // ステップを approved に更新
    sqlx::query(
        "UPDATE approval_steps SET status = 'approved', approved_at = now(), comment = $1 WHERE id = $2",
    )
    .bind(&req.comment)
    .bind(step_id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // 残りの pending ステップがあるか確認
    let remaining_pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM approval_steps
         WHERE document_id = $1 AND route_revision = $2 AND status = 'pending'",
    )
    .bind(doc_id)
    .bind(route_revision)
    .fetch_one(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // 最後のステップなら文書を approved に変更（under_review の場合のみ）
    if remaining_pending == 0 {
        sqlx::query(
            "UPDATE documents SET status = 'approved', updated_at = now()
             WHERE id = $1 AND status = 'under_review'",
        )
        .bind(doc_id)
        .execute(tx.as_mut())
        .await
        .map_err(AppError::Database)?;
    }

    tx.commit().await.map_err(AppError::Database)?;

    // レスポンス用にデータを取得
    let row = sqlx::query(
        "SELECT a.id, a.route_revision, a.document_revision, a.step_order,
                a.status, a.approved_at, a.comment, a.created_at,
                e.id AS approver_id, e.name AS approver_name
         FROM approval_steps a
         JOIN employees e ON e.id = a.approver_id
         WHERE a.id = $1",
    )
    .bind(step_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ApprovalStepResponse {
        id: row.get("id"),
        route_revision: row.get("route_revision"),
        document_revision: row.get("document_revision"),
        step_order: row.get("step_order"),
        approver: ApproverBrief {
            id: row.get("approver_id"),
            name: row.get("approver_name"),
        },
        status: row.get("status"),
        approved_at: row.get("approved_at"),
        comment: row.get("comment"),
        created_at: row.get("created_at"),
    }))
}

/// POST /api/v1/documents/{doc_id}/approval-steps/{step_id}/reject
pub async fn reject_step(
    user: AuthenticatedUser,
    Path((doc_id, step_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(req): Json<ApprovalActionRequest>,
) -> Result<Json<ApprovalStepResponse>, AppError> {
    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    // 対象ステップを取得
    let step = sqlx::query(
        "SELECT a.id, a.document_id, a.route_revision, a.document_revision,
                a.step_order, a.approver_id, a.status
         FROM approval_steps a
         WHERE a.id = $1 AND a.document_id = $2
         FOR UPDATE",
    )
    .bind(step_id)
    .bind(doc_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("approval step {step_id} not found")))?;

    let approver_id: Uuid = step.get("approver_id");
    let route_revision: i32 = step.get("route_revision");

    // 承認者チェック
    if user.id != approver_id {
        return Err(AppError::Forbidden(
            "only the assigned approver can reject this step".to_string(),
        ));
    }

    // アクティブステップチェック
    let active_step_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM approval_steps
         WHERE document_id = $1
           AND route_revision = (SELECT MAX(route_revision) FROM approval_steps WHERE document_id = $1)
           AND status = 'pending'
         ORDER BY step_order
         LIMIT 1",
    )
    .bind(doc_id)
    .fetch_optional(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    if active_step_id != Some(step_id) {
        return Err(AppError::Unprocessable(
            "this step is not the active step".to_string(),
        ));
    }

    // 対象ステップを rejected に更新
    sqlx::query("UPDATE approval_steps SET status = 'rejected', comment = $1 WHERE id = $2")
        .bind(&req.comment)
        .bind(step_id)
        .execute(tx.as_mut())
        .await
        .map_err(AppError::Database)?;

    // 同ルートの残り pending ステップも全て rejected に
    sqlx::query(
        "UPDATE approval_steps SET status = 'rejected'
         WHERE document_id = $1 AND route_revision = $2 AND status = 'pending'",
    )
    .bind(doc_id)
    .bind(route_revision)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    // 文書を rejected に変更（under_review の場合のみ）
    sqlx::query(
        "UPDATE documents SET status = 'rejected', updated_at = now()
         WHERE id = $1 AND status = 'under_review'",
    )
    .bind(doc_id)
    .execute(tx.as_mut())
    .await
    .map_err(AppError::Database)?;

    tx.commit().await.map_err(AppError::Database)?;

    // レスポンス用にデータを取得
    let row = sqlx::query(
        "SELECT a.id, a.route_revision, a.document_revision, a.step_order,
                a.status, a.approved_at, a.comment, a.created_at,
                e.id AS approver_id, e.name AS approver_name
         FROM approval_steps a
         JOIN employees e ON e.id = a.approver_id
         WHERE a.id = $1",
    )
    .bind(step_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ApprovalStepResponse {
        id: row.get("id"),
        route_revision: row.get("route_revision"),
        document_revision: row.get("document_revision"),
        step_order: row.get("step_order"),
        approver: ApproverBrief {
            id: row.get("approver_id"),
            name: row.get("approver_name"),
        },
        status: row.get("status"),
        approved_at: row.get("approved_at"),
        comment: row.get("comment"),
        created_at: row.get("created_at"),
    }))
}
