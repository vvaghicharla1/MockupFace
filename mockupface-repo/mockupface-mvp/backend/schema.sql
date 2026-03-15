-- Run once: psql -U postgres -d mockupface -f schema.sql

CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS mockup_runs (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    product_text     TEXT NOT NULL,
    platform         TEXT NOT NULL,
    ocr_text         TEXT,
    style_hints      TEXT[],
    condition_id     TEXT NOT NULL,
    condition_label  TEXT NOT NULL,
    environment      TEXT NOT NULL,
    prompt_text      TEXT NOT NULL,
    negative_prompt  TEXT,
    image_url        TEXT,
    qa_score         FLOAT,
    qa_passed        BOOLEAN DEFAULT TRUE,
    user_rating      INT,
    embedding        vector(1536)
);

CREATE INDEX IF NOT EXISTS mockup_runs_embedding_idx
    ON mockup_runs USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

CREATE INDEX IF NOT EXISTS mockup_runs_platform_idx
    ON mockup_runs (platform, qa_passed);

CREATE OR REPLACE VIEW rag_candidates AS
    SELECT * FROM mockup_runs
    WHERE qa_passed = TRUE
      AND (qa_score IS NULL OR qa_score >= 0.7)
      AND (user_rating IS NULL OR user_rating >= 3)
    ORDER BY created_at DESC;
