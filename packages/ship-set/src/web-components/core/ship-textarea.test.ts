import { afterEach, describe, expect, it } from 'vitest'
import './ship-textarea'
import type { ShipTextarea } from './ship-textarea'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipTextarea
  await el.updateComplete
  return el
}

describe('ship-textarea', () => {
  afterEach(cleanupTestDom)

  it('renders with rows + placeholder', async () => {
    const el = await mount('<ship-textarea rows="6" placeholder="body"></ship-textarea>')
    const ta = el.shadowRoot!.querySelector('textarea')!
    expect(ta.rows).toBe(6)
    expect(ta.placeholder).toBe('body')
  })

  it('fires ship-input on user input', async () => {
    const el = await mount('<ship-textarea></ship-textarea>')
    let val: string | undefined
    el.addEventListener('ship-input', (e) => { val = (e as CustomEvent).detail.value })
    const ta = el.shadowRoot!.querySelector('textarea')!
    ta.value = 'hi'
    ta.dispatchEvent(new Event('input', { bubbles: true }))
    expect(val).toBe('hi')
  })
})
