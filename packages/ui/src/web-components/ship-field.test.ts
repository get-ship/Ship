import { afterEach, describe, expect, it } from 'vitest'
import './ship-field'
import type { ShipField } from './ship-field'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipField
  await el.updateComplete
  return el
}

describe('ship-field', () => {
  afterEach(cleanupTestDom)

  it('renders label, default and hint slots', async () => {
    const el = await mount(`
      <ship-field>
        <span slot="label">L</span>
        <input />
        <span slot="hint">H</span>
      </ship-field>
    `)
    const slots = el.shadowRoot!.querySelectorAll('slot')
    expect(slots.length).toBeGreaterThanOrEqual(4)
  })

  it('toggles invalid attribute', async () => {
    const el = await mount('<ship-field invalid></ship-field>')
    expect(el.hasAttribute('invalid')).toBe(true)
  })
})
