mod common;
mod controllers;
mod models;
mod repository;
mod services;

use axum::{Router, routing::{get, post}};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use common::constants::{DEFAULT_PORT, DEFAULT_BIND_ADDR};
use repository::PgvectorRepository;
use services::{ClaudeService, DalleService, HttpService, OcrService, QaService};

/// Shared application state — injected into every handler via Axum's
/// `State` extractor. All service and repository instances are `Arc`-wrapped
/// for cheap cloning across async tasks.
pub struct AppState {
    // ── Credentials ───────────────────────────────────────────────────────────
    pub anthropic_key: String,
    pub openai_key:    String,

    // ── Services ──────────────────────────────────────────────────────────────
    pub ocr_service:    OcrService,
    pub claude_service: ClaudeService,
    pub dalle_service:  DalleService,
    pub qa_service:     QaService,

    // ── Repository ────────────────────────────────────────────────────────────
    pub repository: PgvectorRepository,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    // ── Tracing ───────────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Database ──────────────────────────────────────────────────────────────
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL — verify DATABASE_URL and that the server is running");

    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool)
        .await
        .ok();

    tracing::info!("Connected to PostgreSQL with pgvector extension");

    // ── API keys ──────────────────────────────────────────────────────────────
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    let openai_key    = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    if anthropic_key.is_empty() {
        tracing::warn!("ANTHROPIC_API_KEY is not set — Claude endpoints will return errors");
    }
    if openai_key.is_empty() {
        tracing::warn!("OPENAI_API_KEY is not set — DALL-E, embeddings and QA endpoints will return errors");
    }

    // ── Service + repository wiring ───────────────────────────────────────────
    let http = HttpService::new();

    let state = Arc::new(AppState {
        anthropic_key: anthropic_key.clone(),
        openai_key:    openai_key.clone(),

        ocr_service:    OcrService::new(http.clone()),
        claude_service: ClaudeService::new(http.clone()),
        dalle_service:  DalleService::new(http.clone()),
        qa_service:     QaService::new(http.clone()),

        repository: PgvectorRepository::new(pool, http),
    });

    // ── Router ────────────────────────────────────────────────────────────────
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "Mockupface backend — OK" }))

        // OCR
        .route("/api/ocr",             post(controllers::ocr_controller::handle))

        // RAG
        .route("/api/rag/search",      post(controllers::rag_controller::search))
        .route("/api/rag/store",       post(controllers::rag_controller::store))

        // Prompt generation
        .route("/api/prompts",         post(controllers::prompts_controller::handle))

        // Image generation
        .route("/api/generate-image",  post(controllers::images_controller::handle))

        // Quality assurance
        .route("/api/qa",              post(controllers::qa_controller::handle))

        // Full pipeline
        .route("/api/pipeline",        post(controllers::pipeline_controller::handle))

        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // ── Bind & serve ──────────────────────────────────────────────────────────
    let port = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.into());
    let addr = format!("{}:{}", DEFAULT_BIND_ADDR, port);

    tracing::info!("Mockupface listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
