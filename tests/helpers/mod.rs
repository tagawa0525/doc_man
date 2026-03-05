use axum::Router;
use doc_man::{app_with_state, state::AppState};
use sqlx::PgPool;

pub fn build_test_app(pool: PgPool) -> Router {
    let state = AppState { db: pool };
    app_with_state(state)
}
