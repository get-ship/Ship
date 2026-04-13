import { afterEach, describe, expect, it } from 'vitest'
import './ship-checkbox'
import type { ShipCheckbox } from './ship-checkbox'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipCheckbox
  await el.updateComplete
  return el
}

describe('ship-checkbox', () => {
  afterEach(cleanupTestDom)

  it('renders a native checkbox', async () => {
    const el = await mount('<ship-checkbox>accept</ship-checkbox>')
    expect(el.shadowRoot!.querySelector('input[type=checkbox]')).toBeTruthy()
  })

  it('fires ship-change with checked state', async () => {
    const el = await mount('<ship-checkbox></ship-checkbox>')
    let detail: { checked: boolean } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })
    const input = el.shadowRoot!.querySelector('input')!
    input.checked = true
    input.dispatchEvent(new Event('change', { bubbles: true }))
    expect(detail).toEqual({ checked: true })
    expect(el.checked).toBe(true)
  })

  it('supports indeterminate state', async () => {
    const el = await mount('<ship-checkbox></ship-checkbox>')
    el.indeterminate = true
    await el.updateComplete
    const input = el.shadowRoot!.querySelector('input')!
    expect(input.indeterminate).toBe(true)
  })
})
