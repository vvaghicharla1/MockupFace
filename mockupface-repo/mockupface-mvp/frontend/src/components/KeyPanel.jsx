import React, { useState } from 'react'

export default function KeyPanel({ anthropicKey, openaiKey, onSetAnthropic, onSetOpenAI, onClose }) {
  const [antVal, setAntVal] = useState(anthropicKey)
  const [oaiVal, setOaiVal] = useState(openaiKey)

  const save = () => {
    onSetAnthropic(antVal.trim())
    onSetOpenAI(oaiVal.trim())
    onClose()
  }

  return (
    <div style={{
      position: 'fixed', inset:0, background:'rgba(0,0,0,0.75)',
      backdropFilter:'blur(6px)', zIndex:200,
      display:'flex', alignItems:'center', justifyContent:'center',
    }}>
      <div style={{
        background:'#111', border:'1px solid #242424', borderRadius:16,
        padding:28, width:420, maxWidth:'90vw',
      }}>
        <div style={{ display:'flex', justifyContent:'space-between', alignItems:'center', marginBottom:20 }}>
          <h2 style={{ fontFamily:'Bebas Neue, sans-serif', fontSize:22, letterSpacing:'0.08em', color:'#F5C518' }}>
            API Keys
          </h2>
          <button onClick={onClose} style={{ background:'none', border:'none', color:'#5a5a5a', cursor:'pointer', fontSize:18 }}>✕</button>
        </div>

        <div style={{
          background:'#0e0e0e', border:'1px solid #1e1e1e', borderRadius:8,
          padding:'10px 14px', marginBottom:16, fontSize:11, color:'#5a5a5a', lineHeight:1.6,
        }}>
          🔒 Keys are stored in React state only — never logged, never persisted. Gone when you close the tab.
        </div>

        <div style={{ marginBottom:14 }}>
          <label style={{ display:'block', fontSize:10, color:'#8a8278', fontFamily:'monospace', marginBottom:6, letterSpacing:'0.06em' }}>
            ANTHROPIC API KEY <span style={{ color:'#F5C518' }}>— Claude prompt generation</span>
          </label>
          <input
            type="password"
            value={antVal}
            onChange={(e) => setAntVal(e.target.value)}
            placeholder="sk-ant-api03-..."
            style={inputStyle}
          />
        </div>

        <div style={{ marginBottom:20 }}>
          <label style={{ display:'block', fontSize:10, color:'#8a8278', fontFamily:'monospace', marginBottom:6, letterSpacing:'0.06em' }}>
            OPENAI API KEY <span style={{ color:'#FF9900' }}>— DALL-E 3 images + embeddings</span>
          </label>
          <input
            type="password"
            value={oaiVal}
            onChange={(e) => setOaiVal(e.target.value)}
            placeholder="sk-proj-..."
            style={inputStyle}
          />
          <p style={{ fontSize:10, color:'#3d3a35', marginTop:6 }}>
            ~$0.32 per full run (4× DALL-E 3 standard). Also used for pgvector embeddings.
          </p>
        </div>

        <button
          onClick={save}
          style={{
            width:'100%', padding:'12px', background:'#F5C518', color:'#000',
            border:'none', borderRadius:8, fontFamily:'Bebas Neue, sans-serif',
            fontSize:18, letterSpacing:'0.1em', cursor:'pointer',
          }}
        >
          Save Keys
        </button>
      </div>
    </div>
  )
}

const inputStyle = {
  width: '100%',
  background: '#0a0a0a',
  border: '1px solid #242424',
  borderRadius: 7,
  color: '#F0EDE6',
  fontFamily: 'monospace',
  fontSize: 12,
  padding: '9px 12px',
  outline: 'none',
}
