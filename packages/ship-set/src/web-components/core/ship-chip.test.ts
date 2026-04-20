import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-chip'
import type { ShipChip } from './ship-chip'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipChip
  await el.updateComplete
  return el
}

describe('ship-chip', () => {
  afterEach(cleanupTestDom)

  it('renders slotted label', async () => {
    const el = await mount('<ship-chip>typescript</ship-chip>')
    expect(el.textContent).toBe('typescript')
  })

  it('no remove button by default', async () => {
    const el = await mount('<ship-chip>x</ship-chip>')
    expect(el.shadowRoot!.querySelector('.remove')).toBeNull()
  })

  it('emits ship-remove when × clicked', async () => {
    const el = await mount('<ship-chip removable>x</ship-chip>')
    const spy = vi.fn()
    el.addEventListener('ship-remove', spy)
    ;(el.shadowRoot!.querySelector('.remove') as HTMLButtonElement).click()
    expect(spy).toHaveBeenCalledTimes(1)
  })

  it('reflects selected attribute', async () => {
    const el = await mount('<ship-chip selected>x</ship-chip>')
    expect(el.hasAttribute('selected')).toBe(true)
  })
})
