use axum::Router;
use axum::body::to_bytes;
use axum::http::Response;
use doc_man::{app_with_state, state::AppState};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TestUser {
    pub id: Uuid,
    pub employee_code: String,
}

pub fn build_test_app(pool: PgPool) -> Router {
    let state = AppState { db: pool };
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
    .bind(format!("Test {}", code))
    .bind(code)
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap();

    use sqlx::Row;
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

    use sqlx::Row;
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

    use sqlx::Row;
    row.get("id")
}
