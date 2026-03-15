mod routes;

use axum::{Router, routing::{get, post}};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    pub db:            sqlx::PgPool,
    pub anthropic_key: String,
    pub openai_key:    String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = sqlx::PgPool::connect(&database_url).await
        .expect("Failed to connect to PostgreSQL — is it running?");

    // Ensure pgvector extension exists
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool).await.ok();

    tracing::info!("✓ Connected to PostgreSQL + pgvector");

    let state = Arc::new(AppState {
        db:            pool,
        anthropic_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
        openai_key:    std::env::var("OPENAI_API_KEY").unwrap_or_default(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health",             get(|| async { "Mockupface backend OK" }))
        .route("/api/ocr",            post(routes::ocr::handle))
        .route("/api/rag/search",     post(routes::rag::search))
        .route("/api/rag/store",      post(routes::rag::store))
        .route("/api/prompts",        post(routes::prompts::handle))
        .route("/api/generate-image", post(routes::images::handle))
        .route("/api/qa",             post(routes::qa::handle))
        .route("/api/pipeline",       post(routes::pipeline::handle))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🚀 Mockupface listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
