# Mockupface

> AI-powered product mockup generator for Etsy & Amazon listings.

Upload a product image → Claude Vision analyzes it → Generates 4 condition-specific DALL-E 3 mockups automatically.

![Pipeline](https://img.shields.io/badge/pipeline-OCR→RAG→Claude→DALL·E-F5C518?style=flat-square)
![Rust](https://img.shields.io/badge/backend-Rust%20%2B%20Axum-orange?style=flat-square)
![React](https://img.shields.io/badge/frontend-React%20%2B%20Vite-61DAFB?style=flat-square)
![pgvector](https://img.shields.io/badge/RAG-pgvector-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)

---

## What It Does

Mockupface automates the entire product mockup workflow for e-commerce sellers:

1. **Upload** your product image (mug, t-shirt, poster, candle, etc.)
2. **Tesseract OCR** extracts design text, fonts, and layout from the image
3. **Claude (Anthropic)** analyzes the product and generates 4 platform-optimized DALL-E 3 prompts — one per condition slot
4. **DALL-E 3 (OpenAI)** generates photorealistic mockup images in parallel
5. **GPT-4o Vision** scores each image for quality
6. **pgvector RAG** stores successful runs as vector embeddings — future runs get smarter by retrieving similar past prompts

The result: 4 professional mockup images per product, ready for Etsy or Amazon listings, in under a minute.

---

## Pipeline Architecture

```
[Product Image]
      │
      ▼
┌─────────────────┐
│  Tesseract OCR  │  Extract text, fonts, layout from image
└────────┬────────┘
         │ design_info
         ▼
┌─────────────────┐
│  OpenAI Embed   │  text-embedding-3-small → 1536-dim vector
└────────┬────────┘
         │ query_vector
         ▼
┌─────────────────┐
│  pgvector RAG   │  cosine similarity → top-4 past successful runs
│  (PostgreSQL)   │  retrieved as context
└────────┬────────┘
         │ rag_context
         ▼
┌─────────────────┐
│  Claude         │  Generates 4 condition-specific DALL-E prompts
│  (Anthropic)    │  using product info + RAG context
└────────┬────────┘
         │ 4 × prompts
         ▼
┌─────────────────┐
│  DALL-E 3       │  4 × parallel image generation (1024×1024)
│  (OpenAI)       │
└────────┬────────┘
         │ 4 × image URLs
         ▼
┌─────────────────┐
│  GPT-4o Vision  │  QA score each image (0.0–1.0)
└────────┬────────┘
         │ qa_results + embeddings
         ▼
┌─────────────────┐
│  pgvector Store │  Save successful runs → RAG improves over time
└─────────────────┘
```

---

## Condition Slots

Every product run generates 4 mockups across fixed condition slots:

| Slot | Usage              | Environment     | Aesthetic                     |
|------|--------------------|-----------------|-------------------------------|
| C1   | Daily Use          | White Studio    | Clean, minimal, bright        |
| C2   | Gift Presentation  | Warm Lifestyle  | Cozy, warm tones, gift-ready  |
| C3   | Professional       | Dark Dramatic   | Moody, editorial, premium     |
| C4   | Outdoor/Adventure  | Natural Outdoor | Earthy, organic, natural light|

---

## Tech Stack

| Layer        | Technology                                              |
|--------------|---------------------------------------------------------|
| Backend      | Rust, Axum, Tokio                                       |
| Frontend     | React 18, Vite                                          |
| Database     | PostgreSQL 15 + pgvector extension                      |
| OCR          | Tesseract OCR (subprocess)                              |
| AI — Vision  | Anthropic Claude (claude-sonnet) — image analysis       |
| AI — Prompts | Anthropic Claude (claude-sonnet) — prompt generation    |
| AI — Images  | OpenAI DALL-E 3 — mockup generation                     |
| AI — QA      | OpenAI GPT-4o Vision — quality scoring                  |
| AI — Embed   | OpenAI text-embedding-3-small — RAG vectors             |
| RAG          | pgvector — cosine similarity search                     |

---

## Project Structure

```
mockupface/
├── README.md
├── ARCHITECTURE.md                Technical deep-dive
├── CONTRIBUTING.md
├── CHANGELOG.md
├── LICENSE
├── .gitignore
│
├── backend/                       Rust + Axum API server
│   ├── Cargo.toml
│   ├── .env.example
│   ├── schema.sql                 PostgreSQL + pgvector schema
│   └── src/
│       ├── main.rs                Entry point — wires all layers, starts server
│       │
│       ├── common/                Shared foundation
│       │   ├── constants.rs       Model names, URLs, thresholds, stage names
│       │   ├── enums.rs           Platform, ConditionSlot, StageStatus, ProductType
│       │   └── error.rs           AppError + AppResult — uniform error type
│       │
│       ├── models/                Domain structs (no logic)
│       │   ├── mockup.rs          GeneratedPrompt, RagHit, QaResult, MockupResult
│       │   ├── ocr.rs             OcrAnalysis
│       │   └── pipeline.rs        PipelineStage, PipelineResponse
│       │
│       ├── controllers/           HTTP layer — parse requests, delegate, respond
│       │   ├── ocr_controller.rs          POST /api/ocr
│       │   ├── rag_controller.rs          POST /api/rag/search|store
│       │   ├── prompts_controller.rs      POST /api/prompts
│       │   ├── images_controller.rs       POST /api/generate-image
│       │   ├── qa_controller.rs           POST /api/qa
│       │   └── pipeline_controller.rs     POST /api/pipeline
│       │
│       ├── services/              Business logic — all AI and external API calls
│       │   ├── http_service.rs    HttpService — shared HTTP client (Axum/Tokio)
│       │   ├── ocr_service.rs     OcrService — Tesseract + Claude structuring
│       │   ├── claude_service.rs  ClaudeService — Anthropic prompt generation
│       │   ├── dalle_service.rs   DalleService — OpenAI DALL-E 3
│       │   └── qa_service.rs      QaService — GPT-4o Vision scoring
│       │
│       └── repository/            Data access layer
│           └── pgvector_repository.rs  PgvectorRepository — embed + search + store
│
└── frontend/                      React + Vite
    ├── index.html
    ├── package.json
    ├── vite.config.js             Proxies /api → localhost:8080
    └── src/
        ├── main.jsx
        ├── App.jsx                Main app, two modes (backend/frontend)
        ├── constants.js           Platform configs, condition slots
        ├── canvasRenderer.js      Canvas mockup preview drawing
        └── components/
            ├── DropZone.jsx       Image upload + drag & drop
            ├── MockupCard.jsx     Result card with canvas/real image
            ├── PipelineLog.jsx    Live stage tracker UI
            └── KeyPanel.jsx       API key management drawer
```

---

## Prerequisites

- **Rust** (stable) — [rustup.rs](https://rustup.rs)
- **Node.js** 18+ — [nodejs.org](https://nodejs.org)
- **PostgreSQL** 15+ with pgvector extension
- **Tesseract OCR** installed on your system

---

## Installation

### 1. Clone

```bash
git clone https://github.com/yourusername/mockupface.git
cd mockupface
```

### 2. Install Tesseract

```bash
# macOS
brew install tesseract

# Ubuntu / Debian
sudo apt install tesseract-ocr

# Verify
tesseract --version
```

### 3. PostgreSQL + pgvector

```bash
# macOS
brew install postgresql@15
brew services start postgresql@15

# Ubuntu
sudo apt install postgresql-15 postgresql-15-pgvector

# Create DB and run schema
psql -U postgres -c "CREATE DATABASE mockupface;"
psql -U postgres -d mockupface -f backend/schema.sql
```

### 4. Environment

```bash
cd backend
cp .env.example .env
```

Fill in `.env`:

```env
DATABASE_URL=postgres://postgres:yourpassword@localhost:5432/mockupface
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-proj-...
RUST_LOG=info
PORT=8080
```

### 5. Run backend

```bash
cd backend
cargo run
# → http://localhost:8080
```

### 6. Run frontend

```bash
cd frontend
npm install
npm run dev
# → http://localhost:5173
```

---

## API Reference

| Method | Endpoint              | Description                          |
|--------|-----------------------|--------------------------------------|
| GET    | `/health`             | Health check                         |
| POST   | `/api/ocr`            | Tesseract OCR + Claude structuring   |
| POST   | `/api/rag/search`     | pgvector similarity search           |
| POST   | `/api/rag/store`      | Store run embedding to pgvector      |
| POST   | `/api/prompts`        | Claude prompt generation with RAG    |
| POST   | `/api/generate-image` | DALL-E 3 single image                |
| POST   | `/api/qa`             | GPT-4o Vision quality scoring        |
| POST   | `/api/pipeline`       | **Full pipeline — all 5 stages**     |

### Run the full pipeline

```bash
# With product image (Tesseract OCR runs automatically)
curl -X POST http://localhost:8080/api/pipeline \
  -F "image=@product.jpg" \
  -F "platform=etsy" \
  -F "style_hints=Handmade feel,Cozy lifestyle"

# Text only (no image)
curl -X POST http://localhost:8080/api/pipeline \
  -F "product_text=Minimalist floral mug, Bloom Where You're Planted" \
  -F "platform=amazon"
```

---

## How RAG Improves Results Over Time

Every successful run is embedded and stored in pgvector. On the next run for a similar product:

1. Product description is embedded into a 1536-dim vector
2. pgvector runs cosine similarity search against all past runs
3. Top-4 most similar past prompts (with QA scores) are injected into Claude's context
4. Claude generates better prompts by learning from what worked before

The vector store grows with every use — results improve automatically.

---

## Cost Per Run

| Step                | Model                   | Cost (approx)  |
|---------------------|-------------------------|----------------|
| OCR structuring     | claude-sonnet            | ~$0.001        |
| Embeddings (×5)     | text-embedding-3-small   | ~$0.0001       |
| Prompt generation   | claude-sonnet            | ~$0.002        |
| 4× image generation | dall-e-3 standard        | ~$0.32         |
| QA scoring (×4)     | gpt-4o                   | ~$0.01         |
| **Total**           |                          | **~$0.33/run** |

---

## Roadmap

- [ ] User feedback loop — rate mockups to improve RAG quality
- [ ] Batch processing — multiple products in one run
- [ ] Additional platforms (Shopify, Redbubble, eBay)
- [ ] Custom condition slots
- [ ] Export pack — ZIP of all 4 mockups + prompts
- [ ] Mockup history dashboard
- [ ] Fine-tuned model on successful mockup pairs

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

MIT — see [LICENSE](LICENSE).

---

## Author

Built by a software engineer at the intersection of AI and e-commerce automation.

- GitHub: [github.com/vvaghicharla1](https://github.com/vvaghicharla1)
- YouTube: [youtube.com/@cryptoveda](https://youtube.com/@cryptoveda)
