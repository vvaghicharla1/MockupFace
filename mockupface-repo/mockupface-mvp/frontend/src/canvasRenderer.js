export function renderMockupScene(canvas, prompt, productLabel, idx) {
  const ctx = canvas.getContext('2d')
  const W = canvas.width  = 600
  const H = canvas.height = 420

  const CONDITIONS = [
    { color:'#c8a96e', env:'White Studio',    icon:'☀' },
    { color:'#8b7dcc', env:'Warm Lifestyle',  icon:'🎁' },
    { color:'#5aadcc', env:'Dark Dramatic',   icon:'💼' },
    { color:'#7abf6e', env:'Natural Outdoor', icon:'🌿' },
  ]
  const cond   = CONDITIONS[idx % 4]
  const accent = prompt?.accent || cond.color
  const bgFrom = prompt?.bg_from || cond.color + '55'
  const bgTo   = prompt?.bg_to   || '#0a0a0a'

  // Background gradient
  const bg = ctx.createLinearGradient(0, 0, W, H)
  bg.addColorStop(0, bgFrom)
  bg.addColorStop(1, bgTo)
  ctx.fillStyle = bg
  ctx.fillRect(0, 0, W, H)

  // Subtle grid
  ctx.strokeStyle = accent + '18'
  ctx.lineWidth = 0.5
  for (let x = 0; x < W; x += 30) { ctx.beginPath(); ctx.moveTo(x,0); ctx.lineTo(x,H); ctx.stroke() }
  for (let y = 0; y < H; y += 30) { ctx.beginPath(); ctx.moveTo(0,y); ctx.lineTo(W,y); ctx.stroke() }

  // Floor shadow
  const shadow = ctx.createRadialGradient(W/2, H*0.78, 0, W/2, H*0.78, 110)
  shadow.addColorStop(0, 'rgba(0,0,0,0.55)')
  shadow.addColorStop(1, 'rgba(0,0,0,0)')
  ctx.fillStyle = shadow
  ctx.beginPath(); ctx.ellipse(W/2, H*0.78, 110, 22, 0, 0, Math.PI*2); ctx.fill()

  // Product body
  const px = W/2 - 70, py = H*0.14, pw = 140, ph = 210
  const pg = ctx.createLinearGradient(px, py, px+pw, py+ph)
  pg.addColorStop(0, accent + 'ee')
  pg.addColorStop(0.5, accent + 'aa')
  pg.addColorStop(1, accent + '44')
  ctx.fillStyle = pg
  roundRect(ctx, px, py, pw, ph, 18); ctx.fill()

  // Shine
  const shine = ctx.createLinearGradient(px, py, px+pw*0.6, py+ph*0.35)
  shine.addColorStop(0, 'rgba(255,255,255,0.28)')
  shine.addColorStop(1, 'rgba(255,255,255,0)')
  ctx.fillStyle = shine
  roundRect(ctx, px, py, pw*0.65, ph*0.35, 18); ctx.fill()

  // Product label text
  ctx.save()
  ctx.font = 'bold 12px "DM Mono", monospace'
  ctx.fillStyle = 'rgba(255,255,255,0.9)'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  const label = (productLabel || 'Product').toUpperCase().slice(0, 16)
  ctx.fillText(label, W/2, py + ph/2)
  ctx.restore()

  // Condition badge bottom-left
  ctx.font = '11px "DM Mono", monospace'
  ctx.fillStyle = accent
  ctx.textAlign = 'left'
  ctx.textBaseline = 'bottom'
  ctx.fillText(`${cond.icon}  ${prompt?.environment || cond.env}`, 16, H - 14)

  // Mood words bottom-right
  const mood = Array.isArray(prompt?.mood) ? prompt.mood.slice(0,3).join(' · ') : ''
  ctx.font = '10px "DM Mono", monospace'
  ctx.fillStyle = 'rgba(255,255,255,0.28)'
  ctx.textAlign = 'right'
  ctx.textBaseline = 'bottom'
  ctx.fillText(mood, W - 16, H - 14)

  // Watermark
  ctx.font = '9px monospace'
  ctx.fillStyle = 'rgba(255,255,255,0.07)'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'bottom'
  ctx.fillText('AI PROMPT PREVIEW · MOCKUPFACE MVP', W/2, H - 4)
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath()
  ctx.moveTo(x+r, y)
  ctx.lineTo(x+w-r, y)
  ctx.quadraticCurveTo(x+w, y,   x+w, y+r)
  ctx.lineTo(x+w, y+h-r)
  ctx.quadraticCurveTo(x+w, y+h, x+w-r, y+h)
  ctx.lineTo(x+r, y+h)
  ctx.quadraticCurveTo(x, y+h,   x, y+h-r)
  ctx.lineTo(x, y+r)
  ctx.quadraticCurveTo(x, y,     x+r, y)
  ctx.closePath()
}
