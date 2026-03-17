use serde::Serialize;
use crate::common::enums::StageStatus;
use crate::models::mockup::MockupResult;

/// Execution record for a single pipeline stage.
#[derive(Debug, Serialize)]
pub struct PipelineStage {
    /// Stage name (matches STAGE_* constants).
    pub name:   String,
    /// Execution outcome.
    pub status: StageStatus,
    /// Human-readable detail — timing, counts, errors.
    pub detail: String,
}

impl PipelineStage {
    pub fn ok(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), status: StageStatus::Ok, detail: detail.into() }
    }

    pub fn skipped(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), status: StageStatus::Skipped, detail: detail.into() }
    }

    pub fn error(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { name: name.into(), status: StageStatus::Error, detail: detail.into() }
    }
}

/// Full pipeline execution response.
#[derive(Debug, Serialize)]
pub struct PipelineResponse {
    /// Ordered list of stage execution records.
    pub stages:   Vec<PipelineStage>,
    /// Generated mockups — one per condition slot.
    pub mockups:  Vec<MockupResult>,
    /// Number of past runs retrieved from pgvector RAG.
    pub rag_hits: usize,
}
