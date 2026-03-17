/// HttpService encapsulates all outbound HTTP communication.
/// All service modules obtain a client through here — no raw reqwest
/// calls are scattered across the codebase.
use reqwest::{Client, Response};
use serde::Serialize;
use crate::common::error::{AppError, AppResult};

/// Thin wrapper around `reqwest::Client` providing uniform error handling
/// and header injection for each external API.
#[derive(Clone)]
pub struct HttpService {
    client: Client,
}

impl HttpService {
    /// Construct a new `HttpService` with connection pooling.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .expect("Failed to initialise HTTP client"),
        }
    }

    /// POST JSON to the Anthropic API.
    pub async fn post_anthropic<B: Serialize>(
        &self,
        path:    &str,
        api_key: &str,
        body:    &B,
    ) -> AppResult<serde_json::Value> {
        if api_key.is_empty() {
            return Err(AppError::MissingApiKey("ANTHROPIC_API_KEY".into()));
        }
        let url = format!("{}{}", crate::common::constants::ANTHROPIC_API_BASE, path);
        let resp = self.client
            .post(&url)
            .header("x-api-key",          api_key)
            .header("anthropic-version",  crate::common::constants::ANTHROPIC_VERSION)
            .header("content-type",       "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::AnthropicError(format!("Request failed: {}", e)))?;

        Self::parse_json(resp, "Anthropic").await
    }

    /// POST JSON to the OpenAI API.
    pub async fn post_openai<B: Serialize>(
        &self,
        path:    &str,
        api_key: &str,
        body:    &B,
    ) -> AppResult<serde_json::Value> {
        if api_key.is_empty() {
            return Err(AppError::MissingApiKey("OPENAI_API_KEY".into()));
        }
        let url = format!("{}{}", crate::common::constants::OPENAI_API_BASE, path);
        let resp = self.client
            .post(&url)
            .header("Authorization",  format!("Bearer {}", api_key))
            .header("content-type",   "application/json")
            .json(body)
            .send()
            .await
            .map_err(|e| AppError::OpenAiError(format!("Request failed: {}", e)))?;

        Self::parse_json(resp, "OpenAI").await
    }

    /// Deserialise a response body into `serde_json::Value`,
    /// propagating API-level error messages cleanly.
    async fn parse_json(resp: Response, vendor: &str) -> AppResult<serde_json::Value> {
        let status = resp.status();
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::ParseError(format!("{} response parse error: {}", vendor, e)))?;

        if !status.is_success() {
            let msg = data["error"]["message"]
                .as_str()
                .unwrap_or("Unknown API error")
                .to_string();
            return match vendor {
                "Anthropic" => Err(AppError::AnthropicError(msg)),
                _           => Err(AppError::OpenAiError(msg)),
            };
        }

        // Surface inline error objects (Anthropic pattern)
        if let Some(err) = data.get("error") {
            let msg = err["message"].as_str().unwrap_or(&err.to_string()).to_string();
            return match vendor {
                "Anthropic" => Err(AppError::AnthropicError(msg)),
                _           => Err(AppError::OpenAiError(msg)),
            };
        }

        Ok(data)
    }
}

impl Default for HttpService {
    fn default() -> Self { Self::new() }
}
