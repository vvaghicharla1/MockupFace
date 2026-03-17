use serde_json::json;

use crate::common::{
    constants::{GPT4O_MODEL, QA_MAX_TOKENS, QA_PASS_THRESHOLD},
    error::{AppError, AppResult},
};
use crate::models::mockup::QaResult;
use crate::services::http_service::HttpService;

fn clean_json(raw: &str) -> &str {
    raw.trim()
       .trim_start_matches("```json")
       .trim_start_matches("```")
       .trim_end_matches("```")
       .trim()
}

/// Scores generated mockup images using GPT-4o Vision.
#[derive(Clone)]
pub struct QaService {
    http: HttpService,
}

impl QaService {
    pub fn new(http: HttpService) -> Self {
        Self { http }
    }

    /// Score a single mockup image against platform-specific criteria.
    ///
    /// Returns a `QaResult` with a normalised score (0.0–1.0) and
    /// a pass/fail decision based on `QA_PASS_THRESHOLD`.
    pub async fn score(
        &self,
        image_url:       &str,
        original_prompt: &str,
        platform:        &str,
        condition_label: &str,
        api_key:         &str,
    ) -> AppResult<QaResult> {

        let criteria = match platform {
            "etsy"   => "artisanal quality, warm/lifestyle feel, gifting appeal, handmade aesthetic",
            "amazon" => "clear product visibility, clean background, professional commercial look",
            _        => "commercial e-commerce photography quality",
        };

        let review_prompt = format!(
            "Review this {platform} product mockup (condition: {condition_label}).\n\
             Evaluation criteria: {criteria}\n\n\
             Return ONLY valid JSON — no markdown, no explanation:\n\
             {{\"score\":0.85,\"passed\":true,\"feedback\":\"one-sentence assessment\",\"issues\":[]}}\n\n\
             Original DALL-E 3 prompt used: {original_prompt}"
        );

        let body = json!({
            "model":      GPT4O_MODEL,
            "max_tokens": QA_MAX_TOKENS,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "image_url", "image_url": { "url": image_url } },
                    { "type": "text",      "text": review_prompt }
                ]
            }]
        });

        let data = self.http
            .post_openai("/chat/completions", api_key, &body)
            .await?;

        let raw = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or(r#"{"score":0.5,"passed":true,"feedback":"QA response unavailable","issues":[]}"#);

        let mut result: QaResult = serde_json::from_str(clean_json(raw))
            .map_err(|e| AppError::ParseError(format!("GPT-4o QA response parse failed: {}", e)))?;

        // Enforce threshold — don't trust the model's own pass/fail decision.
        result.passed = result.score >= QA_PASS_THRESHOLD;

        tracing::info!(
            platform = platform,
            condition = condition_label,
            score = result.score,
            passed = result.passed,
            "QaService: image scored"
        );

        Ok(result)
    }
}
