use std::{io::Write, process::Command};
use tempfile::NamedTempFile;
use serde_json::json;

use crate::common::{
    constants::{
        CLAUDE_MODEL, CLAUDE_OCR_MAX_TOKENS,
        TESSERACT_OEM, TESSERACT_PSM,
    },
    error::{AppError, AppResult},
};
use crate::models::ocr::OcrAnalysis;
use crate::services::http_service::HttpService;

fn clean_json(raw: &str) -> &str {
    raw.trim()
       .trim_start_matches("```json")
       .trim_start_matches("```")
       .trim_end_matches("```")
       .trim()
}

/// Orchestrates Tesseract OCR followed by Claude-powered result structuring.
///
/// # Flow
/// 1. Write image bytes to a named temp file.
/// 2. Invoke `tesseract` as a subprocess.
/// 3. Send raw OCR text to Claude for structured extraction into `OcrAnalysis`.
#[derive(Clone)]
pub struct OcrService {
    http: HttpService,
}

impl OcrService {
    pub fn new(http: HttpService) -> Self {
        Self { http }
    }

    /// Run OCR on raw image bytes and return a structured `OcrAnalysis`.
    pub async fn analyze(&self, bytes: &[u8], ext: &str, api_key: &str) -> AppResult<OcrAnalysis> {
        let raw_text = self.run_tesseract(bytes, ext)?;

        tracing::info!(
            chars   = raw_text.len(),
            preview = &raw_text[..raw_text.len().min(80)],
            "OcrService: Tesseract extraction complete"
        );

        let analysis = self
            .structure_with_claude(&raw_text, api_key)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    error = %e,
                    "OcrService: Claude structuring failed — falling back to raw OCR text"
                );
                OcrAnalysis {
                    raw_text:      raw_text.clone(),
                    detected_text: vec![raw_text.trim().to_string()],
                    font_hint:     "unknown".into(),
                    color_hint:    "unknown".into(),
                    product_type:  "product".into(),
                    summary:       raw_text.trim().to_string(),
                }
            });

        Ok(analysis)
    }

    /// Convenience method used by `PipelineController` — returns design summary only.
    pub async fn summarize(&self, bytes: &[u8], api_key: &str) -> AppResult<String> {
        let analysis = self.analyze(bytes, "png", api_key).await?;
        Ok(analysis.summary)
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn run_tesseract(&self, bytes: &[u8], ext: &str) -> AppResult<String> {
        let mut tmp = NamedTempFile::with_suffix(&format!(".{}", ext))
            .map_err(|e| AppError::OcrFailure(format!("Temp file creation failed: {}", e)))?;

        tmp.write_all(bytes)
            .map_err(|e| AppError::OcrFailure(format!("Temp file write failed: {}", e)))?;

        let in_path  = tmp.path().to_str().unwrap().to_string();
        let out_base = format!("{}_ocr_out", in_path);

        let output = Command::new("tesseract")
            .args([&in_path, &out_base, "--oem", TESSERACT_OEM, "--psm", TESSERACT_PSM])
            .output()
            .map_err(|e| AppError::OcrFailure(
                format!("Tesseract binary not found or failed to launch: {}", e)
            ))?;

        if output.status.success() {
            std::fs::read_to_string(format!("{}.txt", out_base))
                .map_err(|e| AppError::OcrFailure(format!("Failed to read OCR output file: {}", e)))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(AppError::OcrFailure(format!("Tesseract exited with error: {}", stderr)))
        }
    }

    async fn structure_with_claude(&self, raw: &str, api_key: &str) -> AppResult<OcrAnalysis> {
        let prompt = format!(
            r#"Parse this Tesseract OCR output from a product image.
Return ONLY valid JSON (no markdown, no explanation):
{{
  "detected_text": ["array", "of", "text", "strings", "found"],
  "font_hint":     "script | serif | sans | handwritten",
  "color_hint":    "e.g. soft pink, sage green, gold accent",
  "product_type":  "mug | tshirt | poster | tote | candle | phone_case | pillow | print | other",
  "summary":       "2-sentence product design summary for DALL-E prompt generation"
}}

OCR text:
{}"#,
            raw
        );

        let body = json!({
            "model":      CLAUDE_MODEL,
            "max_tokens": CLAUDE_OCR_MAX_TOKENS,
            "messages":   [{ "role": "user", "content": prompt }]
        });

        let data = self.http.post_anthropic("/messages", api_key, &body).await?;

        let text = data["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AppError::ParseError("No text in Claude OCR response".into()))?;

        #[derive(serde::Deserialize)]
        struct Inner {
            detected_text: Vec<String>,
            font_hint:     String,
            color_hint:    String,
            product_type:  String,
            summary:       String,
        }

        let inner: Inner = serde_json::from_str(clean_json(text))
            .map_err(|e| AppError::ParseError(
                format!("Failed to parse Claude OCR response as JSON: {}", e)
            ))?;

        Ok(OcrAnalysis {
            raw_text:      raw.to_string(),
            detected_text: inner.detected_text,
            font_hint:     inner.font_hint,
            color_hint:    inner.color_hint,
            product_type:  inner.product_type,
            summary:       inner.summary,
        })
    }
}
