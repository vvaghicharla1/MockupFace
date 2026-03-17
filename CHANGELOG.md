# Changelog

All notable changes to Mockupface are documented here.

---

## [0.2.0] — 2025-03

### Backend — Full architectural refactor

**Layered architecture introduced:**
- `common/` — `AppError`, `constants`, `enums` (shared foundation)
- `models/` — pure domain structs with no logic (`GeneratedPrompt`, `RagHit`, `OcrAnalysis`, `PipelineStage`, etc.)
- `controllers/` — thin HTTP handlers (one per endpoint); no business logic
- `services/` — all AI and external API business logic
- `repository/` — all database access (`PgvectorRepository`)

**New modules:**
- `common/error.rs` — `AppError` enum with typed variants, machine-readable `ErrorCode`, consistent JSON error body, `IntoResponse` impl
- `common/constants.rs` — single source of truth for all model names, URLs, thresholds, stage names
- `common/enums.rs` — `Platform`, `ConditionSlot`, `StageStatus`, `ProductType`, `ImageSize`, `ImageQuality`
- `services/http_service.rs` — `HttpService` wrapping `reqwest::Client` with per-vendor header injection and unified error extraction
- `services/ocr_service.rs` — `OcrService` (Tesseract + Claude structuring)
- `services/claude_service.rs` — `ClaudeService` (Anthropic prompt generation)
- `services/dalle_service.rs` — `DalleService` (OpenAI DALL-E 3)
- `services/qa_service.rs` — `QaService` (GPT-4o Vision scoring)
- `repository/pgvector_repository.rs` — `PgvectorRepository` (embed + cosine search + store)

**Removed:** flat `routes/` module replaced entirely by the above layers.

**`AppState` now wires:** `OcrService`, `ClaudeService`, `DalleService`, `QaService`, `PgvectorRepository` — all injected via `Arc<AppState>`.

**Documentation updated:** `ARCHITECTURE.md` fully rewritten; `README.md` project structure updated.

---

## [0.1.0] — 2025-03

### Initial MVP release

### Initial MVP release

**Backend**
- Rust + Axum HTTP server with full CORS support
- Tesseract OCR route — extracts design info from product images
- Claude (Anthropic) — OCR structuring + prompt generation
- DALL-E 3 (OpenAI) — 4× parallel image generation via Tokio JoinSet
- GPT-4o Vision — quality scoring per generated image
- pgvector RAG — cosine similarity search + embedding store
- Full pipeline route `/api/pipeline` — all 5 stages in one request
- PostgreSQL schema with pgvector index (IVFFlat, cosine ops)

**Frontend**
- React 18 + Vite
- Image upload with drag & drop
- Platform selector (Etsy, Amazon)
- Style hint chips per platform
- 4 condition slot preview
- Live pipeline log with progress bar
- Canvas mockup renderer — draws condition-specific scenes
- Product image composited into canvas previews
- MockupCard — expand prompt, copy to clipboard, download PNG
- API key panel — stored in React state only, never persisted
- Two modes: backend (full pipeline) and frontend (Claude + DALL-E direct)
