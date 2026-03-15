use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use crate::routes::{GeneratedPrompt, RagHit, clean_json};

#[derive(Deserialize)]
pub struct PromptsReq {
    pub product_description: String,
    pub platform:            String,
    pub ocr_summary:         Option<String>,
    pub style_hints:         Option<Vec<String>>,
    pub rag_context:         Option<Vec<RagHit>>,
}

#[derive(Serialize)]
pub struct PromptsResp {
    pub prompts:  Vec<GeneratedPrompt>,
    pub rag_used: usize,
}

/// POST /api/prompts
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PromptsReq>,
) -> Result<Json<PromptsResp>, String> {
    let rag = req.rag_context.clone().unwrap_or_default();
    let rag_used = rag.len();
    let prompts = generate(
        &req.product_description,
        &req.platform,
        req.ocr_summary.as_deref(),
        req.style_hints.as_deref(),
        &rag,
        &state.anthropic_key,
    ).await.map_err(|e| e.to_string())?;
    Ok(Json(PromptsResp { prompts, rag_used }))
}

pub async fn generate(
    product:  &str,
    platform: &str,
    ocr:      Option<&str>,
    hints:    Option<&[String]>,
    rag:      &[RagHit],
    key:      &str,
) -> anyhow::Result<Vec<GeneratedPrompt>> {

    let platform_tips = match platform {
        "etsy"   => "Etsy: handmade, artisanal, lifestyle, gifting aesthetics. Warm tones, story scenes.",
        "amazon" => "Amazon: clear product on clean/white bg for main image. Bold lifestyle for alternates.",
        _        => "E-commerce commercial photography.",
    };

    let rag_block = if rag.is_empty() {
        "No past runs yet — generate fresh, high-quality prompts.".to_string()
    } else {
        rag.iter().enumerate().map(|(i, r)| {
            format!("Past #{} (sim {:.2}, {}): \"{}\"  [QA: {:?}]",
                i+1, r.similarity, r.condition_id, r.prompt_text, r.qa_score)
        }).collect::<Vec<_>>().join("\n")
    };

    let ocr_block = ocr.map(|o| format!("\nOCR design info:\n{}", o)).unwrap_or_default();
    let hint_block = hints.map(|h| format!("\nStyle hints: {}", h.join(", "))).unwrap_or_default();

    let system = format!(r#"You are a creative director for {platform} product mockup photography.
{platform_tips}

Generate exactly 4 DALL-E 3 prompts — one per condition slot:
  c1 → Daily Use × White Studio        (clean, minimal, bright)
  c2 → Gift Presentation × Warm Lifestyle (cozy, warm tones, gift-ready)
  c3 → Professional × Dark Dramatic      (moody, editorial, dark bg)
  c4 → Outdoor / Adventure × Natural     (earthy, natural light, organic)

Use the RAG past-run context to improve quality. Incorporate what worked.

Return ONLY a JSON array of exactly 4 objects, no markdown:
[{{
  "id": "c1",
  "label": "Daily Use",
  "environment": "White Studio",
  "prompt": "detailed photorealistic DALL-E 3 prompt...",
  "negative_prompt": "blurry, watermark, text overlay, distorted, low quality",
  "bg_from": "#hexcolor",
  "bg_to": "#hexcolor",
  "accent": "#hexcolor",
  "mood": ["word1","word2","word3"]
}}, ...]"#);

    let user = format!("Product: {product}{ocr_block}{hint_block}\n\nRAG context:\n{rag_block}");

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 2000,
        "system": system,
        "messages": [{"role":"user","content": user}]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body).send().await?;

    let data: serde_json::Value = resp.json().await?;
    if let Some(e) = data.get("error") { anyhow::bail!("{}", e); }

    let raw = data["content"][0]["text"].as_str()
        .ok_or_else(|| anyhow::anyhow!("No text in response"))?;

    let prompts: Vec<GeneratedPrompt> = serde_json::from_str(clean_json(raw))
        .map_err(|e| anyhow::anyhow!("JSON parse: {} — raw: {:.200}", e, raw))?;

    tracing::info!("Claude → {} prompts (rag_ctx={})", prompts.len(), rag.len());
    Ok(prompts)
}
