use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::project::{
    CreateProjectRequest, ProjectResponse, ProjectRow, UpdateProjectRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub status: Option<String>,
    pub discipline_id: Option<Uuid>,
    pub wbs_code: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// GET /api/v1/projects
pub async fn list_projects(
    _user: AuthenticatedUser,
    Query(params): Query<ProjectListQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<ProjectResponse>>, AppError> {
    if let Err(e) = params.pagination.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects
         WHERE ($1::text IS NULL OR status = $1)
           AND ($2::uuid IS NULL OR discipline_id = $2)
           AND ($3::text IS NULL OR wbs_code = $3)",
    )
    .bind(&params.status)
    .bind(params.discipline_id)
    .bind(&params.wbs_code)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    let rows = sqlx::query(
        "SELECT p.id, p.name, p.status, p.start_date, p.end_date, p.wbs_code,
                di.id as disc_id, di.code as disc_code, di.name as disc_name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name,
                e.id as manager_id, e.name as manager_name
         FROM projects p
         JOIN disciplines di ON di.id = p.discipline_id
         JOIN departments d ON d.id = di.department_id
         LEFT JOIN employees e ON e.id = p.manager_id
         WHERE ($1::text IS NULL OR p.status = $1)
           AND ($2::uuid IS NULL OR p.discipline_id = $2)
           AND ($3::text IS NULL OR p.wbs_code = $3)
         ORDER BY p.name, p.id
         LIMIT $4 OFFSET $5",
    )
    .bind(&params.status)
    .bind(params.discipline_id)
    .bind(&params.wbs_code)
    .bind(params.pagination.limit())
    .bind(params.pagination.offset())
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    use sqlx::Row;
    let data: Vec<ProjectResponse> = rows
        .into_iter()
        .map(|r| {
            ProjectRow {
                id: r.get("id"),
                name: r.get("name"),
                status: r.get("status"),
                start_date: r.get("start_date"),
                end_date: r.get("end_date"),
                wbs_code: r.get("wbs_code"),
                disc_id: r.get("disc_id"),
                disc_code: r.get("disc_code"),
                disc_name: r.get("disc_name"),
                dept_id: r.get("dept_id"),
                dept_code: r.get("dept_code"),
                dept_name: r.get("dept_name"),
                manager_id: r.get("manager_id"),
                manager_name: r.get("manager_name"),
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

/// POST /api/v1/projects
pub async fn create_project(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    if user.role != Role::Admin && user.role != Role::ProjectManager {
        return Err(AppError::Forbidden(
            "admin or project_manager role required".to_string(),
        ));
    }

    let status = req.status.as_deref().unwrap_or("planning");

    // project_manager が manager_id を省略した場合、自身を設定
    let manager_id = req.manager_id.or(if user.role == Role::ProjectManager {
        Some(user.id)
    } else {
        None
    });

    let row = sqlx::query(
        "INSERT INTO projects (name, status, start_date, end_date, wbs_code, discipline_id, manager_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id",
    )
    .bind(&req.name)
    .bind(status)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(&req.wbs_code)
    .bind(req.discipline_id)
    .bind(manager_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23514") {
                return AppError::InvalidRequest(
                    "invalid project data (check constraint violated)".to_string(),
                );
            }
            if db_err.code().as_deref() == Some("23503") {
                return AppError::InvalidRequest(
                    "referenced discipline_id or manager_id does not exist".to_string(),
                );
            }
            match db_err.constraint() {
                Some("projects_wbs_code_key") => {
                    AppError::Conflict(format!(
                        "wbs_code '{}' already exists",
                        req.wbs_code.as_deref().unwrap_or("")
                    ))
                }
                _ => AppError::Database(e),
            }
        }
        _ => AppError::Database(e),
    })?;

    use sqlx::Row;
    let id: Uuid = row.get("id");
    let proj = fetch_project_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch created project".to_string()))?;

    Ok((StatusCode::CREATED, Json(proj)))
}

/// GET /api/v1/projects/{id}
pub async fn get_project(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<ProjectResponse>, AppError> {
    let proj = fetch_project_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("project {} not found", id)))?;

    Ok(Json(proj))
}

/// PUT /api/v1/projects/{id}
pub async fn update_project(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateProjectRequest>,
) -> Result<Json<ProjectResponse>, AppError> {
    let existing = sqlx::query(
        "SELECT name, status, start_date, end_date, wbs_code, discipline_id, manager_id
         FROM projects WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound(format!("project {} not found", id)))?;

    use sqlx::Row;
    let current_manager_id: Option<Uuid> = existing.get("manager_id");

    // 権限チェック: adminは無条件、project_managerは担当プロジェクトのみ
    match user.role {
        Role::Admin => {}
        Role::ProjectManager => {
            if current_manager_id != Some(user.id) {
                return Err(AppError::Forbidden(
                    "project_manager can only update own projects".to_string(),
                ));
            }
        }
        _ => {
            return Err(AppError::Forbidden(
                "admin or project_manager role required".to_string(),
            ));
        }
    }

    let current_name: String = existing.get("name");
    let current_status: String = existing.get("status");
    let current_start: Option<chrono::NaiveDate> = existing.get("start_date");
    let current_end: Option<chrono::NaiveDate> = existing.get("end_date");
    let current_wbs: Option<String> = existing.get("wbs_code");
    let current_disc_id: Uuid = existing.get("discipline_id");

    let new_name = req.name.unwrap_or(current_name);
    let new_status = req.status.unwrap_or(current_status);
    let new_start = req.start_date.or(current_start);
    let new_end = req.end_date.or(current_end);
    let new_wbs = req.wbs_code.or(current_wbs);
    let new_disc_id = req.discipline_id.unwrap_or(current_disc_id);
    let new_manager_id = req.manager_id.or(current_manager_id);

    sqlx::query(
        "UPDATE projects
         SET name = $1, status = $2, start_date = $3, end_date = $4,
             wbs_code = $5, discipline_id = $6, manager_id = $7, updated_at = now()
         WHERE id = $8",
    )
    .bind(&new_name)
    .bind(&new_status)
    .bind(new_start)
    .bind(new_end)
    .bind(&new_wbs)
    .bind(new_disc_id)
    .bind(new_manager_id)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23514") {
                return AppError::InvalidRequest(
                    "invalid project data (check constraint violated)".to_string(),
                );
            }
            if db_err.code().as_deref() == Some("23503") {
                return AppError::InvalidRequest(
                    "referenced discipline_id or manager_id does not exist".to_string(),
                );
            }
            match db_err.constraint() {
                Some("projects_wbs_code_key") => AppError::Conflict(format!(
                    "wbs_code '{}' already exists",
                    new_wbs.as_deref().unwrap_or("")
                )),
                _ => AppError::Database(e),
            }
        }
        _ => AppError::Database(e),
    })?;

    let proj = fetch_project_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch updated project".to_string()))?;

    Ok(Json(proj))
}

/// DELETE /api/v1/projects/{id}
pub async fn delete_project(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    // 紐づく文書の存在チェック
    let doc_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM documents WHERE project_id = $1")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if doc_count > 0 {
        return Err(AppError::Conflict(
            "cannot delete project with associated documents".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("project {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_project_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<Option<ProjectResponse>, AppError> {
    let row = sqlx::query(
        "SELECT p.id, p.name, p.status, p.start_date, p.end_date, p.wbs_code,
                di.id as disc_id, di.code as disc_code, di.name as disc_name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name,
                e.id as manager_id, e.name as manager_name
         FROM projects p
         JOIN disciplines di ON di.id = p.discipline_id
         JOIN departments d ON d.id = di.department_id
         LEFT JOIN employees e ON e.id = p.manager_id
         WHERE p.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    use sqlx::Row;
    Ok(row.map(|r| {
        ProjectRow {
            id: r.get("id"),
            name: r.get("name"),
            status: r.get("status"),
            start_date: r.get("start_date"),
            end_date: r.get("end_date"),
            wbs_code: r.get("wbs_code"),
            disc_id: r.get("disc_id"),
            disc_code: r.get("disc_code"),
            disc_name: r.get("disc_name"),
            dept_id: r.get("dept_id"),
            dept_code: r.get("dept_code"),
            dept_name: r.get("dept_name"),
            manager_id: r.get("manager_id"),
            manager_name: r.get("manager_name"),
        }
        .into()
    }))
}
