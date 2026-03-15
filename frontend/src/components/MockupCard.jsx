import React, { useState, useRef, useEffect } from 'react'
import { renderMockupScene } from '../canvasRenderer'

const CONDITIONS = [
  { color:'#c8a96e', icon:'☀️' },
  { color:'#8b7dcc', icon:'🎁' },
  { color:'#5aadcc', icon:'💼' },
  { color:'#7abf6e', icon:'🌿' },
]

export default function MockupCard({ prompt, idx, platform, imageUrl, qaScore, qaPassed }) {
  const canvasRef = useRef(null)
  const [expanded, setExpanded] = useState(false)
  const [copied,   setCopied]   = useState(false)
  const [hovered,  setHovered]  = useState(false)
  const accent = CONDITIONS[idx % 4].color

  // Draw canvas placeholder on mount / when prompt changes
  useEffect(() => {
    if (canvasRef.current) {
      renderMockupScene(canvasRef.current, prompt, prompt.label, idx)
    }
  }, [prompt, idx])

  const copyPrompt = () => {
    navigator.clipboard.writeText(prompt.prompt).catch(() => {})
    setCopied(true)
    setTimeout(() => setCopied(false), 1800)
  }

  const downloadCanvas = () => {
    if (!canvasRef.current) return
    const a = document.createElement('a')
    a.download = `mockupface_${prompt.id}_${platform}_preview.png`
    a.href = canvasRef.current.toDataURL('image/png')
    a.click()
  }

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        background: '#101010',
        border: `1px solid ${hovered ? accent + '55' : '#1e1e1e'}`,
        borderRadius: 14,
        overflow: 'hidden',
        transition: 'all 0.25s cubic-bezier(.16,1,.3,1)',
        transform: hovered ? 'translateY(-3px)' : 'translateY(0)',
        boxShadow: hovered ? `0 10px 32px ${accent}18` : 'none',
      }}
    >
      {/* Image area — real image or canvas placeholder */}
      <div style={{ position:'relative', background:'#0a0a0a' }}>
        {imageUrl ? (
          <img
            src={imageUrl}
            alt={prompt.label}
            style={{ width:'100%', display:'block', aspectRatio:'1/1', objectFit:'cover' }}
          />
        ) : (
          <canvas
            ref={canvasRef}
            style={{ width:'100%', display:'block' }}
          />
        )}

        {/* Condition badge */}
        <div style={{
          position: 'absolute', top:10, left:10,
          background: 'rgba(0,0,0,0.75)', backdropFilter:'blur(8px)',
          border: `1px solid ${accent}40`,
          borderRadius: 20, padding:'4px 10px',
          fontSize:10, color: accent,
          fontFamily:'monospace', letterSpacing:'0.05em',
          display:'flex', alignItems:'center', gap:5,
        }}>
          {CONDITIONS[idx % 4].icon} {prompt.label}
        </div>

        {/* QA badge */}
        {qaScore !== undefined && (
          <div style={{
            position:'absolute', top:10, right:10,
            background: qaPassed ? 'rgba(76,175,80,0.15)' : 'rgba(255,68,68,0.15)',
            border: `1px solid ${qaPassed ? '#4CAF5060' : '#FF444460'}`,
            borderRadius:20, padding:'4px 10px',
            fontSize:10, color: qaPassed ? '#4CAF50' : '#FF4444',
            fontFamily:'monospace',
          }}>
            QA {(qaScore * 100).toFixed(0)}%
          </div>
        )}

        {/* No real image notice */}
        {!imageUrl && (
          <div style={{
            position:'absolute', bottom:8, right:8,
            background:'rgba(0,0,0,0.6)',
            borderRadius:4, padding:'3px 7px',
            fontSize:9, color:'#5a5a5a', fontFamily:'monospace',
          }}>
            placeholder · add OpenAI key for real image
          </div>
        )}
      </div>

      {/* Card body */}
      <div style={{ padding:'12px 14px' }}>
        {/* Environment */}
        <p style={{ fontSize:10, color:'#5a5a5a', fontFamily:'monospace', marginBottom:6, letterSpacing:'0.06em' }}>
          {prompt.environment?.toUpperCase()}
        </p>

        {/* Mood tags */}
        <div style={{ display:'flex', gap:5, flexWrap:'wrap', marginBottom:10 }}>
          {(prompt.mood || []).map((m, i) => (
            <span key={i} style={{
              background: accent + '18', border:`1px solid ${accent}30`,
              color: accent, borderRadius:3,
              fontSize:9, padding:'2px 7px', fontFamily:'monospace',
            }}>
              {m}
            </span>
          ))}
        </div>

        {/* Prompt preview / expand */}
        <div
          onClick={() => setExpanded(!expanded)}
          style={{ cursor:'pointer' }}
        >
          <p style={{
            fontSize:11, color:'#8a8278', lineHeight:1.6,
            overflow: expanded ? 'visible' : 'hidden',
            display: expanded ? 'block' : '-webkit-box',
            WebkitLineClamp: 2,
            WebkitBoxOrient: 'vertical',
            marginBottom:6,
          }}>
            {prompt.prompt}
          </p>
          <span style={{ fontSize:10, color: accent, fontFamily:'monospace' }}>
            {expanded ? '↑ collapse' : '↓ expand prompt'}
          </span>
        </div>

        {/* Actions */}
        <div style={{ display:'flex', gap:7, marginTop:12 }}>
          <button onClick={copyPrompt} style={btnStyle(accent)}>
            {copied ? '✓ copied' : 'copy prompt'}
          </button>
          <button onClick={downloadCanvas} style={btnStyle(accent)}>
            ↓ preview png
          </button>
        </div>
      </div>
    </div>
  )
}

const btnStyle = (accent) => ({
  flex: 1,
  background: 'transparent',
  border: `1px solid #2a2a2a`,
  borderRadius: 6,
  color: '#8a8278',
  fontSize: 10,
  fontFamily: 'monospace',
  padding: '6px 0',
  cursor: 'pointer',
  transition: 'all 0.15s',
  letterSpacing: '0.04em',
})
