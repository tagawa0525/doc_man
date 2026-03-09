use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tower::ServiceExt;

mod helpers;

// ── GET /employees ────────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employees_returns_active_by_default(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    helpers::insert_employee(&pool, "E001", "general").await;
    helpers::insert_employee_inactive(&pool, "E002", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    // admin + E001 = 2 active
    assert_eq!(body["meta"]["total"], 2);
    let data = body["data"].as_array().unwrap();
    assert!(data.iter().all(|e| e["is_active"].as_bool().unwrap()));
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employees_with_is_active_false_returns_inactive(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    helpers::insert_employee_inactive(&pool, "E002", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/employees?is_active=false")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["employee_code"], "E002");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employees_with_department_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept_a = helpers::insert_department(&pool, "001", "技術部", None).await;
    let dept_b = helpers::insert_department(&pool, "002", "営業部", None).await;
    let emp1 = helpers::insert_employee(&pool, "E001", "general").await;
    let emp2 = helpers::insert_employee(&pool, "E002", "general").await;
    helpers::assign_department(&pool, emp1.id, dept_a, true).await;
    helpers::assign_department(&pool, emp2.id, dept_b, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/employees?department_id={dept_a}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["meta"]["total"], 1);
    assert_eq!(body["data"][0]["employee_code"], "E001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employees_includes_current_department(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;
    helpers::assign_department(&pool, emp.id, dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let emp_data = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["employee_code"] == "E001")
        .unwrap();
    assert_eq!(emp_data["current_department"]["name"], "技術部");
}

// ── POST /employees ───────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_employee_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "鈴木 花子",
                        "employee_code": "E001",
                        "role": "general",
                        "department_id": dept,
                        "effective_from": "2026-04-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "鈴木 花子");
    assert_eq!(body["employee_code"], "E001");
    assert!(body["is_active"].as_bool().unwrap());
    assert_eq!(body["current_department"]["name"], "技術部");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_employee_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "鈴木 花子",
                        "employee_code": "E001",
                        "role": "general",
                        "department_id": dept,
                        "effective_from": "2026-04-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── GET /employees/{id} ───────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employee_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/employees/{}", emp.id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["employee_code"], "E001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employee_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/employees/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── POST /employees with email ────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_employee_with_email_returns_email_in_response(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "鈴木 花子",
                        "employee_code": "E001",
                        "email": "suzuki@example.com",
                        "role": "general",
                        "department_id": dept,
                        "effective_from": "2026-04-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["email"], "suzuki@example.com");
}

// ── GET /employees returns email ─────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employee_by_id_returns_email(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    // email 付きで従業員を直接INSERT
    let row = sqlx::query(
        "INSERT INTO employees (name, employee_code, email, role, is_active)
         VALUES ('テスト太郎', 'E001', 'test@example.com', 'general', true)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let emp_id: uuid::Uuid = row.get("id");

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/employees/{emp_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["email"], "test@example.com");
}

// ── PUT /employees/{id} with email ───────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_employee_updates_email(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/employees/{}", emp.id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "email": "updated@example.com" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["email"], "updated@example.com");
}

// ── PUT /employees/{id} ───────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_employee_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/employees/{}", emp.id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "田中 一郎" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "田中 一郎");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_employee_retire_closes_department_assignment(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;
    helpers::assign_department(&pool, emp.id, dept, true).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/employees/{}", emp.id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "is_active": false }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert!(!body["is_active"].as_bool().unwrap());
    assert!(body["current_department"].is_null());

    // employee_departments.effective_to が設定されているか確認

    let row = sqlx::query("SELECT effective_to FROM employee_departments WHERE employee_id = $1")
        .bind(emp.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let effective_to: Option<chrono::NaiveDate> = row.get("effective_to");
    assert!(
        effective_to.is_some(),
        "effective_to should be set on retirement"
    );
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_employee_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let emp = helpers::insert_employee(&pool, "E001", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/employees/{}", emp.id))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "田中 一郎" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
