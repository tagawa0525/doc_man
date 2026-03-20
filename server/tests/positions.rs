use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ── GET /positions ──────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_positions_returns_list_sorted_by_sort_order(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    // マイグレーションで7件入っている

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    let data = body.as_array().unwrap();
    assert_eq!(data.len(), 7);
    // sort_order 順に返る
    assert_eq!(data[0]["name"], "社長");
    assert_eq!(data[1]["name"], "部長");
    assert_eq!(data[6]["name"], "派遣");
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_positions_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/positions")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// ── GET /positions/{id} ─────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_position_by_id_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let pos_id = helpers::insert_position(&pool, "テスト役職", "admin", 99).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/positions/{pos_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "テスト役職");
    assert_eq!(body["default_role"], "admin");
    assert_eq!(body["sort_order"], 99);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_position_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/positions/{fake_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── POST /positions ─────────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_position_admin_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/positions")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "テスト職位",
                        "default_role": "admin",
                        "sort_order": 99
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "テスト職位");
    assert_eq!(body["default_role"], "admin");
    assert_eq!(body["sort_order"], 99);
    assert!(body["id"].is_string());
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_position_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/positions")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "テスト職位",
                        "default_role": "viewer",
                        "sort_order": 99
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
async fn post_position_duplicate_name_returns_409(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    // マイグレーションで「社長」は既に存在

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/positions")
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "社長",
                        "default_role": "admin",
                        "sort_order": 1
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ── PUT /positions/{id} ─────────────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_position_admin_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let pos_id = helpers::insert_position(&pool, "テスト役職", "admin", 99).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/positions/{pos_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "テスト役職改",
                        "default_role": "project_manager"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = helpers::parse_body(response).await;
    assert_eq!(body["name"], "テスト役職改");
    assert_eq!(body["default_role"], "project_manager");
    assert_eq!(body["sort_order"], 99); // 変更なし
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_position_non_admin_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_general(&pool).await;
    let pos_id = helpers::insert_position(&pool, "テスト一般", "general", 50).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/positions/{pos_id}"))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(json!({"name": "変更"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_position_not_found_returns_404(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/positions/{fake_id}"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "存在しない"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
