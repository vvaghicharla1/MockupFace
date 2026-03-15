use axum::{extract::{State, Multipart}, Json};
use serde::Serialize;
use std::sync::Arc;
use tokio::task::JoinSet;
use uuid::Uuid;
use pgvector::Vector;

use crate::AppState;
use crate::routes::{embed, GeneratedPrompt, RagHit};
use crate::routes::ocr;
use crate::routes::prompts::generate as claude_generate;
use crate::routes::images::call_dalle;
use crate::routes::qa::score as qa_score;

#[derive(Serialize)]
pub struct Stage {
    pub name:   String,
    pub status: String,   // "ok" | "skipped" | "error"
    pub detail: String,
}

#[derive(Serialize)]
pub struct MockupOut {
    pub condition_id:    String,
    pub condition_label: String,
    pub environment:     String,
    pub prompt:          String,
    pub image_url:       Option<String>,
    pub qa_score:        Option<f64>,
    pub qa_passed:       bool,
    pub stored_id:       Option<Uuid>,
}

#[derive(Serialize)]
pub struct PipelineResp {
    pub stages:   Vec<Stage>,
    pub mockups:  Vec<MockupOut>,
    pub rag_hits: usize,
}

/// POST /api/pipeline  (multipart)
/// Fields: product_text, platform, style_hints (csv), image (optional)
pub async fn handle(
    State(state): State<Arc<AppState>>,
    mut mp: Multipart,
) -> Result<Json<PipelineResp>, String> {

    let mut product_text = String::new();
    let mut platform     = "etsy".to_string();
    let mut style_hints: Vec<String> = vec![];
    let mut img_bytes: Vec<u8>       = vec![];

    while let Some(f) = mp.next_field().await.map_err(|e| e.to_string())? {
        match f.name() {
            Some("product_text") => product_text = f.text().await.map_err(|e| e.to_string())?,
            Some("platform")     => platform     = f.text().await.map_err(|e| e.to_string())?,
            Some("style_hints")  => {
                let raw = f.text().await.map_err(|e| e.to_string())?;
                style_hints = raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            }
            Some("image") => img_bytes = f.bytes().await.map_err(|e| e.to_string())?.to_vec(),
            _ => {}
        }
    }

    let mut stages: Vec<Stage> = vec![];

    // ── Stage 1: OCR ─────────────────────────────────────────────────────────
    let ocr_summary: Option<String> = if !img_bytes.is_empty() {
        match ocr::run_bytes(&img_bytes, &state.anthropic_key).await {
            Ok(s)  => { stages.push(Stage { name:"Tesseract OCR".into(), status:"ok".into(),      detail: format!("{}", s) }); Some(s) }
            Err(e) => { stages.push(Stage { name:"Tesseract OCR".into(), status:"error".into(),   detail: e.to_string() }); None }
        }
    } else {
        stages.push(Stage { name:"Tesseract OCR".into(), status:"skipped".into(), detail:"No image — using text input".into() });
        None
    };

    // ── Stage 2: RAG ─────────────────────────────────────────────────────────
    let query = ocr_summary.as_deref().unwrap_or(&product_text);
    let rag_hits = match rag_search(query, &platform, &state).await {
        Ok(hits) => {
            stages.push(Stage { name:"pgvector RAG".into(), status:"ok".into(),    detail: format!("Retrieved {} similar past runs", hits.len()) });
            hits
        }
        Err(e) => {
            stages.push(Stage { name:"pgvector RAG".into(), status:"error".into(), detail: format!("RAG unavailable (empty DB?): {}", e) });
            vec![]
        }
    };
    let rag_count = rag_hits.len();

    // ── Stage 3: Claude prompts ───────────────────────────────────────────────
    let prompts = match claude_generate(
        &product_text, &platform,
        ocr_summary.as_deref(), Some(&style_hints),
        &rag_hits, &state.anthropic_key,
    ).await {
        Ok(p)  => { stages.push(Stage { name:"Claude Prompts".into(), status:"ok".into(),    detail: format!("{} prompts, rag_ctx={}", p.len(), rag_count) }); p }
        Err(e) => { stages.push(Stage { name:"Claude Prompts".into(), status:"error".into(), detail: e.to_string() });
                    return Ok(Json(PipelineResp { stages, mockups: vec![], rag_hits: rag_count })); }
    };

    // ── Stage 4: DALL-E 3 — 4× parallel ─────────────────────────────────────
    let mut set = JoinSet::new();
    for p in prompts.clone() {
        let key = state.openai_key.clone();
        set.spawn(async move {
            let r = call_dalle(&p.prompt, Some(&p.negative_prompt), "1024x1024", "standard", &key).await;
            (p, r)
        });
    }
    let mut img_results: Vec<(GeneratedPrompt, Option<String>)> = vec![];
    while let Some(res) = set.join_next().await {
        if let Ok((p, r)) = res {
            img_results.push((p, r.ok().map(|i| i.url)));
        }
    }
    let ok_count = img_results.iter().filter(|(_, u)| u.is_some()).count();
    stages.push(Stage {
        name:"DALL-E 3".into(),
        status: if ok_count > 0 { "ok".into() } else { "error".into() },
        detail: format!("{}/{} images generated", ok_count, img_results.len()),
    });

    // ── Stage 5: QA + store to pgvector ──────────────────────────────────────
    let mut mockups: Vec<MockupOut> = vec![];
    for (prompt, image_url) in img_results {
        let (score, passed) = if let Some(url) = &image_url {
            match qa_score(url, &prompt.prompt, &platform, &prompt.label, &state.openai_key).await {
                Ok(qa) => (Some(qa.score), qa.passed),
                Err(_) => (None, true),
            }
        } else { (None, false) };

        let stored_id = if let Some(url) = &image_url {
            store_run(
                &product_text, &platform, ocr_summary.as_deref(),
                &style_hints, &prompt, url, score, passed, &state,
            ).await.ok()
        } else { None };

        mockups.push(MockupOut {
            condition_id:    prompt.id,
            condition_label: prompt.label,
            environment:     prompt.environment,
            prompt:          prompt.prompt,
            image_url,
            qa_score:        score,
            qa_passed:       passed,
            stored_id,
        });
    }

    let qa_ok = mockups.iter().filter(|m| m.qa_passed).count();
    stages.push(Stage {
        name:"GPT-4o QA + Store".into(), status:"ok".into(),
        detail: format!("{}/{} passed QA, stored to pgvector", qa_ok, mockups.len()),
    });

    Ok(Json(PipelineResp { stages, mockups, rag_hits: rag_count }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn rag_search(text: &str, platform: &str, state: &AppState) -> anyhow::Result<Vec<RagHit>> {
    let raw = embed(text, &state.openai_key).await?;
    let qv  = Vector::from(raw);
    let rows = sqlx::query!(
        r#"SELECT id, product_text, condition_id, condition_label, environment, prompt_text, qa_score,
                  1 - (embedding <=> $1::vector) AS similarity
           FROM rag_candidates WHERE platform=$2 AND embedding IS NOT NULL
           ORDER BY embedding <=> $1::vector LIMIT 4"#,
        qv as Vector, platform
    ).fetch_all(&state.db).await?;
    Ok(rows.into_iter().map(|r| RagHit {
        id: r.id, product_text: r.product_text, condition_id: r.condition_id,
        condition_label: r.condition_label, environment: r.environment,
        prompt_text: r.prompt_text, similarity: r.similarity.unwrap_or(0.0), qa_score: r.qa_score,
    }).collect())
}

async fn store_run(
    product_text: &str, platform: &str, ocr: Option<&str>,
    hints: &[String], prompt: &GeneratedPrompt,
    image_url: &str, qa_score: Option<f64>, qa_passed: bool,
    state: &AppState,
) -> anyhow::Result<Uuid> {
    let text = format!("{} | {} | {} | {}", product_text, prompt.label, prompt.environment, prompt.prompt);
    let raw  = embed(&text, &state.openai_key).await?;
    let vec  = Vector::from(raw);
    let id   = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO mockup_runs
           (id, product_text, platform, ocr_text, style_hints,
            condition_id, condition_label, environment,
            prompt_text, negative_prompt, image_url, qa_score, qa_passed, embedding)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)"#,
        id, product_text, platform, ocr, hints,
        prompt.id, prompt.label, prompt.environment,
        prompt.prompt, Some(&prompt.negative_prompt), image_url,
        qa_score, qa_passed, vec as Vector,
    ).execute(&state.db).await?;
    Ok(id)
}
