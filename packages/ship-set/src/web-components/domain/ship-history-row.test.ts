import { afterEach, describe, expect, it, vi } from 'vitest'
import './ship-history-row'
import type { ShipHistoryRow } from './ship-history-row'
import { cleanupTestDom } from '../core/util'

async function mount(attrs: string): Promise<ShipHistoryRow> {
  document.body.insertAdjacentHTML('beforeend', `<ship-history-row ${attrs}></ship-history-row>`)
  const el = document.body.lastElementChild as ShipHistoryRow
  if (typeof (el as any).updateComplete?.then === 'function') {
    await (el as any).updateComplete
  }
  return el
}

describe('ship-history-row', () => {
  afterEach(cleanupTestDom)

  it('renders subject with scope highlighted', async () => {
    const el = await mount('sha="abc1234" subject="feat(runtime): add kernel" author="alice" timestamp="3h ago"')
    const shadow = el.shadowRoot!.textContent || ''
    expect(shadow).toContain('feat(runtime):')
    expect(shadow).toContain('add kernel')
  })

  it('shows additions and deletions when non-zero', async () => {
    const el = await mount('sha="abc1234" subject="fix: typo" additions="5" deletions="2"')
    const shadow = el.shadowRoot!.textContent || ''
    expect(shadow).toContain('+5')
    expect(shadow).toContain('−2')
  })

  it('hides zero stats', async () => {
    const el = await mount('sha="abc1234" subject="chore: bump" additions="0" deletions="0"')
    const shadow = el.shadowRoot!.textContent || ''
    expect(shadow).not.toContain('+0')
    expect(shadow).not.toContain('−0')
  })

  it('has role=button and tabindex=0', async () => {
    const el = await mount('sha="x" subject="s"')
    expect(el.getAttribute('role')).toBe('button')
    expect(el.getAttribute('tabindex')).toBe('0')
  })

  it('dispatches ship-select with sha on click', async () => {
    const el = await mount('sha="deadbee" subject="fix: bug"')
    const handler = vi.fn()
    el.addEventListener('ship-select', handler)
    el.shadowRoot!.querySelector<HTMLElement>('.msg')!.click()
    expect(handler).toHaveBeenCalledOnce()
    expect((handler.mock.calls[0][0] as CustomEvent).detail.sha).toBe('deadbee')
  })

  it('renders active dot as filled primary circle', async () => {
    const el = await mount('sha="x" subject="s" active')
    const svg = el.shadowRoot!.querySelector('svg')!
    const dot = svg.querySelector('circle')!
    expect(dot.getAttribute('fill')).toContain('primary')
  })
})
