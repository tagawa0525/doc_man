pub mod auth;
pub mod error;
pub mod pagination;
pub mod routes;
pub mod state;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

use axum::Router;
use state::AppState;

/// テスト用: AppState なしのシンプルなアプリ（ヘルスチェックのみ）
pub fn app() -> Router {
    Router::new().route(
        "/health",
        axum::routing::get(|| async { axum::Json(serde_json::json!({ "status": "ok" })) }),
    )
}

/// 本番 / 統合テスト用: DB接続込みのアプリ
pub fn app_with_state(state: AppState) -> Router {
    routes::build_router(state)
}
