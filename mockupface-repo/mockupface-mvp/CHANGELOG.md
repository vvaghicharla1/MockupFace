# Changelog

All notable changes to Mockupface are documented here.

---

## [0.1.0] — 2025-03

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
