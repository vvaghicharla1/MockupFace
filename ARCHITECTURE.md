# Architecture

A technical deep-dive into Mockupface's layered backend design.

---

## Layer Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        HTTP Layer                            │
│              Axum + Tokio  (HttpService)                     │
├──────────────────────────────────────────────────────────────┤
│                      Controllers                             │
│   OcrController · RagController · PromptsController          │
│   ImagesController · QaController · PipelineController       │
├──────────────────────────────────────────────────────────────┤
│                       Services                               │
│   OcrService · ClaudeService · DalleService · QaService      │
├──────────────────────────────────────────────────────────────┤
│                      Repository                              │
│                  PgvectorRepository                          │
├──────────────────────────────────────────────────────────────┤
│                       Common                                 │
│        AppError · constants · enums · models                 │
└──────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
backend/src/
├── main.rs                         Entry point — wires all layers, starts Axum server
│
├── common/
│   ├── mod.rs
│   ├── constants.rs                All magic strings — model names, URLs, thresholds
│   ├── enums.rs                    Domain enumerations — Platform, ConditionSlot, etc.
│   └── error.rs                   AppError + AppResult — uniform error type
│
├── models/
│   ├── mod.rs
│   ├── mockup.rs                  GeneratedPrompt, RagHit, QaResult, MockupResult
│   ├── ocr.rs                     OcrAnalysis
│   └── pipeline.rs                PipelineStage, PipelineResponse
│
├── controllers/                   HTTP handlers — parse requests, call services, return responses
│   ├── mod.rs
│   ├── ocr_controller.rs          POST /api/ocr
│   ├── rag_controller.rs          POST /api/rag/search  POST /api/rag/store
│   ├── prompts_controller.rs      POST /api/prompts
│   ├── images_controller.rs       POST /api/generate-image
│   ├── qa_controller.rs           POST /api/qa
│   └── pipeline_controller.rs     POST /api/pipeline
│
├── services/                      Business logic — all AI and external API calls
│   ├── mod.rs
│   ├── http_service.rs            HttpService — shared Axum/reqwest HTTP client
│   ├── ocr_service.rs             OcrService — Tesseract subprocess + Claude structuring
│   ├── claude_service.rs          ClaudeService — Anthropic prompt generation
│   ├── dalle_service.rs           DalleService — OpenAI DALL-E 3 generation
│   └── qa_service.rs              QaService — GPT-4o Vision scoring
│
└── repository/
    ├── mod.rs
    └── pgvector_repository.rs     PgvectorRepository — embeddings + pgvector CRUD
```

---

## Common Layer

### AppError (`common/error.rs`)

Every handler and service returns `AppResult<T>` which is `Result<T, AppError>`.
`AppError` implements `IntoResponse` — Axum serialises it automatically into a
consistent JSON error body:

```json
{
  "error":   "ANTHROPIC_ERROR",
  "message": "Anthropic API error: model not found",
  "detail":  null
}
```

Error variants map to HTTP status codes:

| Variant                 | HTTP Status |
|-------------------------|-------------|
| `BadRequest`            | 400         |
| `MissingField`          | 400         |
| `NoImageProvided`       | 400         |
| `UnsupportedPlatform`   | 400         |
| `MissingApiKey`         | 401         |
| All others              | 500         |

### Constants (`common/constants.rs`)

Single source of truth for every magic string — model names, API base URLs,
default values, stage names, thresholds. No hardcoded strings anywhere else.

```rust
pub const CLAUDE_MODEL:     &str = "claude-sonnet-4-20250514";
pub const DALLE_MODEL:      &str = "dall-e-3";
pub const RAG_DEFAULT_TOP_K: i64 = 4;
pub const QA_PASS_THRESHOLD: f64 = 0.6;
```

### Enums (`common/enums.rs`)

All domain enumerations with behaviour attached:

- `Platform` — `Etsy | Amazon` — carries `photography_guidelines()` and `qa_criteria()`
- `ConditionSlot` — `C1–C4` — carries `label()`, `environment()`, `aesthetic_guidance()`
- `StageStatus` — `Ok | Skipped | Error`
- `ProductType` — `Mug | Tshirt | Poster | ...`
- `ImageSize` / `ImageQuality` — DALL-E 3 parameter enums

---

## Controllers Layer

Controllers are thin. Their only responsibilities are:

1. Parse and validate the incoming HTTP request
2. Call the appropriate service(s)
3. Map the result to an HTTP response

No business logic. No direct database access. No direct API calls.

```rust
// Example — PromptsController
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PromptsRequest>,
) -> AppResult<Json<PromptsResponse>> {

    if req.product_description.trim().is_empty() {
        return Err(AppError::MissingField("product_description".into()));
    }

    let prompts = state.claude_service
        .generate_prompts(...)
        .await?;

    Ok(Json(PromptsResponse { prompts, rag_used }))
}
```

---

## Services Layer

### HttpService (`services/http_service.rs`)

Central HTTP client used by all services — no raw `reqwest` calls elsewhere.
Handles:
- Connection pooling (`pool_max_idle_per_host = 10`)
- Header injection per vendor (Anthropic key, OpenAI Bearer token)
- Unified error extraction from API response bodies

```rust
// All services receive an HttpService and call through it
let data = self.http.post_anthropic("/messages", api_key, &body).await?;
let data = self.http.post_openai("/embeddings", api_key, &body).await?;
```

### OcrService

Runs Tesseract as a subprocess via `std::process::Command`, writes bytes
to a named temp file, reads the output `.txt` file, then calls Claude to
structure the raw text into an `OcrAnalysis`.

### ClaudeService

Builds the prompt generation system prompt (platform tips + condition slot
definitions + RAG context injection) and calls `HttpService.post_anthropic`.
Deserialises the JSON response into `Vec<GeneratedPrompt>`.

### DalleService

Single-responsibility: call DALL-E 3 for one image. Appends negative prompt
as an "Avoid:" clause (DALL-E 3 has no native negative parameter). Cloneable
so pipeline can fan-out 4 parallel calls via `JoinSet`.

### QaService

Sends image URL + review prompt to GPT-4o Vision. Overrides the model's own
`passed` field with a hard threshold check against `QA_PASS_THRESHOLD = 0.6`.

---

## Repository Layer

### PgvectorRepository (`repository/pgvector_repository.rs`)

All database operations in one place:

**`embed(text, api_key)`** — calls OpenAI `text-embedding-3-small`, returns
`Vec<f32>` (1536 dimensions).

**`search_similar(query, platform, top_k, api_key)`** — embeds the query,
runs a pgvector cosine similarity search against `rag_candidates` view
(filtered for `qa_score >= 0.7`), returns `Vec<RagHit>`.

**`store_run(...)`** — builds a composite embed string
`"{product} | {label} | {env} | {prompt}"`, embeds it, inserts the full
`mockup_runs` record with the vector.

```sql
-- Cosine similarity query
SELECT *, 1 - (embedding <=> $1::vector) AS similarity
FROM rag_candidates
WHERE platform = $2 AND embedding IS NOT NULL
ORDER BY embedding <=> $1::vector
LIMIT $3
```

The `IVFFlat` index (100 lists) keeps similarity search sub-millisecond
at scale.

---

## PipelineController — Full Orchestration

`PipelineController` is the only place that combines all layers. It:

1. Parses multipart form data
2. Calls `OcrService` if an image was uploaded
3. Calls `PgvectorRepository.search_similar` for RAG context
4. Calls `ClaudeService.generate_prompts`
5. Fans out 4× `DalleService.generate` calls via `tokio::task::JoinSet`
6. For each result calls `QaService.score`
7. Stores passing runs via `PgvectorRepository.store_run`
8. Returns `PipelineResponse` with stage log + mockup results

Each stage's outcome is captured in a `PipelineStage` record regardless of
success or failure — the pipeline continues through non-fatal errors.

---

## AppState — Dependency Injection

All services and the repository are instantiated once in `main.rs` and
shared via `Arc<AppState>`:

```rust
pub struct AppState {
    pub anthropic_key:  String,
    pub openai_key:     String,
    pub ocr_service:    OcrService,
    pub claude_service: ClaudeService,
    pub dalle_service:  DalleService,
    pub qa_service:     QaService,
    pub repository:     PgvectorRepository,
}
```

All services share a single `HttpService` instance (and its connection pool)
via `clone()` — `HttpService` wraps `reqwest::Client` which is cheaply
cloneable by design.

---

## Error Propagation

The `?` operator is used throughout — errors bubble up from repository →
service → controller and are converted to HTTP responses at the Axum boundary
via `AppError`'s `IntoResponse` implementation. No `.unwrap()` in production
paths.

```
PgvectorRepository::store_run  →  AppError::DatabaseError
      ↓ ?
PipelineController::handle     →  AppError::IntoResponse → HTTP 500 JSON
```
