use axum::{extract::{State, Multipart}, Json};
use serde::Serialize;
use std::{sync::Arc, io::Write, process::Command};
use tempfile::NamedTempFile;
use crate::AppState;
use crate::routes::clean_json;

#[derive(Serialize)]
pub struct OcrResponse {
    pub raw_text:     String,
    pub detected_text: Vec<String>,
    pub font_hint:    String,
    pub color_hint:   String,
    pub product_type: String,
    pub summary:      String,
}

/// Called directly by pipeline.rs — returns the design summary string
pub async fn run_bytes(bytes: &[u8], anthropic_key: &str) -> anyhow::Result<String> {
    let raw = tesseract_bytes(bytes, "png")?;
    let summary = structure(&raw, anthropic_key).await
        .map(|r| r.summary)
        .unwrap_or_else(|_| raw.trim().to_string());
    Ok(summary)
}

/// POST /api/ocr
/// Multipart: field "image" → PNG/JPG/PDF
pub async fn handle(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<OcrResponse>, String> {

    let mut bytes: Vec<u8> = vec![];
    let mut ext = "png".to_string();

    while let Some(f) = multipart.next_field().await.map_err(|e| e.to_string())? {
        if f.name() == Some("image") {
            if let Some(ct) = f.content_type() {
                if ct.contains("pdf")  { ext = "pdf".into(); }
                if ct.contains("jpeg") { ext = "jpg".into(); }
            }
            bytes = f.bytes().await.map_err(|e| e.to_string())?.to_vec();
        }
    }

    if bytes.is_empty() {
        return Err("No image field in multipart".into());
    }

    let raw_text = tesseract_bytes(&bytes, &ext)
        .map_err(|e| e.to_string())?;
    tracing::info!("OCR ({} chars): {:.60}...", raw_text.len(), raw_text);

    // Structure with Claude
    let structured = structure(&raw_text, &state.anthropic_key).await
        .unwrap_or(OcrResponse {
            raw_text:      raw_text.clone(),
            detected_text: vec![raw_text.trim().to_string()],
            font_hint:     "unknown".into(),
            color_hint:    "unknown".into(),
            product_type:  "product".into(),
            summary:       raw_text.trim().to_string(),
        });

    Ok(Json(structured))
}

fn tesseract_bytes(bytes: &[u8], ext: &str) -> anyhow::Result<String> {
    let mut tmp = NamedTempFile::with_suffix(&format!(".{}", ext))?;
    tmp.write_all(bytes)?;
    let in_path  = tmp.path().to_str().unwrap().to_string();
    let out_base = format!("{}_out", in_path);
    let out = Command::new("tesseract")
        .args([&in_path, &out_base, "--oem", "3", "--psm", "3"])
        .output()
        .map_err(|e| anyhow::anyhow!("Tesseract not found: {}", e))?;
    if out.status.success() {
        Ok(std::fs::read_to_string(format!("{}.txt", out_base)).unwrap_or_default())
    } else {
        Ok(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

async fn structure(raw: &str, key: &str) -> anyhow::Result<OcrResponse> {
    if key.is_empty() { anyhow::bail!("no key"); }
    let client = reqwest::Client::new();
    let prompt = format!(
        r#"Parse this Tesseract OCR output from a product image.
Return ONLY valid JSON (no markdown):
{{
  "detected_text": ["array","of","text","found"],
  "font_hint": "script/serif/sans/handwritten",
  "color_hint": "e.g. soft pink, sage green, gold",
  "product_type": "mug|tshirt|poster|tote|candle|phone_case|pillow|print|other",
  "summary": "2-sentence product design summary for DALL-E prompt generation"
}}
OCR text:
{raw}"#
    );
    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 400,
        "messages": [{"role":"user","content": prompt}]
    });
    let resp = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body).send().await?;
    let data: serde_json::Value = resp.json().await?;
    let text = data["content"][0]["text"].as_str().unwrap_or("{}");
    #[derive(serde::Deserialize)]
    struct Inner {
        detected_text: Vec<String>,
        font_hint: String,
        color_hint: String,
        product_type: String,
        summary: String,
    }
    let inner: Inner = serde_json::from_str(clean_json(text))?;
    Ok(OcrResponse {
        raw_text:      raw.to_string(),
        detected_text: inner.detected_text,
        font_hint:     inner.font_hint,
        color_hint:    inner.color_hint,
        product_type:  inner.product_type,
        summary:       inner.summary,
    })
}
