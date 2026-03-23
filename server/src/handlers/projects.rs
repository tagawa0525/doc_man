use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::{QueryBuilder, Row};
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::authorization;
use crate::error::AppError;
use crate::models::project::{
    CreateProjectRequest, ProjectResponse, ProjectRow, UpdateProjectRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ProjectListQuery {
    pub status: Option<String>,
    pub discipline_ids: Option<String>,
    pub wbs_code: Option<String>,
    pub q: Option<String>,
    pub dept_ids: Option<String>,
    pub fiscal_year: Option<i32>,
    pub fiscal_years: Option<String>,
    pub manager_name: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
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

    let search = params
        .q
        .filter(|s| !s.is_empty())
        .map(|s| escape_like(&s).to_lowercase());

    let mut dept_ids: Vec<Uuid> = Vec::new();
    if let Some(ref raw) = params.dept_ids {
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            let id: Uuid = trimmed.parse().map_err(|_| {
                AppError::InvalidRequest(
                    "invalid dept_ids parameter: must be comma-separated UUIDs".to_string(),
                )
            })?;
            dept_ids.push(id);
        }
    }

    let wbs_code = params
        .wbs_code
        .filter(|s| !s.is_empty())
        .map(|s| escape_like(&s).to_lowercase());

    let manager_name = params
        .manager_name
        .filter(|s| !s.is_empty())
        .map(|s| escape_like(&s).to_lowercase());

    let mut discipline_ids: Vec<Uuid> = Vec::new();
    if let Some(ref raw) = params.discipline_ids {
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            let id: Uuid = trimmed.parse().map_err(|_| {
                AppError::InvalidRequest(
                    "invalid discipline_ids parameter: must be comma-separated UUIDs".to_string(),
                )
            })?;
            discipline_ids.push(id);
        }
    }

    let mut fiscal_years: Vec<i32> = Vec::new();
    if let Some(ref raw) = params.fiscal_years {
        for part in raw.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            let year: i32 = trimmed.parse().map_err(|_| {
                AppError::InvalidRequest(format!("invalid fiscal_years value: {trimmed}"))
            })?;
            fiscal_years.push(year);
        }
    }
    if let Some(year) = params.fiscal_year {
        fiscal_years.push(year);
    }

    let fiscal_date_ranges: Vec<(NaiveDate, NaiveDate)> = fiscal_years
        .iter()
        .map(|&y| {
            (
                NaiveDate::from_ymd_opt(y, 4, 1).unwrap(),
                NaiveDate::from_ymd_opt(y + 1, 4, 1).unwrap(),
            )
        })
        .collect();

    // COUNT クエリ
    let mut count_qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "SELECT COUNT(*) FROM projects p
         JOIN disciplines di ON di.id = p.discipline_id
         JOIN departments d ON d.id = di.department_id
         LEFT JOIN employees e ON e.id = p.manager_id
         WHERE 1=1",
    );
    push_project_filters(
        &mut count_qb,
        params.status.as_deref(),
        &discipline_ids,
        wbs_code.as_deref(),
        search.as_deref(),
        &dept_ids,
        &fiscal_date_ranges,
        manager_name.as_deref(),
    );
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // データクエリ
    let mut data_qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        "SELECT p.id, p.name, p.status, p.start_date, p.end_date, p.wbs_code,
                di.id as disc_id, di.code as disc_code, di.name as disc_name,
                d.id as dept_id, d.code as dept_code, d.name as dept_name,
                e.id as manager_id, e.name as manager_name
         FROM projects p
         JOIN disciplines di ON di.id = p.discipline_id
         JOIN departments d ON d.id = di.department_id
         LEFT JOIN employees e ON e.id = p.manager_id
         WHERE 1=1",
    );
    push_project_filters(
        &mut data_qb,
        params.status.as_deref(),
        &discipline_ids,
        wbs_code.as_deref(),
        search.as_deref(),
        &dept_ids,
        &fiscal_date_ranges,
        manager_name.as_deref(),
    );
    data_qb.push(" ORDER BY p.name, p.id");
    if !params.pagination.is_unpaged() {
        data_qb.push(" LIMIT ");
        data_qb.push_bind(params.pagination.limit());
        data_qb.push(" OFFSET ");
        data_qb.push_bind(params.pagination.offset());
    }

    let rows = data_qb
        .build()
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

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

#[allow(clippy::too_many_arguments)]
fn push_project_filters(
    qb: &mut QueryBuilder<sqlx::Postgres>,
    status: Option<&str>,
    discipline_ids: &[Uuid],
    wbs_code: Option<&str>,
    search: Option<&str>,
    dept_ids: &[Uuid],
    fiscal_date_ranges: &[(NaiveDate, NaiveDate)],
    manager_name: Option<&str>,
) {
    if let Some(s) = status {
        qb.push(" AND p.status = ");
        qb.push_bind(s.to_string());
    }
    if !discipline_ids.is_empty() {
        qb.push(" AND p.discipline_id IN (");
        let mut separated = qb.separated(", ");
        for id in discipline_ids {
            separated.push_bind(*id);
        }
        separated.push_unseparated(")");
    }
    if let Some(w) = wbs_code {
        qb.push(" AND LOWER(p.wbs_code) LIKE '%' || ");
        qb.push_bind(w.to_string());
        qb.push(" || '%' ESCAPE '\\'");
    }
    if let Some(q) = search {
        qb.push(" AND LOWER(p.name) LIKE '%' || ");
        qb.push_bind(q.to_string());
        qb.push(" || '%' ESCAPE '\\'");
    }
    if !dept_ids.is_empty() {
        qb.push(" AND d.id IN (");
        let mut separated = qb.separated(", ");
        for id in dept_ids {
            separated.push_bind(*id);
        }
        separated.push_unseparated(")");
    }
    if !fiscal_date_ranges.is_empty() {
        qb.push(" AND (");
        for (i, (start, end)) in fiscal_date_ranges.iter().enumerate() {
            if i > 0 {
                qb.push(" OR ");
            }
            qb.push("(p.created_at >= ");
            qb.push_bind(*start);
            qb.push(" AND p.created_at < ");
            qb.push_bind(*end);
            qb.push(")");
        }
        qb.push(")");
    }
    if let Some(mn) = manager_name {
        qb.push(" AND LOWER(e.name) LIKE '%' || ");
        qb.push_bind(mn.to_string());
        qb.push(" || '%' ESCAPE '\\'");
    }
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

    let dept_id = authorization::get_discipline_department_id(&state.db, req.discipline_id).await?;
    authorization::check_department_access(&user, dept_id)?;

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
        .ok_or_else(|| AppError::NotFound(format!("project {id} not found")))?;

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
    .ok_or_else(|| AppError::NotFound(format!("project {id} not found")))?;

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

    let dept_id = authorization::get_project_department_id(&state.db, id).await?;
    authorization::check_department_access(&user, dept_id)?;

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

    // discipline_id 変更時は移動先部署のスコープもチェック
    if new_disc_id != current_disc_id {
        let new_dept_id =
            authorization::get_discipline_department_id(&state.db, new_disc_id).await?;
        authorization::check_department_access(&user, new_dept_id)?;
    }
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
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23503") => {
                AppError::Conflict("cannot delete project with associated documents".to_string())
            }
            _ => AppError::Database(e),
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("project {id} not found")));
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
