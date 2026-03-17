// ── AI Model identifiers ──────────────────────────────────────────────────────

pub const CLAUDE_MODEL:        &str = "claude-sonnet-4-20250514";
pub const DALLE_MODEL:         &str = "dall-e-3";
pub const GPT4O_MODEL:         &str = "gpt-4o";
pub const EMBEDDING_MODEL:     &str = "text-embedding-3-small";
pub const EMBEDDING_DIMENSIONS: i32 = 1536;

// ── API base URLs ─────────────────────────────────────────────────────────────

pub const ANTHROPIC_API_BASE:  &str = "https://api.anthropic.com/v1";
pub const OPENAI_API_BASE:     &str = "https://api.openai.com/v1";
pub const ANTHROPIC_VERSION:   &str = "2023-06-01";

// ── Image generation defaults ─────────────────────────────────────────────────

pub const DALLE_DEFAULT_SIZE:    &str = "1024x1024";
pub const DALLE_DEFAULT_QUALITY: &str = "standard";

// ── RAG retrieval ─────────────────────────────────────────────────────────────

pub const RAG_DEFAULT_TOP_K:    i64 = 4;
pub const RAG_MAX_TOP_K:        i64 = 10;
pub const RAG_MIN_QA_SCORE:     f64 = 0.7;
pub const RAG_MIN_USER_RATING:  i32 = 3;

// ── QA scoring ────────────────────────────────────────────────────────────────

pub const QA_PASS_THRESHOLD:    f64 = 0.6;
pub const QA_MAX_TOKENS:        i32 = 300;

// ── Pipeline stage names ──────────────────────────────────────────────────────

pub const STAGE_OCR:     &str = "Tesseract OCR";
pub const STAGE_RAG:     &str = "pgvector RAG";
pub const STAGE_PROMPTS: &str = "Claude Prompts";
pub const STAGE_DALLE:   &str = "DALL-E 3";
pub const STAGE_QA:      &str = "GPT-4o QA";

// ── Platform identifiers ──────────────────────────────────────────────────────

pub const PLATFORM_ETSY:   &str = "etsy";
pub const PLATFORM_AMAZON: &str = "amazon";

// ── Condition slot identifiers ────────────────────────────────────────────────

pub const CONDITION_C1: &str = "c1";
pub const CONDITION_C2: &str = "c2";
pub const CONDITION_C3: &str = "c3";
pub const CONDITION_C4: &str = "c4";

// ── Server defaults ───────────────────────────────────────────────────────────

pub const DEFAULT_PORT:       &str = "8080";
pub const DEFAULT_BIND_ADDR:  &str = "0.0.0.0";

// ── Tesseract OCR config ──────────────────────────────────────────────────────

pub const TESSERACT_OEM: &str = "3";
pub const TESSERACT_PSM: &str = "3";

// ── Claude token limits ───────────────────────────────────────────────────────

pub const CLAUDE_OCR_MAX_TOKENS:    i32 = 400;
pub const CLAUDE_PROMPT_MAX_TOKENS: i32 = 2000;
