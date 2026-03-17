use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::common::error::AppResult;
use crate::AppState;

#[derive(Deserialize)]
pub struct GenerateImageRequest {
    pub prompt:          String,
    pub negative_prompt: Option<String>,
    pub size:            Option<String>,
    pub quality:         Option<String>,
}

#[derive(Serialize)]
pub struct GenerateImageResponse {
    pub url:            String,
    pub revised_prompt: Option<String>,
}

/// POST /api/generate-image
///
/// Generates a single image via DALL-E 3.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateImageRequest>,
) -> AppResult<Json<GenerateImageResponse>> {

    let result = state
        .dalle_service
        .generate(
            &req.prompt,
            req.negative_prompt.as_deref(),
            req.size.as_deref(),
            req.quality.as_deref(),
            &state.openai_key,
        )
        .await?;

    Ok(Json(GenerateImageResponse {
        url:            result.url,
        revised_prompt: result.revised_prompt,
    }))
}
