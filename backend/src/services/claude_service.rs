use serde_json::json;

use crate::common::{
    constants::{CLAUDE_MODEL, CLAUDE_PROMPT_MAX_TOKENS},
    error::{AppError, AppResult},
};
use crate::models::mockup::{GeneratedPrompt, RagHit};
use crate::services::http_service::HttpService;

fn clean_json(raw: &str) -> &str {
    raw.trim()
       .trim_start_matches("```json")
       .trim_start_matches("```")
       .trim_end_matches("```")
       .trim()
}

/// Handles all interactions with the Anthropic Claude API.
#[derive(Clone)]
pub struct ClaudeService {
    http: HttpService,
}

impl ClaudeService {
    pub fn new(http: HttpService) -> Self {
        Self { http }
    }

    /// Generate 4 condition-specific DALL-E 3 prompts from a product
    /// description, optional OCR summary, style hints, and RAG context.
    pub async fn generate_prompts(
        &self,
        product:  &str,
        platform: &str,
        ocr:      Option<&str>,
        hints:    Option<&[String]>,
        rag:      &[RagHit],
        api_key:  &str,
    ) -> AppResult<Vec<GeneratedPrompt>> {

        let platform_tips = match platform {
            "etsy"   => "Etsy: handmade, artisanal, lifestyle and gifting aesthetics. Warm tones, story scenes.",
            "amazon" => "Amazon: pure white background for main image. Bold lifestyle for alternate images.",
            other    => {
                tracing::warn!(platform = other, "Unrecognised platform — using generic photography guidelines");
                "E-commerce commercial photography."
            }
        };

        let rag_block = if rag.is_empty() {
            "No prior runs in the vector store — generate fresh high-quality prompts.".to_string()
        } else {
            rag.iter().enumerate().map(|(i, r)| {
                format!(
                    "Prior run #{} (cosine sim {:.2f}, slot {}): \"{}\"\n  QA score: {:?}",
                    i + 1, r.similarity, r.condition_id, r.prompt_text, r.qa_score
                )
            }).collect::<Vec<_>>().join("\n")
        };

        let ocr_block  = ocr.map(|o| format!("\nOCR design analysis:\n{}", o)).unwrap_or_default();
        let hint_block = hints.map(|h| format!("\nStyle hints: {}", h.join(", "))).unwrap_or_default();

        let system = format!(
            r#"You are a creative director generating DALL-E 3 product mockup prompts for {platform} listings.
{platform_tips}

Generate exactly 4 photorealistic DALL-E 3 prompts — one per condition slot:
  c1 → Daily Use × White Studio        (clean, minimal, bright, everyday)
  c2 → Gift Presentation × Warm Lifestyle (cosy, warm bokeh, gift-ready)
  c3 → Professional × Dark Dramatic      (moody, editorial, dark background)
  c4 → Outdoor / Adventure × Natural     (earthy, organic, natural light)

Leverage the pgvector RAG context below to improve prompt quality — incorporate patterns from high-scoring prior runs.

Return ONLY a valid JSON array of exactly 4 objects. No markdown, no preamble:
[{{
  "id":              "c1",
  "label":           "Daily Use",
  "environment":     "White Studio",
  "prompt":          "detailed DALL-E 3 prompt...",
  "negative_prompt": "blurry, watermark, text overlay, distorted, low quality",
  "bg_from":         "#c8a96e55",
  "bg_to":           "#080808",
  "accent":          "#c8a96e",
  "mood":            ["minimal", "bright", "everyday"]
}}, ...]"#
        );

        let user = format!(
            "Product: {product}{ocr_block}{hint_block}\n\npgvector RAG context:\n{rag_block}"
        );

        let body = json!({
            "model":      CLAUDE_MODEL,
            "max_tokens": CLAUDE_PROMPT_MAX_TOKENS,
            "system":     system,
            "messages":   [{ "role": "user", "content": user }]
        });

        let data = self.http.post_anthropic("/messages", api_key, &body).await?;

        let raw = data["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AppError::ParseError("No text content in Claude response".into()))?;

        let prompts: Vec<GeneratedPrompt> = serde_json::from_str(clean_json(raw))
            .map_err(|e| AppError::ParseError(
                format!("Failed to deserialise Claude prompt array: {} — raw: {:.200}", e, raw)
            ))?;

        tracing::info!(
            count = prompts.len(),
            rag_ctx = rag.len(),
            "ClaudeService: prompt generation complete"
        );

        Ok(prompts)
    }
}
