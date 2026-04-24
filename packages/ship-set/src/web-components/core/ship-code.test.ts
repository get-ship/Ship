import { afterEach, describe, expect, it } from 'vitest'
import './ship-code'
import type { ShipCode } from './ship-code'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipCode
  await el.updateComplete
  return el
}

describe('ship-code', () => {
  afterEach(cleanupTestDom)

  it('renders inline code by default', async () => {
    const el = await mount('<ship-code>foo</ship-code>')
    expect(el.shadowRoot!.querySelector('code.inline')).toBeTruthy()
    expect(el.block).toBe(false)
  })

  it('renders block variant with lang label', async () => {
    const el = await mount('<ship-code block lang="ts">const a = 1</ship-code>')
    expect(el.shadowRoot!.querySelector('pre')).toBeTruthy()
    expect(el.shadowRoot!.querySelector('header')?.textContent?.trim()).toBe('ts')
  })

  it('reacts to block toggle', async () => {
    const el = await mount('<ship-code>x</ship-code>')
    el.toggleAttribute('block', true)
    await el.updateComplete
    expect(el.shadowRoot!.querySelector('pre')).toBeTruthy()
  })
})
