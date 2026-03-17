use pgvector::Vector;
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::constants::{
    EMBEDDING_MODEL, EMBEDDING_DIMENSIONS,
    OPENAI_API_BASE, RAG_DEFAULT_TOP_K, RAG_MAX_TOP_K,
};
use crate::common::error::{AppError, AppResult};
use crate::models::mockup::{GeneratedPrompt, RagHit};
use crate::services::http_service::HttpService;

/// All pgvector and PostgreSQL persistence operations.
#[derive(Clone)]
pub struct PgvectorRepository {
    pool: PgPool,
    http: HttpService,
}

impl PgvectorRepository {
    pub fn new(pool: PgPool, http: HttpService) -> Self {
        Self { pool, http }
    }

    // ── Embedding ─────────────────────────────────────────────────────────────

    /// Embed `text` using OpenAI `text-embedding-3-small`.
    /// Returns a 1536-dimensional float vector.
    pub async fn embed(&self, text: &str, api_key: &str) -> AppResult<Vec<f32>> {
        if api_key.is_empty() {
            return Err(AppError::MissingApiKey("OPENAI_API_KEY required for embeddings".into()));
        }

        let body = serde_json::json!({
            "model":      EMBEDDING_MODEL,
            "input":      text,
            "dimensions": EMBEDDING_DIMENSIONS
        });

        let data = self.http
            .post_openai("/embeddings", api_key, &body)
            .await
            .map_err(|e| AppError::EmbeddingError(e.to_string()))?;

        let vector: Vec<f32> = data["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| AppError::EmbeddingError(
                format!("No embedding array in OpenAI response: {:?}", data)
            ))?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        tracing::debug!(
            dims = vector.len(),
            model = EMBEDDING_MODEL,
            "PgvectorRepository: embedding generated"
        );

        Ok(vector)
    }

    // ── Retrieval ─────────────────────────────────────────────────────────────

    /// Search for the most similar past mockup runs using cosine similarity.
    ///
    /// Only retrieves records from the `rag_candidates` view, which filters
    /// for `qa_score >= 0.7` and `user_rating >= 3`.
    pub async fn search_similar(
        &self,
        query_text: &str,
        platform:   &str,
        top_k:      Option<i64>,
        api_key:    &str,
    ) -> AppResult<Vec<RagHit>> {

        let raw   = self.embed(query_text, api_key).await?;
        let qv    = Vector::from(raw);
        let limit = top_k.unwrap_or(RAG_DEFAULT_TOP_K).min(RAG_MAX_TOP_K);

        let rows = sqlx::query!(
            r#"
            SELECT
                id,
                product_text,
                condition_id,
                condition_label,
                environment,
                prompt_text,
                qa_score,
                1 - (embedding <=> $1::vector) AS similarity
            FROM rag_candidates
            WHERE platform      = $2
              AND embedding IS NOT NULL
            ORDER BY embedding <=> $1::vector
            LIMIT $3
            "#,
            qv as Vector,
            platform,
            limit,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::RagError(format!("pgvector similarity search failed: {}", e)))?;

        let hits: Vec<RagHit> = rows
            .into_iter()
            .map(|r| RagHit {
                id:              r.id,
                product_text:    r.product_text,
                condition_id:    r.condition_id,
                condition_label: r.condition_label,
                environment:     r.environment,
                prompt_text:     r.prompt_text,
                similarity:      r.similarity.unwrap_or(0.0),
                qa_score:        r.qa_score,
            })
            .collect();

        tracing::info!(
            hits    = hits.len(),
            platform = platform,
            query   = &query_text[..query_text.len().min(50)],
            "PgvectorRepository: similarity search complete"
        );

        Ok(hits)
    }

    // ── Persistence ───────────────────────────────────────────────────────────

    /// Embed and persist a completed mockup run to the `mockup_runs` table.
    ///
    /// The composite embedding string combines product text, condition label,
    /// environment, and the generated prompt for richer retrieval semantics.
    pub async fn store_run(
        &self,
        product_text:    &str,
        platform:        &str,
        ocr_text:        Option<&str>,
        style_hints:     &[String],
        prompt:          &GeneratedPrompt,
        image_url:       &str,
        qa_score:        Option<f64>,
        qa_passed:       bool,
        api_key:         &str,
    ) -> AppResult<Uuid> {

        let embed_input = format!(
            "{} | {} | {} | {}",
            product_text, prompt.label, prompt.environment, prompt.prompt
        );

        let raw = self.embed(&embed_input, api_key).await?;
        let vec = Vector::from(raw);
        let id  = Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO mockup_runs (
                id,
                product_text,
                platform,
                ocr_text,
                style_hints,
                condition_id,
                condition_label,
                environment,
                prompt_text,
                negative_prompt,
                image_url,
                qa_score,
                qa_passed,
                embedding
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, $11,
                $12, $13, $14
            )
            "#,
            id,
            product_text,
            platform,
            ocr_text,
            style_hints,
            prompt.id,
            prompt.label,
            prompt.environment,
            prompt.prompt,
            Some(&prompt.negative_prompt),
            image_url,
            qa_score,
            qa_passed,
            vec as Vector,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

        tracing::info!(
            run_id   = %id,
            platform = platform,
            condition = &prompt.id,
            qa_passed = qa_passed,
            "PgvectorRepository: run stored"
        );

        Ok(id)
    }
}
