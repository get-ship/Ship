import { describe, expect, it } from 'vitest'
import '../src/ship-agent-card/ship-agent-card.compiled.js'

describe('<ship-agent-card>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-agent-card')).toBeDefined()
  })
})
