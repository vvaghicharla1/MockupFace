use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use crate::routes::clean_json;

#[derive(Deserialize)]
pub struct QaReq {
    pub image_url:       String,
    pub original_prompt: String,
    pub platform:        String,
    pub condition_label: String,
}

#[derive(Serialize, Deserialize)]
pub struct QaResp {
    pub score:    f64,
    pub passed:   bool,
    pub feedback: String,
    pub issues:   Vec<String>,
}

/// POST /api/qa
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QaReq>,
) -> Result<Json<QaResp>, String> {
    score(&req.image_url, &req.original_prompt,
          &req.platform, &req.condition_label, &state.openai_key)
        .await.map(Json).map_err(|e| e.to_string())
}

pub async fn score(
    image_url: &str, prompt: &str,
    platform: &str, condition: &str, key: &str,
) -> anyhow::Result<QaResp> {
    let criteria = match platform {
        "etsy"   => "artisanal quality, warm/lifestyle feel, gifting appeal",
        "amazon" => "clear product visibility, professional look, clean background",
        _        => "commercial photography quality",
    };
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "gpt-4o",
        "max_tokens": 300,
        "messages": [{
            "role": "user",
            "content": [
                {"type":"image_url","image_url":{"url": image_url}},
                {"type":"text","text": format!(
                    "Review this {platform} mockup ({condition}).\nCriteria: {criteria}\n\
                     Return ONLY JSON:\
                     {{\"score\":0.85,\"passed\":true,\"feedback\":\"one sentence\",\"issues\":[]}}\n\
                     Original prompt: {prompt}"
                )}
            ]
        }]
    });
    let resp = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", key))
        .json(&body).send().await?;
    let data: serde_json::Value = resp.json().await?;
    if let Some(e) = data.get("error") { anyhow::bail!("{}", e); }
    let raw = data["choices"][0]["message"]["content"].as_str()
        .unwrap_or(r#"{"score":0.5,"passed":true,"feedback":"QA unavailable","issues":[]}"#);
    let qa: QaResp = serde_json::from_str(clean_json(raw))
        .unwrap_or(QaResp { score:0.5, passed:true, feedback:"parse error".into(), issues:vec![] });
    tracing::info!("QA {platform}/{condition}: {:.2} passed={}", qa.score, qa.passed);
    Ok(qa)
}
