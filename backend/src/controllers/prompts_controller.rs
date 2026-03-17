use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::common::error::{AppError, AppResult};
use crate::models::mockup::{GeneratedPrompt, RagHit};
use crate::AppState;

#[derive(Deserialize)]
pub struct PromptsRequest {
    pub product_description: String,
    pub platform:            String,
    pub ocr_summary:         Option<String>,
    pub style_hints:         Option<Vec<String>>,
    /// Pre-fetched RAG context — callers may inject this or leave empty.
    pub rag_context:         Option<Vec<RagHit>>,
}

#[derive(Serialize)]
pub struct PromptsResponse {
    pub prompts:  Vec<GeneratedPrompt>,
    pub rag_used: usize,
}

/// POST /api/prompts
///
/// Generates 4 condition-specific DALL-E 3 prompts using Claude,
/// optionally enriched with pgvector RAG context.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PromptsRequest>,
) -> AppResult<Json<PromptsResponse>> {

    if req.product_description.trim().is_empty() {
        return Err(AppError::MissingField("product_description".into()));
    }

    let rag = req.rag_context.clone().unwrap_or_default();
    let rag_used = rag.len();

    let prompts = state
        .claude_service
        .generate_prompts(
            &req.product_description,
            &req.platform,
            req.ocr_summary.as_deref(),
            req.style_hints.as_deref(),
            &rag,
            &state.anthropic_key,
        )
        .await?;

    Ok(Json(PromptsResponse { prompts, rag_used }))
}
