use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /disciplines ──────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_disciplines_returns_list(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;
    helpers::insert_discipline(&pool, "ELEC", "電気設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/disciplines")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_disciplines_with_department_filter(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept_a = helpers::insert_department(&pool, "001", "技術部", None).await;
    let dept_b = helpers::insert_department(&pool, "002", "営業部", None).await;
    helpers::insert_discipline(&pool, "MECH", "機械設計", dept_a).await;
    helpers::insert_discipline(&pool, "SALE", "営業企画", dept_b).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/disciplines?department_id={}", dept_a))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["code"], "MECH");
}

// ── POST /disciplines ─────────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_discipline_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/disciplines")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": "MECH",
                        "name": "機械設計",
                        "department_id": dept
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "MECH");
    assert_eq!(body["name"], "機械設計");
    assert_eq!(body["department"]["name"], "技術部");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_discipline_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/disciplines")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": "MECH",
                        "name": "機械設計",
                        "department_id": dept
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_discipline_duplicate_code_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/disciplines")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "code": "MECH",
                        "name": "別名",
                        "department_id": dept
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── GET /disciplines/{id} ─────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_discipline_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc_id = helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/disciplines/{}", disc_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["code"], "MECH");
    assert_eq!(body["department"]["code"], "001");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_discipline_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/disciplines/00000000-0000-0000-0000-000000000000")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── PUT /disciplines/{id} ─────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_discipline_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc_id = helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/disciplines/{}", disc_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "機械補修設計" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "機械補修設計");
    assert_eq!(body["code"], "MECH"); // code は変わらない
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_discipline_code_change_returns_422(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc_id = helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/disciplines/{}", disc_id))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "code": "ELEC" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_discipline_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let dept = helpers::insert_department(&pool, "001", "技術部", None).await;
    let disc_id = helpers::insert_discipline(&pool, "MECH", "機械設計", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/disciplines/{}", disc_id))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({ "name": "変更" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
