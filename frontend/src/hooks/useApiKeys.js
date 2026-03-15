import { useState } from 'react'

export function useApiKeys() {
  const [anthropicKey, setAnthropicKey] = useState('')
  const [openaiKey,    setOpenaiKey]    = useState('')

  const clearKey = (which) => {
    if (which === 'anthropic') setAnthropicKey('')
    if (which === 'openai')    setOpenaiKey('')
  }

  const hasAnthropic = anthropicKey.length > 10
  const hasOpenAI    = openaiKey.length > 10

  return {
    anthropicKey, setAnthropicKey,
    openaiKey,    setOpenaiKey,
    clearKey,
    hasAnthropic,
    hasOpenAI,
  }
}
