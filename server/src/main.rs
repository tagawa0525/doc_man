use doc_man::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("failed to connect to database");

    doc_man::MIGRATOR
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState { db: pool };
    let app = doc_man::app_with_state(state);

    let listener = TcpListener::bind(&bind_addr).await.expect("failed to bind");

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("server error");
}
