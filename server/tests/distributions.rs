mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

use helpers::*;

/// マスタデータ一式を準備するヘルパー
struct MasterData {
    kind: uuid::Uuid,
    proj: uuid::Uuid,
}

async fn setup_master(pool: &PgPool, admin: &TestUser) -> MasterData {
    let dept = insert_department(pool, "設計", "設計部", None).await;
    let kind = insert_document_kind(pool, "内", "社内文書", 3).await;
    let disc = insert_discipline(pool, "MECH", "機械", dept).await;
    insert_document_register(pool, "内設計", kind, dept).await;
    let proj = insert_project(pool, "テストPJ", disc, Some(admin.id)).await;
    MasterData { kind, proj }
}

// ── GET: 空配列 ───────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_distributions_returns_empty_array(pool: PgPool) {
    let app = build_test_app(pool.clone());
    let admin = insert_admin(&pool).await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_body(response).await;
    assert!(body.as_array().unwrap().is_empty());
}

// ── POST: 配布作成 → 201 ─────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_distributions_creates_records(pool: PgPool) {
    let app = build_test_app(pool.clone());
    let admin = insert_admin(&pool).await;
    let recipient1 = insert_employee(&pool, "GEN001", "general").await;
    let recipient2 = insert_employee(&pool, "GEN002", "general").await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"recipient_ids": [recipient1.id, recipient2.id]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_body(response).await;
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    // distributed_by が admin であること
    assert_eq!(arr[0]["distributed_by"]["id"], admin.id.to_string());
    // 両レコードの distributed_at が同じタイムスタンプであること（バッチ単位）
    assert_eq!(arr[0]["distributed_at"], arr[1]["distributed_at"]);
}

// ── POST: draft 文書でも 201 ──────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_distributions_on_draft_returns_201(pool: PgPool) {
    let app = build_test_app(pool.clone());
    let admin = insert_admin(&pool).await;
    let recipient = insert_employee(&pool, "GEN001", "general").await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;
    // 文書はデフォルトで draft ステータス

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"recipient_ids": [recipient.id]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

// ── POST: viewer は 403 ──────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_distributions_viewer_returns_403(pool: PgPool) {
    let app = build_test_app(pool.clone());
    let admin = insert_admin(&pool).await;
    let viewer = insert_employee(&pool, "VW001", "viewer").await;
    let recipient = insert_employee(&pool, "GEN001", "general").await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({"recipient_ids": [recipient.id]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── POST: 空の recipient_ids は 400 ──

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_distributions_empty_recipients_returns_400(pool: PgPool) {
    let app = build_test_app(pool.clone());
    let admin = insert_admin(&pool).await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"recipient_ids": []}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ── POST: 再配布可能 ─────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_distributions_allows_redistribution(pool: PgPool) {
    let admin = insert_admin(&pool).await;
    let recipient = insert_employee(&pool, "GEN001", "general").await;
    let data = setup_master(&pool, &admin).await;
    let doc_id = insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let make_request = || {
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/documents/{doc_id}/distributions"))
            .header("Authorization", format!("Bearer {}", admin.employee_code))
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({"recipient_ids": [recipient.id]}).to_string(),
            ))
            .unwrap()
    };

    // 1回目の配布
    let app1 = build_test_app(pool.clone());
    let response1 = app1.oneshot(make_request()).await.unwrap();
    assert_eq!(response1.status(), StatusCode::CREATED);

    // 2回目の配布（同じ宛先に再配布）
    let app2 = build_test_app(pool.clone());
    let response2 = app2.oneshot(make_request()).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);

    // GET で全配布履歴を取得 → 2件あること
    let app3 = build_test_app(pool.clone());
    let response3 = app3
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/documents/{doc_id}/distributions"))
                .header("Authorization", format!("Bearer {}", admin.employee_code))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response3.status(), StatusCode::OK);
    let body = parse_body(response3).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}
