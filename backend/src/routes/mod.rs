pub mod ocr;
pub mod rag;
pub mod prompts;
pub mod images;
pub mod qa;
pub mod pipeline;

// ── Shared types used across routes ──────────────────────────────────────────

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPrompt {
    pub id:              String,
    pub label:           String,
    pub environment:     String,
    pub prompt:          String,
    pub negative_prompt: String,
    pub bg_from:         String,
    pub bg_to:           String,
    pub accent:          String,
    pub mood:            Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagHit {
    pub id:              Uuid,
    pub product_text:    String,
    pub condition_id:    String,
    pub condition_label: String,
    pub environment:     String,
    pub prompt_text:     String,
    pub similarity:      f64,
    pub qa_score:        Option<f64>,
}

// ── Shared helper: OpenAI embedding ──────────────────────────────────────────

pub async fn embed(text: &str, api_key: &str) -> anyhow::Result<Vec<f32>> {
    if api_key.is_empty() {
        anyhow::bail!("OPENAI_API_KEY not set");
    }
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "text-embedding-3-small",
        "input": text,
        "dimensions": 1536
    });
    let resp = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send().await?;
    let data: serde_json::Value = resp.json().await?;
    let v: Vec<f32> = data["data"][0]["embedding"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No embedding: {:?}", data))?
        .iter()
        .filter_map(|x| x.as_f64().map(|f| f as f32))
        .collect();
    Ok(v)
}

// ── Shared helper: parse Claude/GPT JSON response ─────────────────────────

pub fn clean_json(raw: &str) -> &str {
    raw.trim()
       .trim_start_matches("```json")
       .trim_start_matches("```")
       .trim_end_matches("```")
       .trim()
}
