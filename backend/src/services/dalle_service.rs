use serde_json::json;

use crate::common::{
    constants::{DALLE_MODEL, DALLE_DEFAULT_SIZE, DALLE_DEFAULT_QUALITY},
    error::{AppError, AppResult},
};
use crate::services::http_service::HttpService;

/// Response from a single DALL-E 3 generation call.
#[derive(Debug)]
pub struct DalleResult {
    /// CDN URL of the generated image.
    pub url:            String,
    /// DALL-E 3 may rewrite the prompt for safety — surfaced here if present.
    pub revised_prompt: Option<String>,
}

/// Handles all DALL-E 3 image generation requests.
#[derive(Clone)]
pub struct DalleService {
    http: HttpService,
}

impl DalleService {
    pub fn new(http: HttpService) -> Self {
        Self { http }
    }

    /// Generate a single image from a prompt.
    ///
    /// `negative` is appended as an "Avoid:" clause since DALL-E 3 has
    /// no native negative-prompt parameter.
    pub async fn generate(
        &self,
        prompt:   &str,
        negative: Option<&str>,
        size:     Option<&str>,
        quality:  Option<&str>,
        api_key:  &str,
    ) -> AppResult<DalleResult> {

        let full_prompt = match negative {
            Some(neg) => format!("{}\n\nAvoid: {}", prompt, neg),
            None      => prompt.to_string(),
        };

        let body = json!({
            "model":           DALLE_MODEL,
            "prompt":          full_prompt,
            "n":               1,
            "size":            size.unwrap_or(DALLE_DEFAULT_SIZE),
            "quality":         quality.unwrap_or(DALLE_DEFAULT_QUALITY),
            "response_format": "url"
        });

        let data = self.http
            .post_openai("/images/generations", api_key, &body)
            .await?;

        let url = data["data"][0]["url"]
            .as_str()
            .ok_or_else(|| AppError::OpenAiError(
                "DALL-E 3 response contained no image URL".into()
            ))?
            .to_string();

        let revised_prompt = data["data"][0]["revised_prompt"]
            .as_str()
            .map(|s| s.to_string());

        tracing::info!(
            url = %url[..url.len().min(60)],
            revised = revised_prompt.is_some(),
            "DalleService: image generated"
        );

        Ok(DalleResult { url, revised_prompt })
    }
}
