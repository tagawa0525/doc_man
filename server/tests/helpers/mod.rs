#![allow(dead_code)]

use std::sync::Arc;

use axum::Router;
use axum::body::to_bytes;
use axum::http::Response;
use doc_man::services::mail::StubMailSender;
use doc_man::{app_with_state, state::AppState};
use serde_json::Value;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct TestUser {
    pub id: Uuid,
    pub employee_code: String,
}

pub fn build_test_app(pool: PgPool) -> Router {
    let state = AppState {
        db: pool,
        mail_sender: Arc::new(StubMailSender),
    };
    app_with_state(state)
}

pub async fn parse_body(response: Response<axum::body::Body>) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

pub async fn insert_admin(pool: &PgPool) -> TestUser {
    insert_employee(pool, "ADMIN001", "admin").await
}

pub async fn insert_general(pool: &PgPool) -> TestUser {
    insert_employee(pool, "GEN001", "general").await
}

pub async fn insert_employee(pool: &PgPool, code: &str, role: &str) -> TestUser {
    let row = sqlx::query(
        "INSERT INTO employees (name, employee_code, role, is_active)
         VALUES ($1, $2, $3, true)
         RETURNING id",
    )
    .bind(format!("Test {code}"))
    .bind(code)
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap();

    TestUser {
        id: row.get("id"),
        employee_code: code.to_string(),
    }
}

pub async fn insert_department(
    pool: &PgPool,
    code: &str,
    name: &str,
    parent_id: Option<Uuid>,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO departments (code, name, parent_id, effective_from)
         VALUES ($1, $2, $3, '2020-01-01')
         RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(parent_id)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_employee_inactive(pool: &PgPool, code: &str, role: &str) -> TestUser {
    let row = sqlx::query(
        "INSERT INTO employees (name, employee_code, role, is_active)
         VALUES ($1, $2, $3, false)
         RETURNING id",
    )
    .bind(format!("Test {code}"))
    .bind(code)
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap();

    TestUser {
        id: row.get("id"),
        employee_code: code.to_string(),
    }
}

pub async fn assign_department(
    pool: &PgPool,
    employee_id: Uuid,
    department_id: Uuid,
    is_primary: bool,
) {
    sqlx::query(
        "INSERT INTO employee_departments (employee_id, department_id, is_primary, effective_from)
         VALUES ($1, $2, $3, CURRENT_DATE)",
    )
    .bind(employee_id)
    .bind(department_id)
    .bind(is_primary)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn insert_discipline(pool: &PgPool, code: &str, name: &str, department_id: Uuid) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO disciplines (code, name, department_id)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(department_id)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_document_kind(pool: &PgPool, code: &str, name: &str, seq_digits: i32) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO document_kinds (code, name, seq_digits)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(code)
    .bind(name)
    .bind(seq_digits)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_document_register(
    pool: &PgPool,
    register_code: &str,
    doc_kind_id: Uuid,
    department_id: Uuid,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO document_registers (register_code, doc_kind_id, department_id, file_server_root)
         VALUES ($1, $2, $3, '/default/path')
         RETURNING id",
    )
    .bind(register_code)
    .bind(doc_kind_id)
    .bind(department_id)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_project(
    pool: &PgPool,
    name: &str,
    discipline_id: Uuid,
    manager_id: Option<Uuid>,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO projects (name, discipline_id, manager_id)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(name)
    .bind(discipline_id)
    .bind(manager_id)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_project_with_status(
    pool: &PgPool,
    name: &str,
    discipline_id: Uuid,
    manager_id: Option<Uuid>,
    status: &str,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO projects (name, discipline_id, manager_id, status)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(name)
    .bind(discipline_id)
    .bind(manager_id)
    .bind(status)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_project_with_created_at(
    pool: &PgPool,
    name: &str,
    discipline_id: Uuid,
    manager_id: Option<Uuid>,
    created_at: &str,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO projects (name, discipline_id, manager_id, created_at)
         VALUES ($1, $2, $3, $4::timestamptz)
         RETURNING id",
    )
    .bind(name)
    .bind(discipline_id)
    .bind(manager_id)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_project_with_wbs(
    pool: &PgPool,
    name: &str,
    discipline_id: Uuid,
    manager_id: Option<Uuid>,
    wbs_code: &str,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO projects (name, discipline_id, manager_id, wbs_code)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(name)
    .bind(discipline_id)
    .bind(manager_id)
    .bind(wbs_code)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_document(
    pool: &PgPool,
    doc_number: &str,
    title: &str,
    author_id: Uuid,
    doc_kind_id: Uuid,
    project_id: Uuid,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, project_id)
         VALUES ($1, $2, $3, $4, '設計', $5)
         RETURNING id",
    )
    .bind(doc_number)
    .bind(title)
    .bind(author_id)
    .bind(doc_kind_id)
    .bind(project_id)
    .fetch_one(pool)
    .await
    .unwrap();

    let doc_id: Uuid = row.get("id");
    let file_path = format!("{doc_number}/0");

    sqlx::query(
        "INSERT INTO document_revisions (document_id, revision, file_path, created_by)
         VALUES ($1, 0, $2, $3)",
    )
    .bind(doc_id)
    .bind(&file_path)
    .bind(author_id)
    .execute(pool)
    .await
    .unwrap();

    doc_id
}

pub async fn insert_document_with_dept(
    pool: &PgPool,
    doc_number: &str,
    title: &str,
    author_id: Uuid,
    doc_kind_id: Uuid,
    project_id: Uuid,
    frozen_dept_code: &str,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, project_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(doc_number)
    .bind(title)
    .bind(author_id)
    .bind(doc_kind_id)
    .bind(frozen_dept_code)
    .bind(project_id)
    .fetch_one(pool)
    .await
    .unwrap();

    let doc_id: Uuid = row.get("id");
    let file_path = format!("{doc_number}/0");

    sqlx::query(
        "INSERT INTO document_revisions (document_id, revision, file_path, created_by)
         VALUES ($1, 0, $2, $3)",
    )
    .bind(doc_id)
    .bind(&file_path)
    .bind(author_id)
    .execute(pool)
    .await
    .unwrap();

    doc_id
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_document_with_created_at(
    pool: &PgPool,
    doc_number: &str,
    title: &str,
    author_id: Uuid,
    doc_kind_id: Uuid,
    project_id: Uuid,
    frozen_dept_code: &str,
    created_at: &str,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO documents (doc_number, title, author_id, doc_kind_id, frozen_dept_code, project_id, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz)
         RETURNING id",
    )
    .bind(doc_number)
    .bind(title)
    .bind(author_id)
    .bind(doc_kind_id)
    .bind(frozen_dept_code)
    .bind(project_id)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .unwrap();

    let doc_id: Uuid = row.get("id");
    let file_path = format!("{doc_number}/0");

    sqlx::query(
        "INSERT INTO document_revisions (document_id, revision, file_path, created_by)
         VALUES ($1, 0, $2, $3)",
    )
    .bind(doc_id)
    .bind(&file_path)
    .bind(author_id)
    .execute(pool)
    .await
    .unwrap();

    doc_id
}

pub async fn insert_position(
    pool: &PgPool,
    name: &str,
    default_role: &str,
    sort_order: i32,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO positions (name, default_role, sort_order)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(name)
    .bind(default_role)
    .bind(sort_order)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_tag(pool: &PgPool, name: &str) -> Uuid {
    let row = sqlx::query("INSERT INTO tags (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap();

    row.get("id")
}

pub async fn insert_approval_step(
    pool: &PgPool,
    document_id: Uuid,
    route_revision: i32,
    document_revision: i32,
    step_order: i32,
    approver_id: Uuid,
) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO approval_steps (document_id, route_revision, document_revision, step_order, approver_id)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id",
    )
    .bind(document_id)
    .bind(route_revision)
    .bind(document_revision)
    .bind(step_order)
    .bind(approver_id)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}

pub async fn insert_department_inactive(pool: &PgPool, code: &str, name: &str) -> Uuid {
    let row = sqlx::query(
        "INSERT INTO departments (code, name, effective_from, effective_to)
         VALUES ($1, $2, '2020-01-01', '2025-12-31')
         RETURNING id",
    )
    .bind(code)
    .bind(name)
    .fetch_one(pool)
    .await
    .unwrap();

    row.get("id")
}
