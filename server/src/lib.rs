pub mod auth;
pub mod authorization;
pub mod error;
pub mod handlers;
pub mod models;
pub mod pagination;
pub mod routes;
pub mod services;
pub mod state;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

use axum::Router;
use axum::http::StatusCode;
use axum::routing::any;
use state::AppState;
use tower_http::services::{ServeDir, ServeFile};

/// DB接続込みのアプリを構築して返す（本番・統合テスト共通）
pub fn app_with_state(state: AppState) -> Router {
    let api_router = routes::build_router(state);

    let frontend_dir =
        std::env::var("FRONTEND_DIST_DIR").unwrap_or_else(|_| "frontend/dist".to_string());
    let index_path = format!("{frontend_dir}/index.html");

    // API routes take priority, then static files, then SPA fallback.
    // Unknown /api/* paths return 404 (not index.html) to avoid misleading API clients.
    if std::path::Path::new(&frontend_dir).exists() {
        api_router
            .route("/api/{*path}", any(|| async { StatusCode::NOT_FOUND }))
            .fallback_service(
                ServeDir::new(&frontend_dir).not_found_service(ServeFile::new(&index_path)),
            )
    } else {
        api_router
    }
}
