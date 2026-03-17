use axum::{extract::{State, Multipart}, Json};
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::common::{
    constants::{STAGE_OCR, STAGE_RAG, STAGE_PROMPTS, STAGE_DALLE, STAGE_QA},
    error::{AppError, AppResult},
};
use crate::models::{
    mockup::{GeneratedPrompt, MockupResult},
    pipeline::{PipelineResponse, PipelineStage},
};
use crate::AppState;

/// POST /api/pipeline
///
/// Multipart fields:
/// - `image`        — product image (optional; triggers OCR when present)
/// - `product_text` — fallback description when no image is provided
/// - `platform`     — "etsy" | "amazon"
/// - `style_hints`  — comma-separated style keywords (optional)
///
/// Executes all 5 pipeline stages in order:
///   1. Tesseract OCR
///   2. pgvector RAG retrieval
///   3. Claude prompt generation
///   4. DALL-E 3 image generation (4× parallel)
///   5. GPT-4o Vision QA + pgvector store
pub async fn handle(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<Json<PipelineResponse>> {

    let mut product_text = String::new();
    let mut platform     = "etsy".to_string();
    let mut style_hints: Vec<String> = Vec::new();
    let mut img_bytes:   Vec<u8>     = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart read error: {}", e)))?
    {
        match field.name() {
            Some("product_text") => {
                product_text = field.text().await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            Some("platform") => {
                platform = field.text().await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
            }
            Some("style_hints") => {
                let raw = field.text().await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                style_hints = raw
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            Some("image") => {
                img_bytes = field.bytes().await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read image bytes: {}", e)))?
                    .to_vec();
            }
            _ => {}
        }
    }

    if product_text.trim().is_empty() && img_bytes.is_empty() {
        return Err(AppError::BadRequest(
            "Either product_text or an image must be provided".into()
        ));
    }

    let mut stages: Vec<PipelineStage> = Vec::new();

    // ── Stage 1: Tesseract OCR ────────────────────────────────────────────────
    let ocr_summary: Option<String> = if !img_bytes.is_empty() {
        match state.ocr_service.summarize(&img_bytes, &state.anthropic_key).await {
            Ok(summary) => {
                tracing::info!(stage = STAGE_OCR, "OCR extraction successful");
                stages.push(PipelineStage::ok(STAGE_OCR, format!("Extracted: {}", &summary[..summary.len().min(120)])));
                Some(summary)
            }
            Err(e) => {
                tracing::warn!(stage = STAGE_OCR, error = %e, "OCR extraction failed — continuing without");
                stages.push(PipelineStage::error(STAGE_OCR, e.to_string()));
                None
            }
        }
    } else {
        stages.push(PipelineStage::skipped(STAGE_OCR, "No image provided — using product_text directly"));
        None
    };

    // ── Stage 2: pgvector RAG retrieval ──────────────────────────────────────
    let query_text = ocr_summary.as_deref().unwrap_or(&product_text);
    let rag_hits = match state
        .repository
        .search_similar(query_text, &platform, None, &state.openai_key)
        .await
    {
        Ok(hits) => {
            tracing::info!(stage = STAGE_RAG, hits = hits.len(), "RAG retrieval successful");
            stages.push(PipelineStage::ok(
                STAGE_RAG,
                format!("Retrieved {} similar past runs from pgvector", hits.len()),
            ));
            hits
        }
        Err(e) => {
            tracing::warn!(stage = STAGE_RAG, error = %e, "RAG retrieval failed — proceeding without context");
            stages.push(PipelineStage::error(STAGE_RAG, format!("RAG unavailable: {}", e)));
            vec![]
        }
    };
    let rag_count = rag_hits.len();

    // ── Stage 3: Claude prompt generation ────────────────────────────────────
    let prompts = match state
        .claude_service
        .generate_prompts(
            &product_text,
            &platform,
            ocr_summary.as_deref(),
            Some(&style_hints),
            &rag_hits,
            &state.anthropic_key,
        )
        .await
    {
        Ok(p) => {
            tracing::info!(stage = STAGE_PROMPTS, count = p.len(), "Prompt generation successful");
            stages.push(PipelineStage::ok(
                STAGE_PROMPTS,
                format!("{} prompts generated (RAG context: {} prior runs)", p.len(), rag_count),
            ));
            p
        }
        Err(e) => {
            tracing::error!(stage = STAGE_PROMPTS, error = %e, "Prompt generation failed");
            stages.push(PipelineStage::error(STAGE_PROMPTS, e.to_string()));
            return Ok(Json(PipelineResponse { stages, mockups: vec![], rag_hits: rag_count }));
        }
    };

    // ── Stage 4: DALL-E 3 — 4× parallel ──────────────────────────────────────
    let mut join_set = JoinSet::new();
    for prompt in prompts.clone() {
        let dalle   = state.dalle_service.clone();
        let api_key = state.openai_key.clone();
        join_set.spawn(async move {
            let result = dalle
                .generate(&prompt.prompt, Some(&prompt.negative_prompt), None, None, &api_key)
                .await;
            (prompt, result)
        });
    }

    let mut image_results: Vec<(GeneratedPrompt, Option<String>)> = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok((prompt, Ok(img)))  => image_results.push((prompt, Some(img.url))),
            Ok((prompt, Err(e)))   => {
                tracing::warn!(condition = %prompt.id, error = %e, "DALL-E generation failed for condition");
                image_results.push((prompt, None));
            }
            Err(e) => tracing::error!(error = %e, "JoinSet task panicked"),
        }
    }

    let success_count = image_results.iter().filter(|(_, u)| u.is_some()).count();
    tracing::info!(stage = STAGE_DALLE, success = success_count, total = image_results.len(), "Image generation complete");
    stages.push(PipelineStage::ok(
        STAGE_DALLE,
        format!("{}/{} images generated successfully", success_count, image_results.len()),
    ));

    // ── Stage 5: GPT-4o QA + pgvector store ──────────────────────────────────
    let mut mockups: Vec<MockupResult> = Vec::new();

    for (prompt, image_url) in image_results {
        let (qa_score, qa_passed) = match &image_url {
            Some(url) => {
                match state
                    .qa_service
                    .score(url, &prompt.prompt, &platform, &prompt.label, &state.openai_key)
                    .await
                {
                    Ok(qa)  => (Some(qa.score), qa.passed),
                    Err(e)  => {
                        tracing::warn!(condition = %prompt.id, error = %e, "QA scoring failed — defaulting to pass");
                        (None, true)
                    }
                }
            }
            None => (None, false),
        };

        let stored_id = match &image_url {
            Some(url) if qa_passed => {
                match state
                    .repository
                    .store_run(
                        &product_text,
                        &platform,
                        ocr_summary.as_deref(),
                        &style_hints,
                        &prompt,
                        url,
                        qa_score,
                        qa_passed,
                        &state.openai_key,
                    )
                    .await
                {
                    Ok(id)  => Some(id),
                    Err(e)  => {
                        tracing::warn!(condition = %prompt.id, error = %e, "Failed to persist run to pgvector");
                        None
                    }
                }
            }
            _ => None,
        };

        mockups.push(MockupResult {
            condition_id:    prompt.id,
            condition_label: prompt.label,
            environment:     prompt.environment,
            prompt:          prompt.prompt,
            image_url,
            qa_score,
            qa_passed,
            stored_id,
        });
    }

    let qa_passed_count = mockups.iter().filter(|m| m.qa_passed).count();
    let stored_count    = mockups.iter().filter(|m| m.stored_id.is_some()).count();
    tracing::info!(
        stage   = STAGE_QA,
        passed  = qa_passed_count,
        stored  = stored_count,
        total   = mockups.len(),
        "QA and storage complete"
    );
    stages.push(PipelineStage::ok(
        STAGE_QA,
        format!(
            "{}/{} passed QA — {} runs stored to pgvector",
            qa_passed_count, mockups.len(), stored_count
        ),
    ));

    Ok(Json(PipelineResponse { stages, mockups, rag_hits: rag_count }))
}
