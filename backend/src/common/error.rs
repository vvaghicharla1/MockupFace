use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

/// Canonical application error returned by every handler and service.
#[derive(Debug, Error)]
pub enum AppError {
    // ── External service errors ───────────────────────────────────────────────
    #[error("OCR engine failure: {0}")]
    OcrFailure(String),

    #[error("Anthropic API error: {0}")]
    AnthropicError(String),

    #[error("OpenAI API error: {0}")]
    OpenAiError(String),

    // ── Data / persistence errors ─────────────────────────────────────────────
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),

    #[error("RAG retrieval failed: {0}")]
    RagError(String),

    // ── Request / validation errors ───────────────────────────────────────────
    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    // ── Response / serialisation errors ──────────────────────────────────────
    #[error("Response parsing failed: {0}")]
    ParseError(String),

    #[error("No image provided in multipart upload")]
    NoImageProvided,

    // ── Configuration errors ──────────────────────────────────────────────────
    #[error("API key not configured: {0}")]
    MissingApiKey(String),

    // ── Catch-all ─────────────────────────────────────────────────────────────
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// JSON body returned to the client on every error response.
#[derive(Serialize)]
pub struct ErrorBody {
    pub error:   ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail:  Option<String>,
}

/// Machine-readable error codes for client-side handling.
#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    OcrFailure,
    AnthropicError,
    OpenAiError,
    DatabaseError,
    EmbeddingError,
    RagError,
    BadRequest,
    MissingField,
    UnsupportedPlatform,
    ParseError,
    NoImageProvided,
    MissingApiKey,
    InternalServerError,
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_)
            | AppError::MissingField(_)
            | AppError::UnsupportedPlatform(_)
            | AppError::NoImageProvided => StatusCode::BAD_REQUEST,

            AppError::MissingApiKey(_) => StatusCode::UNAUTHORIZED,

            AppError::OcrFailure(_)
            | AppError::AnthropicError(_)
            | AppError::OpenAiError(_)
            | AppError::EmbeddingError(_)
            | AppError::RagError(_)
            | AppError::ParseError(_)
            | AppError::DatabaseError(_)
            | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> ErrorCode {
        match self {
            AppError::OcrFailure(_)        => ErrorCode::OcrFailure,
            AppError::AnthropicError(_)    => ErrorCode::AnthropicError,
            AppError::OpenAiError(_)       => ErrorCode::OpenAiError,
            AppError::DatabaseError(_)     => ErrorCode::DatabaseError,
            AppError::EmbeddingError(_)    => ErrorCode::EmbeddingError,
            AppError::RagError(_)          => ErrorCode::RagError,
            AppError::BadRequest(_)        => ErrorCode::BadRequest,
            AppError::MissingField(_)      => ErrorCode::MissingField,
            AppError::UnsupportedPlatform(_) => ErrorCode::UnsupportedPlatform,
            AppError::ParseError(_)        => ErrorCode::ParseError,
            AppError::NoImageProvided      => ErrorCode::NoImageProvided,
            AppError::MissingApiKey(_)     => ErrorCode::MissingApiKey,
            AppError::Internal(_)          => ErrorCode::InternalServerError,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status  = self.status_code();
        let code    = self.error_code();
        let message = self.to_string();

        tracing::error!(error_code = ?code, "{}", message);

        let body = ErrorBody {
            error:   code,
            message,
            detail:  None,
        };

        (status, Json(body)).into_response()
    }
}

/// Convenience alias — all handlers return this.
pub type AppResult<T> = Result<T, AppError>;

/// Convert any anyhow error into an internal AppError.
impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Internal(format!("HTTP client error: {}", e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}
