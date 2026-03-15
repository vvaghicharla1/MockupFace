import React, { useState, useRef } from 'react'

export default function DropZone({ image, onImageDrop, productText, onTextChange }) {
  const [over, setOver] = useState(false)
  const inputRef = useRef(null)

  const handleDrop = (e) => {
    e.preventDefault(); setOver(false)
    const file = e.dataTransfer.files[0]
    if (file && file.type.startsWith('image/')) onImageDrop(file)
  }

  const handleFile = (e) => {
    const file = e.target.files[0]
    if (file) onImageDrop(file)
  }

  return (
    <div style={{ display:'flex', flexDirection:'column', gap:10 }}>
      {/* Drop area */}
      <div
        onDragOver={(e) => { e.preventDefault(); setOver(true) }}
        onDragLeave={() => setOver(false)}
        onDrop={handleDrop}
        onClick={() => inputRef.current?.click()}
        style={{
          border: `1.5px dashed ${over ? '#F5C518' : image ? '#3a3a3a' : '#242424'}`,
          borderRadius: 10,
          padding: '18px 16px',
          textAlign: 'center',
          cursor: 'pointer',
          background: over ? '#1a180080' : image ? '#111' : '#0e0e0e',
          transition: 'all 0.2s',
        }}
      >
        <input
          ref={inputRef}
          type="file"
          accept="image/*"
          onChange={handleFile}
          style={{ display:'none' }}
        />
        {image ? (
          <div style={{ display:'flex', alignItems:'center', justifyContent:'space-between' }}>
            <div style={{ display:'flex', alignItems:'center', gap:10 }}>
              <span style={{ fontSize:20 }}>🖼️</span>
              <div style={{ textAlign:'left' }}>
                <p style={{ fontSize:12, color:'#F5C518', fontFamily:'monospace' }}>{image.name}</p>
                <p style={{ fontSize:10, color:'#5a5a5a', marginTop:2 }}>
                  {(image.size / 1024).toFixed(0)} KB · Tesseract OCR will extract design info
                </p>
              </div>
            </div>
            <button
              onClick={(e) => { e.stopPropagation(); onImageDrop(null) }}
              style={{ background:'none', border:'1px solid #333', color:'#666', borderRadius:4, padding:'3px 8px', cursor:'pointer', fontSize:11 }}
            >
              ✕
            </button>
          </div>
        ) : (
          <>
            <div style={{ fontSize:28, marginBottom:6 }}>📦</div>
            <p style={{ fontSize:12, color:'#8a8278' }}>Drop product image / PDF</p>
            <p style={{ fontSize:10, color:'#3d3a35', marginTop:4 }}>Tesseract OCR extracts design details automatically</p>
          </>
        )}
      </div>

      {/* Divider */}
      <div style={{ display:'flex', alignItems:'center', gap:10 }}>
        <div style={{ flex:1, height:1, background:'#1e1e1e' }} />
        <span style={{ fontSize:10, color:'#3d3a35', fontFamily:'monospace' }}>or describe your product</span>
        <div style={{ flex:1, height:1, background:'#1e1e1e' }} />
      </div>

      {/* Text input */}
      <textarea
        value={productText}
        onChange={(e) => onTextChange(e.target.value)}
        placeholder="e.g. Minimalist floral mug, 'Bloom Where You're Planted', soft pink and sage green, gold rim..."
        rows={3}
        style={{
          width: '100%',
          background: '#0e0e0e',
          border: '1px solid #242424',
          borderRadius: 8,
          color: '#F0EDE6',
          fontFamily: 'inherit',
          fontSize: 13,
          padding: '10px 12px',
          resize: 'vertical',
          outline: 'none',
          transition: 'border-color 0.2s',
        }}
        onFocus={(e)  => e.target.style.borderColor = '#F5C518'}
        onBlur={(e)   => e.target.style.borderColor = '#242424'}
      />
    </div>
  )
}
