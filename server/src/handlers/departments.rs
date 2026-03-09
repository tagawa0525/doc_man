use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::department::{
    CreateDepartmentRequest, DepartmentResponse, DepartmentRow, DepartmentTree,
    UpdateDepartmentRequest,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DepartmentListQuery {
    #[serde(default)]
    pub include_inactive: bool,
}

/// GET /api/v1/departments
pub async fn list_departments(
    _user: AuthenticatedUser,
    Query(params): Query<DepartmentListQuery>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DepartmentTree>>, AppError> {
    let rows = fetch_department_rows(&state, params.include_inactive).await?;
    let tree = build_tree(rows);
    Ok(Json(tree))
}

/// POST /api/v1/departments
pub async fn create_department(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<(StatusCode, Json<DepartmentResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let row = sqlx::query(
        "INSERT INTO departments (code, name, parent_id, effective_from)
         VALUES ($1, $2, $3, $4)
         RETURNING id, code, name, parent_id, effective_from, effective_to, merged_into_id",
    )
    .bind(&req.code)
    .bind(&req.name)
    .bind(req.parent_id)
    .bind(req.effective_from)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => match db_err.constraint() {
            Some("departments_code_unique") => {
                AppError::Conflict(format!("department code '{}' already exists", req.code))
            }
            _ => AppError::Database(e),
        },
        _ => AppError::Database(e),
    })?;

    let dept = DepartmentRow {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        parent_id: row.get("parent_id"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        merged_into_id: row.get("merged_into_id"),
    };

    Ok((StatusCode::CREATED, Json(DepartmentResponse::from(dept))))
}

/// GET /api/v1/departments/{id}
pub async fn get_department(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<DepartmentResponse>, AppError> {
    let row = sqlx::query(
        "SELECT id, code, name, parent_id, effective_from, effective_to, merged_into_id
         FROM departments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("department {id} not found")))?;

    let dept = DepartmentRow {
        id: row.get("id"),
        code: row.get("code"),
        name: row.get("name"),
        parent_id: row.get("parent_id"),
        effective_from: row.get("effective_from"),
        effective_to: row.get("effective_to"),
        merged_into_id: row.get("merged_into_id"),
    };

    Ok(Json(DepartmentResponse::from(dept)))
}

/// PUT /api/v1/departments/{id}
pub async fn update_department(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<DepartmentResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    // 存在チェック
    let existing = sqlx::query(
        "SELECT id, code, name, parent_id, effective_from, effective_to, merged_into_id
         FROM departments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("department {id} not found")))?;

    let current_name: String = existing.get("name");
    let current_effective_to: Option<NaiveDate> = existing.get("effective_to");
    let current_merged_into_id: Option<Uuid> = existing.get("merged_into_id");

    let new_name = req.name.unwrap_or(current_name);
    let new_effective_to = req.effective_to.or(current_effective_to);
    let new_merged_into_id = req.merged_into_id.or(current_merged_into_id);

    let updated = sqlx::query(
        "UPDATE departments
         SET name = $1, effective_to = $2, merged_into_id = $3, updated_at = now()
         WHERE id = $4
         RETURNING id, code, name, parent_id, effective_from, effective_to, merged_into_id",
    )
    .bind(&new_name)
    .bind(new_effective_to)
    .bind(new_merged_into_id)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23503") => {
            AppError::InvalidRequest(
                "merged_into_id references a non-existent department".to_string(),
            )
        }
        _ => AppError::Database(e),
    })?;

    let dept = DepartmentRow {
        id: updated.get("id"),
        code: updated.get("code"),
        name: updated.get("name"),
        parent_id: updated.get("parent_id"),
        effective_from: updated.get("effective_from"),
        effective_to: updated.get("effective_to"),
        merged_into_id: updated.get("merged_into_id"),
    };

    Ok(Json(DepartmentResponse::from(dept)))
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_department_rows(
    state: &AppState,
    include_inactive: bool,
) -> Result<Vec<DepartmentRow>, AppError> {
    let query = if include_inactive {
        "SELECT id, code, name, parent_id, effective_from, effective_to, merged_into_id
         FROM departments ORDER BY code"
    } else {
        "SELECT id, code, name, parent_id, effective_from, effective_to, merged_into_id
         FROM departments WHERE effective_to IS NULL ORDER BY code"
    };

    let rows = sqlx::query(query)
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| DepartmentRow {
            id: r.get("id"),
            code: r.get("code"),
            name: r.get("name"),
            parent_id: r.get("parent_id"),
            effective_from: r.get("effective_from"),
            effective_to: r.get("effective_to"),
            merged_into_id: r.get("merged_into_id"),
        })
        .collect())
}

/// フラットなリストをツリーに組み立てる
fn build_tree(rows: Vec<DepartmentRow>) -> Vec<DepartmentTree> {
    use std::collections::HashMap;

    // 再帰的にサブツリーを構築する内部関数
    fn build_subtree(
        id: Uuid,
        nodes: &mut HashMap<Uuid, DepartmentTree>,
        children_map: &HashMap<Option<Uuid>, Vec<Uuid>>,
    ) -> DepartmentTree {
        let mut node = nodes.remove(&id).expect("node must exist");
        if let Some(child_ids) = children_map.get(&Some(id)) {
            node.children = child_ids
                .iter()
                .map(|&child_id| build_subtree(child_id, nodes, children_map))
                .collect();
        }
        node
    }

    let mut nodes: HashMap<Uuid, DepartmentTree> = rows
        .iter()
        .map(|r| {
            (
                r.id,
                DepartmentTree {
                    id: r.id,
                    code: r.code.clone(),
                    name: r.name.clone(),
                    parent_id: r.parent_id,
                    effective_from: r.effective_from,
                    effective_to: r.effective_to,
                    children: vec![],
                },
            )
        })
        .collect();

    // 親ID -> 子ID一覧 のマップを構築（None はルート）
    let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
    for r in &rows {
        children_map.entry(r.parent_id).or_default().push(r.id);
    }

    let roots: Vec<Uuid> = children_map.get(&None).cloned().unwrap_or_default();

    roots
        .into_iter()
        .map(|id| build_subtree(id, &mut nodes, &children_map))
        .collect()
}
