use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{AuthenticatedUser, Role};
use crate::error::AppError;
use crate::models::employee::{
    CreateEmployeeRequest, EmployeeResponse, EmployeeRow, UpdateEmployeeRequest,
};
use crate::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EmployeeListQuery {
    pub department_id: Option<Uuid>,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

fn default_is_active() -> bool {
    true
}

/// GET /api/v1/employees
pub async fn list_employees(
    _user: AuthenticatedUser,
    Query(params): Query<EmployeeListQuery>,
    State(state): State<AppState>,
) -> Result<Json<PaginatedResponse<EmployeeResponse>>, AppError> {
    if let Err(e) = params.pagination.validate() {
        return Err(AppError::InvalidRequest(e));
    }

    let (total, rows) = if let Some(dept_id) = params.department_id {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM employees e
             LEFT JOIN employee_departments ed ON ed.employee_id = e.id
               AND ed.effective_to IS NULL AND ed.is_primary = true
             WHERE e.is_active = $1 AND ed.department_id = $2",
        )
        .bind(params.is_active)
        .bind(dept_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::Database)?;

        let rows = sqlx::query(
            "SELECT e.id, e.name, e.employee_code, e.email, e.ad_account, e.role, e.is_active,
                    ed.department_id as dept_id, d.name as dept_name
             FROM employees e
             LEFT JOIN employee_departments ed ON ed.employee_id = e.id
               AND ed.effective_to IS NULL AND ed.is_primary = true
             LEFT JOIN departments d ON d.id = ed.department_id
             WHERE e.is_active = $1 AND ed.department_id = $2
             ORDER BY e.employee_code NULLS LAST, e.id
             LIMIT $3 OFFSET $4",
        )
        .bind(params.is_active)
        .bind(dept_id)
        .bind(params.pagination.limit())
        .bind(params.pagination.offset())
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

        (total, rows)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM employees WHERE is_active = $1")
            .bind(params.is_active)
            .fetch_one(&state.db)
            .await
            .map_err(AppError::Database)?;

        let rows = sqlx::query(
            "SELECT e.id, e.name, e.employee_code, e.email, e.ad_account, e.role, e.is_active,
                    ed.department_id as dept_id, d.name as dept_name
             FROM employees e
             LEFT JOIN employee_departments ed ON ed.employee_id = e.id
               AND ed.effective_to IS NULL AND ed.is_primary = true
             LEFT JOIN departments d ON d.id = ed.department_id
             WHERE e.is_active = $1
             ORDER BY e.employee_code NULLS LAST, e.id
             LIMIT $2 OFFSET $3",
        )
        .bind(params.is_active)
        .bind(params.pagination.limit())
        .bind(params.pagination.offset())
        .fetch_all(&state.db)
        .await
        .map_err(AppError::Database)?;

        (total, rows)
    };

    use sqlx::Row;
    let data: Vec<EmployeeResponse> = rows
        .into_iter()
        .map(|r| {
            EmployeeRow {
                id: r.get("id"),
                name: r.get("name"),
                employee_code: r.get("employee_code"),
                email: r.get("email"),
                ad_account: r.get("ad_account"),
                role: r.get("role"),
                is_active: r.get("is_active"),
                dept_id: r.get("dept_id"),
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

/// POST /api/v1/employees
pub async fn create_employee(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<EmployeeResponse>), AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    let row = sqlx::query(
        "INSERT INTO employees (name, employee_code, email, ad_account, role)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
    .bind(&req.name)
    .bind(&req.employee_code)
    .bind(&req.email)
    .bind(&req.ad_account)
    .bind(req.role.as_deref().unwrap_or("general"))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23514") {
                return AppError::InvalidRequest(
                    "invalid employee data (check constraint violated)".to_string(),
                );
            }
            match db_err.constraint() {
                Some("employees_employee_code_key") => AppError::Conflict(format!(
                    "employee code '{}' already exists",
                    req.employee_code.as_deref().unwrap_or("")
                )),
                Some("employees_ad_account_key") => AppError::Conflict(format!(
                    "ad_account '{}' already exists",
                    req.ad_account.as_deref().unwrap_or("")
                )),
                _ => AppError::Database(e),
            }
        }
        _ => AppError::Database(e),
    })?;

    use sqlx::Row;
    let employee_id: Uuid = row.get("id");

    sqlx::query(
        "INSERT INTO employee_departments (employee_id, department_id, is_primary, effective_from)
         VALUES ($1, $2, true, $3)",
    )
    .bind(employee_id)
    .bind(req.department_id)
    .bind(req.effective_from)
    .execute(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23503") => {
            AppError::InvalidRequest(format!(
                "department_id '{}' does not exist",
                req.department_id
            ))
        }
        _ => AppError::Database(e),
    })?;

    tx.commit().await.map_err(AppError::Database)?;

    let emp = fetch_employee_by_id(&state, employee_id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch created employee".to_string()))?;

    Ok((StatusCode::CREATED, Json(emp)))
}

/// GET /api/v1/employees/{id}
pub async fn get_employee(
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<EmployeeResponse>, AppError> {
    let emp = fetch_employee_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("employee {} not found", id)))?;

    Ok(Json(emp))
}

/// PUT /api/v1/employees/{id}
pub async fn update_employee(
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<Json<EmployeeResponse>, AppError> {
    if user.role != Role::Admin {
        return Err(AppError::Forbidden("admin role required".to_string()));
    }

    let existing =
        sqlx::query("SELECT name, email, ad_account, role, is_active FROM employees WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::NotFound(format!("employee {} not found", id)))?;

    use sqlx::Row;
    let current_name: String = existing.get("name");
    let current_email: Option<String> = existing.get("email");
    let current_ad_account: Option<String> = existing.get("ad_account");
    let current_role: String = existing.get("role");
    let current_is_active: bool = existing.get("is_active");

    let new_name = req.name.unwrap_or(current_name);
    let new_email = req.email.or(current_email);
    let new_ad_account = req.ad_account.or(current_ad_account);
    let new_role = req.role.unwrap_or(current_role);
    let new_is_active = req.is_active.unwrap_or(current_is_active);

    let mut tx = state.db.begin().await.map_err(AppError::Database)?;

    sqlx::query(
        "UPDATE employees
         SET name = $1, email = $2, ad_account = $3, role = $4, is_active = $5, updated_at = now()
         WHERE id = $6",
    )
    .bind(&new_name)
    .bind(&new_email)
    .bind(&new_ad_account)
    .bind(&new_role)
    .bind(new_is_active)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) => {
            if db_err.code().as_deref() == Some("23514") {
                return AppError::InvalidRequest("invalid role".to_string());
            }
            if db_err.constraint() == Some("employees_ad_account_key") {
                return AppError::Conflict("ad_account already exists".to_string());
            }
            AppError::Database(e)
        }
        _ => AppError::Database(e),
    })?;

    // 退職処理: is_active が false になる場合、有効な所属レコードを閉じる
    if !new_is_active && current_is_active {
        sqlx::query(
            "UPDATE employee_departments
             SET effective_to = CURRENT_DATE
             WHERE employee_id = $1 AND effective_to IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;
    }

    tx.commit().await.map_err(AppError::Database)?;

    let emp = fetch_employee_by_id(&state, id)
        .await?
        .ok_or_else(|| AppError::Internal("failed to fetch updated employee".to_string()))?;

    Ok(Json(emp))
}

// ──────────────── 内部ヘルパー ────────────────

async fn fetch_employee_by_id(
    state: &AppState,
    id: Uuid,
) -> Result<Option<EmployeeResponse>, AppError> {
    let row = sqlx::query(
        "SELECT e.id, e.name, e.employee_code, e.email, e.ad_account, e.role, e.is_active,
                ed.department_id as dept_id, d.name as dept_name
         FROM employees e
         LEFT JOIN employee_departments ed ON ed.employee_id = e.id
           AND ed.effective_to IS NULL AND ed.is_primary = true
         LEFT JOIN departments d ON d.id = ed.department_id
         WHERE e.id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?;

    use sqlx::Row;
    Ok(row.map(|r| {
        EmployeeRow {
            id: r.get("id"),
            name: r.get("name"),
            employee_code: r.get("employee_code"),
            email: r.get("email"),
            ad_account: r.get("ad_account"),
            role: r.get("role"),
            is_active: r.get("is_active"),
            dept_id: r.get("dept_id"),
            dept_name: r.get("dept_name"),
        }
        .into()
    }))
}
