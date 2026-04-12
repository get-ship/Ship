import { afterEach, describe, expect, it } from 'vitest'
import './ship-label'
import type { ShipLabel } from './ship-label'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipLabel
  await el.updateComplete
  return el
}

describe('ship-label', () => {
  afterEach(cleanupTestDom)

  it('renders slotted text', async () => {
    const el = await mount('<ship-label>Status</ship-label>')
    expect(el.textContent).toBe('Status')
    expect(el.shadowRoot!.querySelector('label')).toBeTruthy()
  })

  it('renders required marker', async () => {
    const el = await mount('<ship-label required>Title</ship-label>')
    expect(el.shadowRoot!.querySelector('.req')).toBeTruthy()
  })

  it('forwards htmlFor', async () => {
    const el = await mount('<ship-label for="x">x</ship-label>')
    expect(el.shadowRoot!.querySelector('label')!.getAttribute('for')).toBe('x')
  })
})
