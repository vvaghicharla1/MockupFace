export const CONDITIONS = [
  { id:'c1', usage:'Daily Use',          environment:'White Studio',    color:'#c8a96e', icon:'☀️' },
  { id:'c2', usage:'Gift Presentation',  environment:'Warm Lifestyle',  color:'#8b7dcc', icon:'🎁' },
  { id:'c3', usage:'Professional',       environment:'Dark Dramatic',   color:'#5aadcc', icon:'💼' },
  { id:'c4', usage:'Outdoor / Adventure',environment:'Natural Outdoor', color:'#7abf6e', icon:'🌿' },
]

export const PLATFORM_CONFIGS = {
  etsy: {
    name:        'Etsy',
    icon:        '🛍️',
    color:       '#F56400',
    sizes:       ['2000×2000px', '1:1 square'],
    tips:        'Handmade & artisanal feel. Lifestyle scenes, warm tones, gifting context perform best.',
    suggestions: [
      'Handmade feel', 'Cozy lifestyle', 'Gift-ready',
      'Vintage aesthetic', 'Nature-inspired', 'Minimal & clean',
      'Boho style', 'Cottagecore', 'Warm golden hour',
    ],
  },
  amazon: {
    name:        'Amazon',
    icon:        '📦',
    color:       '#FF9900',
    sizes:       ['2000×2000px', 'white bg required for main'],
    tips:        'Pure white background for main image. High contrast, clear product, lifestyle for alt images.',
    suggestions: [
      'Pure white background', 'High contrast', 'Multiple angles',
      'Lifestyle shot', 'Bold typography', 'Premium packaging',
      'Clean studio', 'Scale reference', 'Detail closeup',
    ],
  },
}
