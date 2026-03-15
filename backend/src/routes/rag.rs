use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use pgvector::Vector;
use crate::AppState;
use crate::routes::{embed, RagHit};

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchReq {
    pub product_text: String,
    pub platform:     String,
    pub top_k:        Option<i64>,
}

/// POST /api/rag/search
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchReq>,
) -> Result<Json<Vec<RagHit>>, String> {

    let raw = embed(&req.product_text, &state.openai_key)
        .await.map_err(|e| e.to_string())?;
    let qv  = Vector::from(raw);
    let k   = req.top_k.unwrap_or(4).min(10);

    let rows = sqlx::query!(
        r#"
        SELECT id, product_text, condition_id, condition_label,
               environment, prompt_text, qa_score,
               1 - (embedding <=> $1::vector) AS similarity
        FROM   rag_candidates
        WHERE  platform = $2 AND embedding IS NOT NULL
        ORDER  BY embedding <=> $1::vector
        LIMIT  $3
        "#,
        qv as Vector,
        req.platform,
        k,
    )
    .fetch_all(&state.db).await.map_err(|e| e.to_string())?;

    let hits: Vec<RagHit> = rows.into_iter().map(|r| RagHit {
        id:              r.id,
        product_text:    r.product_text,
        condition_id:    r.condition_id,
        condition_label: r.condition_label,
        environment:     r.environment,
        prompt_text:     r.prompt_text,
        similarity:      r.similarity.unwrap_or(0.0),
        qa_score:        r.qa_score,
    }).collect();

    tracing::info!("RAG search → {} hits for '{:.40}'", hits.len(), req.product_text);
    Ok(Json(hits))
}

// ── Store ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StoreReq {
    pub product_text:    String,
    pub platform:        String,
    pub ocr_text:        Option<String>,
    pub style_hints:     Option<Vec<String>>,
    pub condition_id:    String,
    pub condition_label: String,
    pub environment:     String,
    pub prompt_text:     String,
    pub negative_prompt: Option<String>,
    pub image_url:       Option<String>,
    pub qa_score:        Option<f64>,
    pub qa_passed:       Option<bool>,
}

#[derive(Serialize)]
pub struct StoreResp { pub id: Uuid }

/// POST /api/rag/store
pub async fn store(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StoreReq>,
) -> Result<Json<StoreResp>, String> {

    // Embed a rich composite string for better retrieval
    let embed_text = format!(
        "{} | {} | {} | {}",
        req.product_text, req.condition_label, req.environment, req.prompt_text
    );
    let raw = embed(&embed_text, &state.openai_key)
        .await.map_err(|e| e.to_string())?;
    let vec  = Vector::from(raw);
    let id   = Uuid::new_v4();
    let hints = req.style_hints.unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO mockup_runs
            (id, product_text, platform, ocr_text, style_hints,
             condition_id, condition_label, environment,
             prompt_text, negative_prompt, image_url,
             qa_score, qa_passed, embedding)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
        "#,
        id, req.product_text, req.platform,
        req.ocr_text, &hints,
        req.condition_id, req.condition_label, req.environment,
        req.prompt_text, req.negative_prompt, req.image_url,
        req.qa_score, req.qa_passed.unwrap_or(true),
        vec as Vector,
    )
    .execute(&state.db).await.map_err(|e| e.to_string())?;

    tracing::info!("RAG stored {} for '{:.40}'", id, req.product_text);
    Ok(Json(StoreResp { id }))
}
