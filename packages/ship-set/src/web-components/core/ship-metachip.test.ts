import { afterEach, describe, expect, it } from 'vitest'
import './ship-metachip'
import type { ShipMetachip } from './ship-metachip'
import { cleanupTestDom } from './util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipMetachip
  await el.updateComplete
  return el
}

describe('ship-metachip', () => {
  afterEach(cleanupTestDom)

  it('renders label + value from attrs', async () => {
    const el = await mount('<ship-metachip label="Status" value="Draft"></ship-metachip>')
    const root = el.shadowRoot!
    expect(root.querySelector('.k')!.textContent).toContain('Status')
    expect(root.querySelector('.v')!.textContent).toContain('Draft')
  })

  it('slotted default content overrides value attr', async () => {
    const el = await mount('<ship-metachip label="Pipeline">tdd</ship-metachip>')
    expect(el.textContent?.trim()).toBe('tdd')
  })

  it('reflects label + value attributes', async () => {
    const el = await mount('<ship-metachip label="Agent" value="test-writer"></ship-metachip>')
    expect(el.getAttribute('label')).toBe('Agent')
    expect(el.getAttribute('value')).toBe('test-writer')
  })
})
