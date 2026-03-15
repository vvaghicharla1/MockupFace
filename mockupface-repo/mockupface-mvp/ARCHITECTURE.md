# Architecture

A technical deep-dive into how Mockupface is built.

---

## System Overview

Mockupface is a two-process application:

- **Backend** — Rust/Axum HTTP server handling all AI API calls, OCR, database, and pipeline orchestration
- **Frontend** — React/Vite SPA that proxies `/api/*` requests to the backend in development, and talks directly to the backend in production

```
Browser (React)
     │
     │ /api/* (Vite proxy in dev, direct in prod)
     ▼
Rust/Axum Server :8080
     │
     ├── Tesseract OCR (subprocess)
     ├── PostgreSQL + pgvector
     ├── Anthropic API (Claude)
     └── OpenAI API (DALL-E 3, GPT-4o, Embeddings)
```

---

## Backend

### Framework choices

- **Axum** — ergonomic async Rust web framework built on top of Tokio and Tower. Chosen for its type-safe extractors and zero-cost middleware composition.
- **Tokio** — async runtime. The pipeline runs 4 DALL-E 3 calls in parallel using `JoinSet`, which is only possible with async.
- **sqlx** — compile-time verified SQL queries against PostgreSQL. No ORM overhead.
- **pgvector** — Rust crate providing `Vector` type that maps directly to PostgreSQL's `vector` column type.

### Route structure

Each route is a single async function in its own file:

```
main.rs         → router setup, AppState (db pool + API keys)
routes/
  mod.rs        → shared types (GeneratedPrompt, RagHit), embed() helper
  ocr.rs        → Tesseract subprocess + Claude structuring
  rag.rs        → pgvector cosine search + store
  prompts.rs    → Claude prompt generation
  images.rs     → DALL-E 3 call
  qa.rs         → GPT-4o Vision scoring
  pipeline.rs   → orchestrates all 5 stages
```

### State management

`AppState` is wrapped in `Arc` and shared across all handlers:

```rust
pub struct AppState {
    pub db:            sqlx::PgPool,
    pub anthropic_key: String,
    pub openai_key:    String,
}
```

API keys are read from environment at startup and never logged.

### Parallel image generation

The pipeline runs all 4 DALL-E 3 calls concurrently using Tokio's `JoinSet`:

```rust
let mut set = JoinSet::new();
for prompt in prompts {
    let key = state.openai_key.clone();
    set.spawn(async move {
        call_dalle(&prompt, &key).await
    });
}
while let Some(result) = set.join_next().await { ... }
```

This cuts image generation time from ~60s sequential to ~15s parallel.

---

## pgvector RAG

### Schema

```sql
CREATE TABLE mockup_runs (
    id          UUID PRIMARY KEY,
    product_text TEXT,
    platform    TEXT,
    condition_id TEXT,
    prompt_text TEXT,
    image_url   TEXT,
    qa_score    FLOAT,
    embedding   vector(1536)   -- text-embedding-3-small
);

CREATE INDEX ON mockup_runs
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);
```

### Embedding strategy

The embedding is built from a composite string combining product text, condition label, environment, and the generated prompt. This ensures retrieval matches on the full semantic context, not just the product name:

```
"{product_text} | {condition_label} | {environment} | {prompt_text}"
```

### Retrieval

Cosine similarity search filtered by platform:

```sql
SELECT *, 1 - (embedding <=> $1::vector) AS similarity
FROM rag_candidates
WHERE platform = $2 AND embedding IS NOT NULL
ORDER BY embedding <=> $1::vector
LIMIT 4
```

The `rag_candidates` view filters for `qa_score >= 0.7` and `user_rating >= 3`, ensuring only high-quality past runs are used as context.

---

## Frontend

### Two modes

The app supports two runtime modes, toggled in the header:

**Backend mode** (default): All pipeline stages run on the Rust server. The frontend submits a multipart form to `/api/pipeline` and receives the full result.

**Frontend mode** (fallback): Claude prompt generation and DALL-E 3 run directly from the browser using the user's API keys. OCR, RAG, and QA are skipped.

### Canvas renderer

When DALL-E 3 images aren't available (no OpenAI key, or frontend mode), the app renders placeholder mockup scenes using the HTML5 Canvas API. The canvas renderer in `canvasRenderer.js`:

1. Draws a gradient background using the AI-generated `bg_from`/`bg_to` colors
2. Renders a subtle grid texture
3. Composites the uploaded product image onto a rounded product shape
4. Applies a condition-specific tint overlay
5. Renders condition badge, mood tags, and environment label

### Key security

API keys are stored only in React `useState`. They are:
- Never written to `localStorage`, `sessionStorage`, or cookies
- Never logged or sent to the Mockupface backend (in frontend mode, calls go directly to OpenAI/Anthropic)
- Cleared automatically when the browser tab closes

---

## OCR Pipeline

Tesseract runs as a subprocess via `std::process::Command`:

```rust
Command::new("tesseract")
    .args([&in_path, &out_base, "--oem", "3", "--psm", "3"])
    .output()?
```

The raw OCR text is then passed to Claude with a structured extraction prompt, which returns JSON with:
- `detected_text` — array of text strings found on the product
- `font_hint` — apparent font style
- `color_hint` — dominant color palette
- `product_type` — mug, tshirt, poster, etc.
- `summary` — 2-sentence description for DALL-E prompt generation

---

## AI Prompt Design

### Claude system prompt structure

The Claude prompt generation system prompt encodes:
1. Platform-specific photography guidelines (Etsy vs Amazon)
2. The 4 condition slot definitions with aesthetic guidance
3. Instructions to use RAG context from past runs
4. Strict JSON output format with all required fields

### RAG context injection

Past similar runs are formatted and injected as numbered examples:

```
Past run #1 (similarity 0.91, condition c1):
"Professional product photography, minimalist mug on marble..."
→ QA score: 0.87

Past run #2 (similarity 0.84, condition c2):
...
```

Claude uses these as reference for what produces high-scoring results on this platform.

---

## Database

### Connection pooling

sqlx manages a pool of PostgreSQL connections. The pool is initialized at startup and shared via `AppState`:

```rust
let pool = sqlx::PgPool::connect(&database_url).await?;
```

### Migrations

Schema is managed via a single `schema.sql` file. For production, this would be replaced with sqlx migrations (`sqlx migrate run`).
