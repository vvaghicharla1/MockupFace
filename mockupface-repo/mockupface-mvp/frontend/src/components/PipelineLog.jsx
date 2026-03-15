import React from 'react'

const STATUS_COLOR = { ok:'#4CAF50', error:'#FF4444', skipped:'#5a5a5a', running:'#F5C518' }
const STATUS_ICON  = { ok:'✓', error:'✕', skipped:'–', running:'◌' }

export default function PipelineLog({ stages, active }) {
  const ALL_STAGES = [
    'Tesseract OCR',
    'pgvector RAG',
    'Claude Prompts',
    'DALL-E 3',
    'GPT-4o QA + Store',
  ]

  return (
    <div style={{
      background: '#0a0a0a',
      border: '1px solid #1e1e1e',
      borderRadius: 10,
      padding: '14px 16px',
      fontFamily: 'monospace',
      fontSize: 11,
    }}>
      <p style={{ color:'#3d3a35', fontSize:10, marginBottom:10, letterSpacing:'0.08em', textTransform:'uppercase' }}>
        Pipeline Log
      </p>
      {ALL_STAGES.map((name, i) => {
        const stage  = stages.find(s => s.name === name)
        const status = stage ? stage.status : (active === name ? 'running' : 'pending')
        const color  = STATUS_COLOR[status] || '#3d3a35'
        const icon   = STATUS_ICON[status]  || '·'
        return (
          <div key={name} style={{
            display: 'flex', alignItems: 'flex-start', gap: 10,
            padding: '5px 0',
            borderBottom: i < ALL_STAGES.length-1 ? '1px solid #111' : 'none',
            opacity: status === 'pending' ? 0.35 : 1,
          }}>
            <span style={{
              color, minWidth:14, textAlign:'center',
              animation: status === 'running' ? 'spin 1s linear infinite' : 'none',
            }}>
              {icon}
            </span>
            <div style={{ flex:1 }}>
              <span style={{ color: status === 'pending' ? '#3d3a35' : '#F0EDE6' }}>{name}</span>
              {stage?.detail && (
                <p style={{ color:'#5a5a5a', marginTop:2, lineHeight:1.4 }}>{stage.detail}</p>
              )}
            </div>
            <span style={{
              fontSize:9, color, background: color+'15',
              padding:'2px 6px', borderRadius:3, textTransform:'uppercase', letterSpacing:'0.06em',
              whiteSpace:'nowrap',
            }}>
              {status}
            </span>
          </div>
        )
      })}
    </div>
  )
}
