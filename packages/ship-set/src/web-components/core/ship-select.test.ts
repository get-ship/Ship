import { afterEach, describe, expect, it } from 'vitest'
import './ship-select'
import type { ShipSelect } from './ship-select'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSelect
  await el.updateComplete
  return el
}

describe('ship-select', () => {
  afterEach(cleanupTestDom)

  it('mirrors light-DOM options into the shadow select', async () => {
    const el = await mount(
      '<ship-select value="b"><option value="a">A</option><option value="b">B</option></ship-select>'
    )
    const sel = el.shadowRoot!.querySelector('select')!
    expect(sel.options.length).toBe(2)
    expect(sel.value).toBe('b')
  })

  it('emits ship-change on selection', async () => {
    const el = await mount(
      '<ship-select><option value="x">X</option><option value="y">Y</option></ship-select>'
    )
    let detail: { value: string } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })
    const sel = el.shadowRoot!.querySelector('select')!
    sel.value = 'y'
    sel.dispatchEvent(new Event('change', { bubbles: true }))
    expect(detail).toEqual({ value: 'y' })
  })
})
