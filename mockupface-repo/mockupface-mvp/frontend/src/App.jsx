import React, { useState } from 'react'
import { PLATFORM_CONFIGS, CONDITIONS } from './constants'
import { useApiKeys } from './hooks/useApiKeys'
import DropZone     from './components/DropZone'
import MockupCard   from './components/MockupCard'
import PipelineLog  from './components/PipelineLog'
import KeyPanel     from './components/KeyPanel'

// ── API helpers ───────────────────────────────────────────────────────────────

async function callPipeline(image, productText, platform, styleHints) {
  const form = new FormData()
  form.append('product_text', productText)
  form.append('platform',     platform)
  form.append('style_hints',  styleHints.join(','))
  if (image) form.append('image', image)

  const res = await fetch('/api/pipeline', { method:'POST', body: form })
  if (!res.ok) {
    const e = await res.json().catch(() => ({}))
    throw new Error(e || `Backend error ${res.status}`)
  }
  return res.json()
}

async function callPromptsOnly(productText, platform, styleHints, anthropicKey) {
  // Frontend-direct Claude call when backend isn't running
  const cfg = PLATFORM_CONFIGS[platform]
  const res = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type':      'application/json',
      'x-api-key':         anthropicKey,
      'anthropic-version': '2023-06-01',
      'anthropic-dangerous-direct-browser-access': 'true',
    },
    body: JSON.stringify({
      model:      'claude-sonnet-4-20250514',
      max_tokens: 1800,
      system: `You are a creative director generating DALL-E 3 product mockup prompts for ${cfg.name}.
${cfg.tips}
Return ONLY a JSON array of exactly 4 objects (c1–c4), no markdown:
[{"id":"c1","label":"Daily Use","environment":"White Studio","prompt":"...","negative_prompt":"...","bg_from":"#hex","bg_to":"#hex","accent":"#hex","mood":["w1","w2","w3"]},...]
c1=Daily Use/White Studio, c2=Gift Presentation/Warm Lifestyle, c3=Professional/Dark Dramatic, c4=Outdoor/Natural`,
      messages: [{
        role: 'user',
        content: `Product: ${productText}\nPlatform: ${platform}\nStyle hints: ${styleHints.join(', ')||'none'}`,
      }],
    }),
  })
  if (!res.ok) throw new Error(`Claude API ${res.status}`)
  const data = await res.json()
  if (data.error) throw new Error(data.error.message)
  const raw = data.content[0].text.replace(/```json?|```/g,'').trim()
  return JSON.parse(raw)
}

// ── Main App ──────────────────────────────────────────────────────────────────

export default function App() {
  const { anthropicKey, setAnthropicKey, openaiKey, setOpenaiKey, hasAnthropic, hasOpenAI } = useApiKeys()

  const [showKeys,     setShowKeys]     = useState(false)
  const [platform,     setPlatform]     = useState('etsy')
  const [image,        setImage]        = useState(null)
  const [productText,  setProductText]  = useState('')
  const [hints,        setHints]        = useState([])
  const [phase,        setPhase]        = useState('idle')      // idle | running | results
  const [activeStage,  setActiveStage]  = useState('')
  const [stages,       setStages]       = useState([])
  const [mockups,      setMockups]      = useState([])
  const [ragHits,      setRagHits]      = useState(0)
  const [error,        setError]        = useState(null)
  const [useBackend,   setUseBackend]   = useState(true)

  const cfg        = PLATFORM_CONFIGS[platform]
  const canRun     = !!(image || productText.trim().length > 2)
  const productLabel = productText || image?.name?.replace(/\.[^.]+$/, '') || 'Product'

  const toggleHint = (s) =>
    setHints(prev => prev.includes(s) ? prev.filter(x => x !== s) : [...prev, s])

  const pushStage = (name, status, detail) =>
    setStages(prev => [...prev.filter(s => s.name !== name), { name, status, detail }])

  // ── Run pipeline ────────────────────────────────────────────────────────────
  const run = async () => {
    setError(null)
    setPhase('running')
    setStages([])
    setMockups([])
    setRagHits(0)

    try {
      if (useBackend) {
        // ── Full backend pipeline ───────────────────────────────────────────
        setActiveStage('Tesseract OCR')
        const result = await callPipeline(image, productText, platform, hints)
        setStages(result.stages)
        setRagHits(result.rag_hits)

        // Map backend response to our mockup shape
        const mapped = result.mockups.map((m, i) => ({
          id:          m.condition_id,
          label:       m.condition_label,
          environment: m.environment,
          prompt:      m.prompt,
          mood:        [],
          accent:      CONDITIONS[i]?.color || '#F5C518',
          bg_from:     CONDITIONS[i]?.color + '55' || '#F5C51855',
          bg_to:       '#0a0a0a',
          imageUrl:    m.image_url,
          qaScore:     m.qa_score,
          qaPassed:    m.qa_passed,
        }))
        setMockups(mapped)

      } else {
        // ── Frontend-only fallback (no Rust backend needed) ─────────────────
        if (!hasAnthropic) throw new Error('Add your Anthropic key to generate prompts without the backend.')

        setActiveStage('Tesseract OCR')
        pushStage('Tesseract OCR', 'skipped', 'Frontend mode — no OCR without backend')
        await delay(400)

        setActiveStage('pgvector RAG')
        pushStage('pgvector RAG', 'skipped', 'Frontend mode — no RAG without backend')
        await delay(300)

        setActiveStage('Claude Prompts')
        const prompts = await callPromptsOnly(productText, platform, hints, anthropicKey)
        pushStage('Claude Prompts', 'ok', `${prompts.length} prompts generated`)

        setActiveStage('DALL-E 3')
        let imageUrls = []
        if (hasOpenAI) {
          const results = await Promise.allSettled(prompts.map(p =>
            fetch('https://api.openai.com/v1/images/generations', {
              method: 'POST',
              headers: { 'Content-Type':'application/json', 'Authorization':`Bearer ${openaiKey}` },
              body: JSON.stringify({ model:'dall-e-3', prompt:p.prompt, n:1, size:'1024x1024', quality:'standard' })
            }).then(r => r.json()).then(d => d.data?.[0]?.url || null)
          ))
          imageUrls = results.map(r => r.status === 'fulfilled' ? r.value : null)
          pushStage('DALL-E 3', 'ok', `${imageUrls.filter(Boolean).length}/${prompts.length} images generated`)
        } else {
          pushStage('DALL-E 3', 'skipped', 'No OpenAI key — showing canvas previews')
        }

        pushStage('GPT-4o QA + Store', 'skipped', 'Frontend mode — QA and storage skipped')

        const mapped = prompts.map((p, i) => ({
          ...p,
          accent:   p.accent || CONDITIONS[i]?.color || '#F5C518',
          imageUrl: imageUrls[i] || null,
          qaScore:  undefined,
          qaPassed: true,
        }))
        setMockups(mapped)
      }

      setPhase('results')
    } catch (e) {
      setError(e.message)
      setPhase('idle')
    }
    setActiveStage('')
  }

  const reset = () => {
    setPhase('idle'); setMockups([]); setStages([]); setError(null)
  }

  // ── Render ──────────────────────────────────────────────────────────────────
  return (
    <div style={{ minHeight:'100vh', background:'#080808', color:'#F0EDE6', fontFamily:'DM Sans, sans-serif' }}>
      <style>{CSS}</style>

      {showKeys && (
        <KeyPanel
          anthropicKey={anthropicKey} openaiKey={openaiKey}
          onSetAnthropic={setAnthropicKey} onSetOpenAI={setOpenaiKey}
          onClose={() => setShowKeys(false)}
        />
      )}

      {/* ── Header ── */}
      <header className="header">
        <div className="logo">
          <span className="logo-text">MOCKUPFACE</span>
          <span className="logo-tag">MVP</span>
        </div>
        <div style={{ display:'flex', gap:10, alignItems:'center' }}>
          {/* Backend toggle */}
          <button
            onClick={() => setUseBackend(!useBackend)}
            style={{
              background: useBackend ? '#0a1a0a' : '#1a0d00',
              border: `1px solid ${useBackend ? '#2a5a2a' : '#5a2a00'}`,
              color: useBackend ? '#4CAF50' : '#F56400',
              borderRadius:20, padding:'5px 12px',
              fontSize:10, fontFamily:'monospace', cursor:'pointer',
            }}
          >
            {useBackend ? '⚙ Backend mode' : '⚡ Frontend mode'}
          </button>

          {/* Key status pills */}
          <button className="key-pill" onClick={() => setShowKeys(true)}>
            <span style={{ width:6, height:6, borderRadius:'50%', background: hasAnthropic ? '#4CAF50' : '#333', display:'inline-block' }} />
            Claude
          </button>
          <button className="key-pill" onClick={() => setShowKeys(true)}>
            <span style={{ width:6, height:6, borderRadius:'50%', background: hasOpenAI ? '#4CAF50' : '#333', display:'inline-block' }} />
            OpenAI
          </button>
          <button className="key-pill" onClick={() => setShowKeys(true)}>🔑 Keys</button>
        </div>
      </header>

      <main style={{ maxWidth:1100, margin:'0 auto', padding:'40px 24px' }}>

        {/* ── Hero ── */}
        <section className="hero">
          <h1 className="hero-title">
            <span>4 Mockups.</span>
            <span style={{ WebkitTextStroke:'1px #F5C518', WebkitTextFillColor:'transparent' }}>4 Conditions.</span>
            <span>One Product.</span>
          </h1>
          <p className="hero-sub">
            OCR → RAG → Claude → DALL-E 3 · Full AI pipeline for {cfg.name} listings
          </p>
        </section>

        {/* ── Condition strip ── */}
        <div className="condition-strip">
          {CONDITIONS.map(c => (
            <div key={c.id} className="cond-chip" style={{ borderColor: c.color + '40' }}>
              <span>{c.icon}</span>
              <span style={{ color: c.color }}>{c.usage}</span>
              <span style={{ color:'#3d3a35' }}>·</span>
              <span style={{ color:'#5a5a5a' }}>{c.environment}</span>
            </div>
          ))}
        </div>

        {phase !== 'results' && (
          <>
            {/* ── Input grid ── */}
            <div className="input-grid">

              {/* Left column */}
              <div className="input-col">

                {/* Platform tabs */}
                <div>
                  <label className="field-label">Platform <span className="req-badge">REQUIRED</span></label>
                  <div style={{ display:'flex', gap:8 }}>
                    {Object.entries(PLATFORM_CONFIGS).map(([key, pc]) => (
                      <button
                        key={key}
                        onClick={() => setPlatform(key)}
                        style={{
                          flex:1, padding:'10px 0',
                          background: platform===key ? (key==='etsy' ? '#1a0d00' : '#1a1300') : '#0e0e0e',
                          border: `1px solid ${platform===key ? pc.color : '#242424'}`,
                          borderRadius:8, color: platform===key ? pc.color : '#5a5a5a',
                          fontSize:13, cursor:'pointer', transition:'all 0.2s',
                          fontFamily:'inherit',
                        }}
                      >
                        {pc.icon} {pc.name}
                      </button>
                    ))}
                    <div style={{
                      flex:0.6, padding:'10px 0', textAlign:'center',
                      background:'#0a0a0a', border:'1px solid #1a1a1a',
                      borderRadius:8, color:'#3d3a35', fontSize:11,
                    }}>
                      + more
                    </div>
                  </div>
                  <p style={{ fontSize:11, color:'#5a5a5a', marginTop:8, lineHeight:1.5 }}>
                    💡 {cfg.tips}
                  </p>
                </div>

                {/* Drop zone */}
                <div>
                  <label className="field-label">Product / Design <span className="req-badge">REQUIRED</span></label>
                  <DropZone
                    image={image} onImageDrop={setImage}
                    productText={productText} onTextChange={setProductText}
                  />
                </div>

                {/* Mode info */}
                <div style={{
                  background: useBackend ? '#0a120a' : '#120d00',
                  border: `1px solid ${useBackend ? '#1e3a1e' : '#3a2000'}`,
                  borderRadius:8, padding:'10px 14px', fontSize:11,
                  color: useBackend ? '#4CAF50' : '#F56400', lineHeight:1.7,
                }}>
                  {useBackend ? (
                    <>⚙ <strong>Backend mode:</strong> All 5 pipeline stages run on your Rust server → Tesseract OCR, pgvector RAG, Claude, DALL-E 3, GPT-4o QA.</>
                  ) : (
                    <>⚡ <strong>Frontend mode:</strong> Claude prompt generation + DALL-E 3 run directly in browser. No OCR, RAG, or QA. Add Anthropic + OpenAI keys above.</>
                  )}
                </div>
              </div>

              {/* Right column */}
              <div className="input-col">

                {/* Style suggestions */}
                <div>
                  <label className="field-label">Style Hints for {cfg.name}</label>
                  <div style={{ display:'flex', flexWrap:'wrap', gap:7, marginTop:2 }}>
                    {cfg.suggestions.map((s, i) => (
                      <button
                        key={i}
                        onClick={() => toggleHint(s)}
                        style={{
                          background: hints.includes(s) ? '#1a1800' : '#0e0e0e',
                          border: `1px solid ${hints.includes(s) ? '#F5C518' : '#242424'}`,
                          color: hints.includes(s) ? '#F5C518' : '#5a5a5a',
                          borderRadius:20, padding:'5px 12px',
                          fontSize:11, cursor:'pointer', transition:'all 0.15s',
                          fontFamily:'monospace',
                        }}
                      >
                        {hints.includes(s) ? '✓ ' : ''}{s}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Condition preview */}
                <div>
                  <label className="field-label">4 Condition Slots</label>
                  <div style={{
                    background:'#0e0e0e', border:'1px solid #1e1e1e',
                    borderRadius:10, overflow:'hidden',
                  }}>
                    <div style={{
                      padding:'10px 14px', borderBottom:'1px solid #1a1a1a',
                      display:'flex', justifyContent:'space-between', alignItems:'center',
                    }}>
                      <span style={{ fontSize:10, color:'#5a5a5a', fontFamily:'monospace' }}>Condition Slots</span>
                      <span style={{ fontSize:10, color:'#F5C518', fontFamily:'monospace' }}>4 images</span>
                    </div>
                    {CONDITIONS.map((c, i) => (
                      <div key={c.id} style={{
                        display:'flex', alignItems:'center', gap:10,
                        padding:'9px 14px',
                        borderBottom: i < 3 ? '1px solid #111' : 'none',
                      }}>
                        <span style={{
                          fontSize:10, fontFamily:'monospace', color:'#3d3a35',
                          background:'#161616', border:'1px solid #222',
                          borderRadius:3, padding:'1px 5px',
                        }}>C{i+1}</span>
                        <span style={{ fontSize:13 }}>{c.icon}</span>
                        <span style={{ fontSize:12, color:'#F0EDE6' }}>{c.usage}</span>
                        <span style={{ color:'#2a2a2a' }}>·</span>
                        <span style={{ fontSize:11, color:'#5a5a5a' }}>{c.environment}</span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* RAG info */}
                <div style={{
                  background:'#0a0a0f', border:'1px solid #1e1e2a',
                  borderRadius:10, padding:'12px 14px',
                }}>
                  <p style={{ fontSize:11, color:'#8b7dcc', fontFamily:'monospace', marginBottom:6 }}>
                    🧠 pgvector RAG
                  </p>
                  <p style={{ fontSize:11, color:'#5a5a5a', lineHeight:1.6 }}>
                    Every successful run is embedded and stored. Future runs retrieve similar past prompts via cosine similarity — results improve automatically over time.
                  </p>
                </div>
              </div>
            </div>

            {/* ── Generate button ── */}
            <div style={{ textAlign:'center', marginTop:32 }}>
              <button
                className={`generate-btn ${!canRun ? 'disabled' : ''}`}
                onClick={canRun ? run : undefined}
              >
                ✦ Generate 4 Mockups
              </button>
              {!canRun && (
                <p style={{ fontSize:12, color:'#3d3a35', marginTop:10 }}>
                  Describe your product or upload an image to begin
                </p>
              )}
            </div>
          </>
        )}

        {/* ── Running phase ── */}
        {phase === 'running' && (
          <div style={{ marginTop:32 }}>
            <div style={{ textAlign:'center', marginBottom:28 }}>
              <div className="spinner" />
              <p style={{ fontFamily:'Bebas Neue, sans-serif', fontSize:26, letterSpacing:'0.1em', color:'#F5C518', marginTop:16 }}>
                Running Pipeline
              </p>
              <p style={{ fontSize:12, color:'#5a5a5a', marginTop:6 }}>
                {activeStage || 'Processing...'}
              </p>
            </div>
            <PipelineLog stages={stages} active={activeStage} />
          </div>
        )}

        {/* ── Error ── */}
        {error && (
          <div style={{
            background:'#1a0000', border:'1px solid #FF444440',
            borderRadius:8, padding:'12px 16px',
            color:'#FF4444', fontSize:12, marginTop:16, fontFamily:'monospace',
          }}>
            ⚠ {error}
          </div>
        )}

        {/* ── Results phase ── */}
        {phase === 'results' && mockups.length > 0 && (
          <>
            {/* Results header */}
            <div style={{
              display:'flex', justifyContent:'space-between', alignItems:'flex-start',
              marginBottom:24, gap:16,
            }}>
              <div>
                <h2 style={{ fontFamily:'Bebas Neue, sans-serif', fontSize:32, letterSpacing:'0.06em', color:'#F0EDE6' }}>
                  {mockups.some(m => m.imageUrl) ? '4 Mockups Ready' : '4 Prompts Ready'}
                </h2>
                <p style={{ fontSize:12, color:'#5a5a5a', marginTop:4 }}>
                  {cfg.icon} {cfg.name}
                  {ragHits > 0 && <> · <span style={{ color:'#8b7dcc' }}>🧠 {ragHits} RAG hits used</span></>}
                  {!mockups.some(m => m.imageUrl) && <> · Canvas previews shown — add OpenAI key for real images</>}
                </p>
              </div>
              <div style={{ display:'flex', gap:8 }}>
                <button onClick={reset} style={outlineBtn}>← New Product</button>
                <button onClick={run}   style={outlineBtn}>↻ Regenerate</button>
              </div>
            </div>

            {/* Pipeline log (collapsed view) */}
            <div style={{ marginBottom:24 }}>
              <PipelineLog stages={stages} active="" />
            </div>

            {/* Mockup cards grid */}
            <div className="card-grid">
              {mockups.map((m, i) => (
                <div key={m.id} style={{ animation:`fadeUp 0.45s cubic-bezier(.16,1,.3,1) ${i*0.07}s both` }}>
                  <MockupCard
                    prompt={m}
                    idx={i}
                    platform={platform}
                    imageUrl={m.imageUrl}
                    qaScore={m.qaScore}
                    qaPassed={m.qaPassed}
                  />
                </div>
              ))}
            </div>

            {/* Info bar */}
            <div style={{
              display:'grid', gridTemplateColumns:'1fr 1fr',
              gap:12, marginTop:28,
              background:'#0e0e0e', border:'1px solid #1e1e1e',
              borderRadius:10, padding:'16px 20px',
            }}>
              <div>
                <p style={{ fontSize:11, color:'#F5C518', fontFamily:'monospace', marginBottom:5 }}>File naming</p>
                <p style={{ fontSize:11, color:'#5a5a5a', lineHeight:1.6 }}>
                  <code style={{ color:'#8a8278' }}>mockupface_[condition]_[platform]_v1.png</code> — re-runs auto-increment version.
                </p>
              </div>
              <div>
                <p style={{ fontSize:11, color:'#F5C518', fontFamily:'monospace', marginBottom:5 }}>Key security</p>
                <p style={{ fontSize:11, color:'#5a5a5a', lineHeight:1.6 }}>
                  API keys live in React state only. Never stored, never logged. Gone on tab close.
                </p>
              </div>
            </div>
          </>
        )}
      </main>
    </div>
  )
}

// ── Helpers ───────────────────────────────────────────────────────────────────
const delay = (ms) => new Promise(r => setTimeout(r, ms))

const outlineBtn = {
  background: 'transparent',
  border: '1px solid #2a2a2a',
  color: '#8a8278',
  borderRadius: 7,
  padding: '8px 16px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'monospace',
}

// ── Styles ────────────────────────────────────────────────────────────────────
const CSS = `
@import url('https://fonts.googleapis.com/css2?family=Bebas+Neue&family=DM+Sans:wght@400;500&family=DM+Mono:wght@400;500&display=swap');
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
::-webkit-scrollbar { width: 2px; }
::-webkit-scrollbar-thumb { background: #F5C518; border-radius: 2px; }

@keyframes fadeUp { from { opacity:0; transform:translateY(14px); } to { opacity:1; transform:translateY(0); } }
@keyframes spin   { to { transform: rotate(360deg); } }
@keyframes shimmer{ 0%{background-position:-200% center} 100%{background-position:200% center} }

.header {
  position: sticky; top: 0; z-index: 100;
  background: rgba(8,8,8,0.92); backdrop-filter: blur(20px);
  border-bottom: 1px solid #1a1a1a;
  padding: 0 32px; height: 58px;
  display: flex; align-items: center; justify-content: space-between;
}
.logo { display: flex; align-items: baseline; gap: 10px; }
.logo-text {
  font-family: 'Bebas Neue', sans-serif; font-size: 26px; letter-spacing: 0.12em;
  background: linear-gradient(135deg, #F5C518, #FFE066, #C9A000);
  -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
}
.logo-tag {
  font-size: 9px; color: #3d3a35; font-family: monospace; letter-spacing: 0.12em;
  border: 1px solid #242424; padding: 2px 6px; border-radius: 3px;
}
.key-pill {
  display: flex; align-items: center; gap: 6px;
  padding: 5px 12px; border-radius: 20px; cursor: pointer;
  font-family: monospace; font-size: 11px; letter-spacing: 0.04em;
  background: #111; border: 1px solid #242424; color: #8a8278;
  transition: all 0.2s;
}
.key-pill:hover { border-color: #F5C518; color: #F5C518; }

.hero { text-align: center; padding: 40px 0 24px; animation: fadeUp 0.5s both; }
.hero-title {
  font-family: 'Bebas Neue', sans-serif;
  font-size: clamp(42px, 7vw, 80px);
  letter-spacing: 0.04em; line-height: 0.92;
  display: flex; flex-direction: column; gap: 4px;
}
.hero-sub { font-size: 13px; color: #5a5a5a; margin-top: 14px; font-family: monospace; }

.condition-strip {
  display: flex; flex-wrap: wrap; gap: 8px;
  justify-content: center; padding: 16px 0 32px;
  animation: fadeUp 0.5s 0.05s both;
}
.cond-chip {
  display: flex; align-items: center; gap: 7px;
  padding: 6px 14px; border-radius: 20px;
  background: #0e0e0e; border: 1px solid #1e1e1e;
  font-size: 11px; font-family: monospace;
}

.input-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: 24px;
  animation: fadeUp 0.5s 0.1s both;
}
.input-col { display: flex; flex-direction: column; gap: 20px; }

.field-label {
  display: block; font-size: 10px; color: #8a8278;
  font-family: monospace; letter-spacing: 0.08em; text-transform: uppercase;
  margin-bottom: 8px;
}
.req-badge {
  background: #1a1800; border: 1px solid #F5C51840; color: #F5C518;
  border-radius: 3px; font-size: 8px; padding: 1px 5px; margin-left: 6px;
}

.generate-btn {
  background: #F5C518; color: #000;
  border: none; border-radius: 10px;
  font-family: 'Bebas Neue', sans-serif; font-size: 22px; letter-spacing: 0.1em;
  padding: 14px 48px; cursor: pointer;
  background-size: 200% auto;
  transition: all 0.25s;
  animation: fadeUp 0.5s 0.15s both;
}
.generate-btn:hover:not(.disabled) {
  background-image: linear-gradient(90deg, #F5C518, #FFE066, #C9A000, #F5C518);
  animation: shimmer 1.5s linear infinite;
  transform: translateY(-2px);
}
.generate-btn.disabled { background: #1a1800; color: #3d3a35; cursor: not-allowed; }

.card-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.spinner {
  width: 36px; height: 36px; border-radius: 50%;
  border: 2px solid #1e1e1e; border-top-color: #F5C518;
  animation: spin 0.8s linear infinite; margin: 0 auto;
}

@media (max-width: 700px) {
  .input-grid  { grid-template-columns: 1fr; }
  .card-grid   { grid-template-columns: 1fr; }
  .header      { padding: 0 16px; }
}
`
