use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::common::error::{AppError, AppResult};
use crate::models::mockup::RagHit;
use crate::AppState;

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchRequest {
    pub product_text: String,
    pub platform:     String,
    pub top_k:        Option<i64>,
}

/// POST /api/rag/search
///
/// Embeds `product_text` and retrieves the most similar past runs
/// from the pgvector store.
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> AppResult<Json<Vec<RagHit>>> {

    if req.product_text.trim().is_empty() {
        return Err(AppError::MissingField("product_text".into()));
    }

    let hits = state
        .repository
        .search_similar(&req.product_text, &req.platform, req.top_k, &state.openai_key)
        .await?;

    Ok(Json(hits))
}

// ── Store ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StoreRequest {
    pub product_text:    String,
    pub platform:        String,
    pub ocr_text:        Option<String>,
    pub style_hints:     Option<Vec<String>>,
    pub condition_id:    String,
    pub condition_label: String,
    pub environment:     String,
    pub prompt_text:     String,
    pub negative_prompt: Option<String>,
    pub image_url:       String,
    pub qa_score:        Option<f64>,
    pub qa_passed:       Option<bool>,
}

#[derive(Serialize)]
pub struct StoreResponse {
    pub id:      Uuid,
    pub message: String,
}

/// POST /api/rag/store
///
/// Embeds a completed mockup run and persists it to pgvector for
/// future RAG retrieval.
pub async fn store(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StoreRequest>,
) -> AppResult<Json<StoreResponse>> {

    if req.product_text.trim().is_empty() {
        return Err(AppError::MissingField("product_text".into()));
    }
    if req.image_url.trim().is_empty() {
        return Err(AppError::MissingField("image_url".into()));
    }

    let hints = req.style_hints.unwrap_or_default();

    // Build a GeneratedPrompt from the flat request fields
    let prompt = crate::models::mockup::GeneratedPrompt {
        id:              req.condition_id,
        label:           req.condition_label,
        environment:     req.environment,
        prompt:          req.prompt_text,
        negative_prompt: req.negative_prompt.unwrap_or_default(),
        bg_from:         String::new(),
        bg_to:           String::new(),
        accent:          String::new(),
        mood:            vec![],
    };

    let id = state
        .repository
        .store_run(
            &req.product_text,
            &req.platform,
            req.ocr_text.as_deref(),
            &hints,
            &prompt,
            &req.image_url,
            req.qa_score,
            req.qa_passed.unwrap_or(true),
            &state.openai_key,
        )
        .await?;

    Ok(Json(StoreResponse {
        id,
        message: format!("Run {} stored successfully in pgvector", id),
    }))
}
