use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /departments ──────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_departments_returns_tree(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    // 親部署
    let parent_id = helpers::insert_department(&pool, "001", "技術部", None).await;
    // 子部署
    helpers::insert_department(&pool, "002", "設計課", Some(parent_id)).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    // 配列のルートには親のみ
    assert!(body.is_array());
    let roots: Vec<&Value> = body.as_array().unwrap().iter().collect();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["code"], "001");
    // children に子部署
    let children = roots[0]["children"].as_array().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0]["code"], "002");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_departments_excludes_inactive_by_default(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    helpers::insert_department_inactive(&pool, "001", "廃止部署").await;
    helpers::insert_department(&pool, "002", "現役部署", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let roots = body.as_array().unwrap();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["code"], "002");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_departments_include_inactive(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    helpers::insert_department_inactive(&pool, "001", "廃止部署").await;
    helpers::insert_department(&pool, "002", "現役部署", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/departments?include_inactive=true")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

// ── POST /departments ─────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_department_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": "001",
                        "name": "技術部",
                        "effective_from": "2026-01-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "001");
    assert_eq!(body["name"], "技術部");
    assert!(body["id"].is_string());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_department_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": "001",
                        "name": "技術部",
                        "effective_from": "2026-01-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── GET /departments/:id ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_department_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept_id = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/departments/{}", dept_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_department_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/departments/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /departments/:id ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_department_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept_id = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/departments/{}", dept_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "技術開発部" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "技術開発部");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_department_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept_id = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/departments/{}", dept_id))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "技術開発部" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
