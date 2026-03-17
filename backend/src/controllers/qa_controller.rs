use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::common::error::{AppError, AppResult};
use crate::models::mockup::QaResult;
use crate::AppState;

#[derive(Deserialize)]
pub struct QaRequest {
    pub image_url:       String,
    pub original_prompt: String,
    pub platform:        String,
    pub condition_label: String,
}

#[derive(Serialize)]
pub struct QaResponse {
    pub score:    f64,
    pub passed:   bool,
    pub feedback: String,
    pub issues:   Vec<String>,
}

impl From<QaResult> for QaResponse {
    fn from(r: QaResult) -> Self {
        Self {
            score:    r.score,
            passed:   r.passed,
            feedback: r.feedback,
            issues:   r.issues,
        }
    }
}

/// POST /api/qa
///
/// Scores a generated mockup image using GPT-4o Vision.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QaRequest>,
) -> AppResult<Json<QaResponse>> {

    if req.image_url.trim().is_empty() {
        return Err(AppError::MissingField("image_url".into()));
    }

    let result = state
        .qa_service
        .score(
            &req.image_url,
            &req.original_prompt,
            &req.platform,
            &req.condition_label,
            &state.openai_key,
        )
        .await?;

    Ok(Json(QaResponse::from(result)))
}
