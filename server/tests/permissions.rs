use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;

mod helpers;

// ============================================================
// 権限マトリクス網羅テスト
//
// 各エンドポイントの権限チェックが正しく動作するかを
// 全ロール（admin, project_manager, general, viewer）で検証する。
// 既存テストで十分カバーされているケースは省略し、
// 不足しているロール×エンドポイントの組み合わせのみ追加。
// ============================================================

// ── helpers ──────────────────────────────────────────────────

/// 共通のマスタデータ（部署・分野・文書種別・プロジェクト）を作成
async fn setup_master_data(pool: &PgPool) -> MasterData {
    let dept = helpers::insert_department(pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(pool, "MECH", "機械", dept).await;
    let kind = helpers::insert_document_kind(pool, "内", "社内", 3).await;
    let proj = helpers::insert_project(pool, "テスト", disc, None).await;
    MasterData {
        _dept: dept,
        disc,
        kind,
        proj,
    }
}

struct MasterData {
    _dept: uuid::Uuid,
    disc: uuid::Uuid,
    kind: uuid::Uuid,
    proj: uuid::Uuid,
}

// ── positions: admin-only write ──────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_position_pm_returns_403(pool: PgPool) {
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
                    json!({"name": "テスト", "default_role": "viewer", "sort_order": 99}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_position_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let pos_id = helpers::insert_position(&pool, "テストPM職", "general", 50).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/positions/{pos_id}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "変更"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_positions_viewer_returns_200_perms(pool: PgPool) {
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

// ── admin-only endpoints: project_manager → 403 ─────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_department_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"code": "TEST", "name": "テスト部", "effective_from": "2024-01-01"})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_department_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/departments/{dept}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "新設計部"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_employee_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "name": "テスト社員",
                        "employee_code": "E999",
                        "role": "general",
                        "department_id": dept,
                        "effective_from": "2024-01-01"
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
async fn put_employee_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let target = helpers::insert_employee(&pool, "GEN001", "general").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/employees/{}", target.id))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "新名前"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_discipline_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/disciplines")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"code": "ELEC", "name": "電気", "department_id": dept}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_discipline_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let disc = helpers::insert_discipline(&pool, "MECH", "機械", dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/disciplines/{disc}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "新機械"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_kind_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-kinds")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"code": "外", "name": "社外", "seq_digits": 3}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_kind_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-kinds/{kind}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "社内新"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_register_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/document-registers")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "register_code": "REG001",
                        "doc_kind_id": kind,
                        "department_id": dept,
                        "file_server_root": "/path"
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
async fn put_document_register_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let dept = helpers::insert_department(&pool, "設計", "設計部", None).await;
    let kind = helpers::insert_document_kind(&pool, "内", "社内", 3).await;
    let reg = helpers::insert_document_register(&pool, "REG001", kind, dept).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/document-registers/{reg}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"file_server_root": "/new/path"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── admin-only endpoints: delete ────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_project_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let data = setup_master_data(&pool).await;
    let proj = helpers::insert_project(&pool, "削除対象", data.disc, Some(pm.id)).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/projects/{proj}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn delete_document_pm_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let data = setup_master_data(&pool).await;
    let doc_id = helpers::insert_document(
        &pool,
        "内設計-2603001",
        "テスト文書",
        admin.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── admin+PM endpoints: PM → 成功, general → 403 ────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_approval_steps_pm_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let approver = helpers::insert_employee(&pool, "APP001", "general").await;
    let data = setup_master_data(&pool).await;
    let doc_id = helpers::insert_document(
        &pool,
        "内設計-2603001",
        "テスト",
        pm.id,
        data.kind,
        data.proj,
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "steps": [{"step_order": 1, "approver_id": approver.id}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_approval_steps_general_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let general = helpers::insert_employee(&pool, "GEN001", "general").await;
    let approver = helpers::insert_employee(&pool, "APP001", "general").await;
    let data = setup_master_data(&pool).await;
    let doc_id = helpers::insert_document(
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
                .uri(format!("/api/v1/documents/{doc_id}/approval-steps"))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "steps": [{"step_order": 1, "approver_id": approver.id}]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── viewer以外（admin/PM/general）許可: documents POST ──────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_pm_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;
    let data = setup_master_data(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "title": "テスト文書",
                        "doc_kind_id": data.kind,
                        "project_id": data.proj
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_document_general_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_employee(&pool, "GEN001", "general").await;
    let data = setup_master_data(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({
                        "title": "テスト文書",
                        "doc_kind_id": data.kind,
                        "project_id": data.proj
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

// ── documents PUT: viewer → 403 ─────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_document_viewer_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let admin = helpers::insert_admin(&pool).await;
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;
    let data = setup_master_data(&pool).await;
    let doc_id = helpers::insert_document(
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
                .method("PUT")
                .uri(format!("/api/v1/documents/{doc_id}"))
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"title": "新タイトル"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── projects PUT: general → 403 ─────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn put_project_general_returns_403(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let general = helpers::insert_employee(&pool, "GEN001", "general").await;
    let data = setup_master_data(&pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/projects/{}", data.proj))
                .header("Authorization", format!("Bearer {}", general.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "新プロジェクト"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

// ── tags POST: PM/general → 成功 ────────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn post_tag_pm_returns_201(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let pm = helpers::insert_employee(&pool, "PM001", "project_manager").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/tags")
                .header("Authorization", format!("Bearer {}", pm.employee_code))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(
                    json!({"name": "PMタグ"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

// ── GET endpoints: viewer can read ──────────────────────────

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_departments_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/departments")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_employees_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/employees")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_documents_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/documents")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_projects_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/projects")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrator = "doc_man::MIGRATOR")]
async fn get_tags_viewer_returns_200(pool: PgPool) {
    let app = helpers::build_test_app(pool.clone());
    let viewer = helpers::insert_employee(&pool, "VIEW001", "viewer").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/tags")
                .header("Authorization", format!("Bearer {}", viewer.employee_code))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
