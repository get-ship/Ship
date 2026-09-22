import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-session-list/ship-session-list.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot ?? el
const rowsAttr = (rows: unknown) => JSON.stringify(rows)

const FIXTURE = [
  { id: '1', status: 'active', harness: 'claude', agent: 'code-reviewer', input: 'Review PR', turns: 5, model: 'opus-4-6', time: 'now', cost: '$0.21' },
  { id: '2', status: 'suspended', harness: 'floki', agent: 'lead-tracker', input: 'Email leads', turns: 1, reason: 'awaiting approval', reasonTone: 'warn' },
  { id: '3', status: 'completed', harness: 'floki', agent: 'lead-tracker', input: 'Summarize', turns: 2 },
  { id: '4', status: 'failed', harness: 'floki', agent: 'spawner', input: 'Spawn child', turns: 1, reason: 'denied', reasonTone: 'fail' },
  { id: '5', status: 'cancelled', harness: 'floki', agent: 'note-taker', input: 'Standup', turns: 0, child: 'child of code-reviewer', reason: 'user-requested', reasonTone: 'mute' },
]

describe('<ship-session-list>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-session-list')).toBeDefined()
  })

  it('renders one row per session from the rows prop (no fixtures)', async () => {
    const { el } = mount(reg, 'ship-session-list', { rows: rowsAttr(FIXTURE), scope: 'space-acme' })
    await flush()
    expect(shadow(el).querySelectorAll('[data-test="session-row"]').length).toBe(5)
    expect(shadow(el).querySelector('[data-test="list-count"]')?.textContent).toBe('5 sessions')
  })

  it('renders the empty state with no rows', async () => {
    const { el } = mount(reg, 'ship-session-list', { rows: '[]' })
    await flush()
    expect(shadow(el).querySelector('[data-test="list-empty"]')).not.toBeNull()
  })

  it('the status filter folds the rows and emits ship-filter', async () => {
    const { el } = mount(reg, 'ship-session-list', { rows: rowsAttr(FIXTURE) })
    await flush()
    const onFilter = vi.fn()
    el.addEventListener('ship-filter', onFilter)
    const pills = shadow(el).querySelectorAll('[data-test="filter-pill"]')
    // order: all, active, suspended, completed, failed, cancelled
    ;(pills[1] as HTMLButtonElement).click()
    await flush()
    expect(shadow(el).querySelectorAll('[data-test="session-row"]').length).toBe(1)
    expect((onFilter.mock.calls[0][0] as CustomEvent).detail).toEqual({ status: 'active' })
  })

  it('covers the full five-status fold via per-status dot classes', async () => {
    const { el } = mount(reg, 'ship-session-list', { rows: rowsAttr(FIXTURE) })
    await flush()
    for (const s of ['active', 'suspended', 'completed', 'failed', 'cancelled']) {
      expect(shadow(el).querySelector(`.dot.d-${s}`)).not.toBeNull()
    }
  })
})
