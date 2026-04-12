import { afterEach, describe, expect, it } from 'vitest'
import './ship-list-item'
import type { ShipListItem } from './ship-list-item'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipListItem
  await el.updateComplete
  return el
}

describe('ship-list-item', () => {
  afterEach(cleanupTestDom)

  it('renders + reflects selected', async () => {
    const el = await mount('<ship-list-item selected>A</ship-list-item>')
    const btn = el.shadowRoot!.querySelector('button')!
    expect(btn.getAttribute('aria-selected')).toBe('true')
    expect(btn.getAttribute('role')).toBe('option')
  })

  it('fires ship-select on click', async () => {
    const el = await mount('<ship-list-item>A</ship-list-item>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    el.shadowRoot!.querySelector('button')!.click()
    expect(fired).toBe(1)
  })

  it('does not fire when disabled', async () => {
    const el = await mount('<ship-list-item disabled>A</ship-list-item>')
    let fired = 0
    el.addEventListener('ship-select', () => fired++)
    el.shadowRoot!.querySelector('button')!.click()
    expect(fired).toBe(0)
  })
})
