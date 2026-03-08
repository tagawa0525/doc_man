pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod pagination;
pub mod routes;
pub mod services;
pub mod state;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

use axum::Router;
use state::AppState;

/// DB接続込みのアプリを構築して返す（本番・統合テスト共通）
pub fn app_with_state(state: AppState) -> Router {
    routes::build_router(state)
}
