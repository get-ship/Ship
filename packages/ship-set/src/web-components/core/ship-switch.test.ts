import { afterEach, describe, expect, it } from 'vitest'
import './ship-switch'
import type { ShipSwitch } from './ship-switch'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSwitch
  await el.updateComplete
  return el
}

describe('ship-switch', () => {
  afterEach(cleanupTestDom)

  it('renders with role=switch', async () => {
    const el = await mount('<ship-switch></ship-switch>')
    const input = el.shadowRoot!.querySelector('input')!
    expect(input.getAttribute('role')).toBe('switch')
  })

  it('fires ship-change on toggle', async () => {
    const el = await mount('<ship-switch></ship-switch>')
    let detail: { checked: boolean } | null = null
    el.addEventListener('ship-change', (e) => { detail = (e as CustomEvent).detail })
    const input = el.shadowRoot!.querySelector('input')!
    input.checked = true
    input.dispatchEvent(new Event('change', { bubbles: true }))
    expect(detail).toEqual({ checked: true })
    expect(el.checked).toBe(true)
  })
})
