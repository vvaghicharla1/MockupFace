use axum::{extract::{State, Multipart}, Json};
use std::sync::Arc;

use crate::common::error::{AppError, AppResult};
use crate::models::ocr::OcrAnalysis;
use crate::AppState;

/// POST /api/ocr
///
/// Accepts a multipart upload with an `image` field (PNG / JPG / PDF).
/// Runs Tesseract OCR and returns a structured `OcrAnalysis`.
pub async fn handle(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<Json<OcrAnalysis>> {

    let mut bytes: Vec<u8> = Vec::new();
    let mut ext = "png".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart read error: {}", e)))?
    {
        if field.name() == Some("image") {
            if let Some(ct) = field.content_type() {
                if ct.contains("pdf")  { ext = "pdf".into(); }
                if ct.contains("jpeg") { ext = "jpg".into(); }
            }
            bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read image bytes: {}", e)))?
                .to_vec();
        }
    }

    if bytes.is_empty() {
        return Err(AppError::NoImageProvided);
    }

    let analysis = state
        .ocr_service
        .analyze(&bytes, &ext, &state.anthropic_key)
        .await?;

    Ok(Json(analysis))
}
