import { afterEach, describe, expect, it } from 'vitest'
import './ship-json-view'
import type { ShipJsonView } from './ship-json-view'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipJsonView
  await el.updateComplete
  return el
}

describe('ship-json-view', () => {
  afterEach(cleanupTestDom)

  it('renders primitives', async () => {
    const el = await mount(`<ship-json-view data='{"a":1,"b":true,"c":null,"d":"hi"}'></ship-json-view>`)
    const txt = el.shadowRoot!.textContent!
    expect(txt).toContain('"a"')
    expect(txt).toContain('1')
    expect(txt).toContain('true')
    expect(txt).toContain('null')
    expect(txt).toContain('"hi"')
  })

  it('shows error for invalid JSON', async () => {
    const el = await mount(`<ship-json-view data='{bad'></ship-json-view>`)
    expect(el.shadowRoot!.textContent).toContain('Invalid JSON')
  })

  it('reacts to expanded attribute', async () => {
    const el = await mount(
      `<ship-json-view data='{"outer":{"inner":1}}' depth="0"></ship-json-view>`,
    )
    el.expanded = true
    await el.updateComplete
    const openDetails = el.shadowRoot!.querySelectorAll('details[open]')
    expect(openDetails.length).toBeGreaterThan(0)
  })
})
