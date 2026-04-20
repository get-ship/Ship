import { afterEach, describe, expect, it } from 'vitest'
import './ship-spinner'
import type { ShipSpinner } from './ship-spinner'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSpinner
  await el.updateComplete
  return el
}

describe('ship-spinner', () => {
  afterEach(cleanupTestDom)

  it('renders with role=status + default label', async () => {
    const el = await mount('<ship-spinner></ship-spinner>')
    const s = el.shadowRoot!.querySelector('.spin')!
    expect(s.getAttribute('role')).toBe('status')
    expect(s.getAttribute('aria-label')).toBe('Loading')
  })

  it('supports custom label', async () => {
    const el = await mount('<ship-spinner label="Syncing"></ship-spinner>')
    expect(el.shadowRoot!.querySelector('.spin')!.getAttribute('aria-label')).toBe('Syncing')
  })
})
