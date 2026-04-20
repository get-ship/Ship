import { afterEach, describe, expect, it } from 'vitest'
import './ship-spec-row'
import type { ShipSpecRow } from './ship-spec-row'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipSpecRow
  await el.updateComplete
  await el.updateComplete
  return el
}

describe('ship-spec-row', () => {
  afterEach(cleanupTestDom)

  it('renders composed list-item + tone-mapped badge', async () => {
    const el = await mount('<ship-spec-row item-title="Draft" status="draft"></ship-spec-row>')
    const badge = el.shadowRoot!.querySelector('ship-badge')!
    expect(badge.getAttribute('tone')).toBe('muted')
    const li = el.shadowRoot!.querySelector('ship-list-item')!
    expect(li.textContent).toContain('Draft')
  })

  it('falls back to (untitled)', async () => {
    const el = await mount('<ship-spec-row status="ready"></ship-spec-row>')
    const li = el.shadowRoot!.querySelector('ship-list-item')!
    expect(li.textContent).toContain('(untitled)')
  })

  it('maps status=failed to destructive', async () => {
    const el = await mount('<ship-spec-row status="failed"></ship-spec-row>')
    expect(el.shadowRoot!.querySelector('ship-badge')!.getAttribute('tone')).toBe('destructive')
  })
})
