use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;

#[derive(Deserialize)]
pub struct ImageReq {
    pub prompt:          String,
    pub negative_prompt: Option<String>,
    pub size:            Option<String>,
    pub quality:         Option<String>,
}

#[derive(Serialize)]
pub struct ImageResp {
    pub url:            String,
    pub revised_prompt: Option<String>,
}

/// POST /api/generate-image
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImageReq>,
) -> Result<Json<ImageResp>, String> {
    call_dalle(&req.prompt, req.negative_prompt.as_deref(),
               req.size.as_deref().unwrap_or("1024x1024"),
               req.quality.as_deref().unwrap_or("standard"),
               &state.openai_key)
        .await.map(Json).map_err(|e| e.to_string())
}

pub async fn call_dalle(
    prompt: &str, negative: Option<&str>,
    size: &str, quality: &str, key: &str,
) -> anyhow::Result<ImageResp> {
    let full = match negative {
        Some(n) => format!("{}\n\nAvoid: {}", prompt, n),
        None    => prompt.to_string(),
    };
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "dall-e-3",
        "prompt": full,
        "n": 1,
        "size": size,
        "quality": quality,
        "response_format": "url"
    });
    let resp = client.post("https://api.openai.com/v1/images/generations")
        .header("Authorization", format!("Bearer {}", key))
        .json(&body).send().await?;
    let data: serde_json::Value = resp.json().await?;
    if let Some(e) = data.get("error") {
        anyhow::bail!("{}", e["message"].as_str().unwrap_or("DALL-E error"));
    }
    Ok(ImageResp {
        url: data["data"][0]["url"].as_str()
            .ok_or_else(|| anyhow::anyhow!("No URL in response"))?.to_string(),
        revised_prompt: data["data"][0]["revised_prompt"].as_str().map(|s| s.to_string()),
    })
}
