import { afterEach, describe, expect, it } from 'vitest'
import './ship-card'
import type { ShipCard } from './ship-card'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipCard
  await el.updateComplete
  return el
}

describe('ship-card', () => {
  afterEach(cleanupTestDom)

  it('renders header/default/footer slots', async () => {
    const el = await mount(`
      <ship-card>
        <div slot="header">H</div>
        body
        <div slot="footer">F</div>
      </ship-card>
    `)
    expect(el.shadowRoot!.querySelector('slot[name="header"]')).toBeTruthy()
    expect(el.shadowRoot!.querySelector('slot[name="footer"]')).toBeTruthy()
  })

  it('reflects tone + status-rail + accent-bar', async () => {
    const el = await mount('<ship-card tone="info" status-rail accent-bar>x</ship-card>')
    expect(el.tone).toBe('info')
    expect(el.statusRail).toBe(true)
    expect(el.accentBar).toBe(true)
  })

  it('reflects fill attribute for tone-tinted wash', async () => {
    const el = await mount('<ship-card tone="warning" fill>x</ship-card>')
    expect(el.fill).toBe(true)
    expect(el.hasAttribute('fill')).toBe(true)
  })
})
