pub mod auth;
pub mod error;
pub mod pagination;
pub mod routes;
pub mod state;

use axum::Router;

pub fn app() -> Router {
    routes::build_router()
}
