import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/ship-run-history/ship-run-history.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
// The rows container carries bind:this AND an authored class. Querying through
// `.rows` pins #285: static attributes must survive alongside the ref binding.
const rows = (el: HTMLElement) => shadow(el).querySelector('.rows')!
const runsAttr = (runs: unknown) => JSON.stringify(runs)

const FIXTURE = [
  { id: 'r-103', status: 'running', trigger: 'push', duration: '—', durationPct: 40, started: 'now' },
  { id: 'r-102', status: 'success', trigger: 'manual', duration: '48s', durationPct: 80, started: '2m' },
  { id: 'r-101', status: 'failed', trigger: 'schedule', duration: '12s', durationPct: 20, started: '1h' },
]

describe('<ship-run-history>', () => {
  it('registers as a custom element', () => {
    expect(customElements.get('ship-run-history')).toBeDefined()
  })

  it('keeps the authored class on the bind:this host (#285)', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: '[]' })
    await flush()
    expect(shadow(el).querySelector('.rows')).not.toBeNull()
  })

  it('renders the header from the title and meta props', async () => {
    const { el } = mount(reg, 'ship-run-history', {
      runs: runsAttr(FIXTURE),
      title: 'Pipeline runs',
      meta: '3 total',
    })
    await flush()
    const hd = shadow(el).querySelector('[part="header"]')!
    expect(hd.querySelector('span')?.textContent).toBe('Pipeline runs')
    expect(hd.querySelector('.cnt')?.textContent).toBe('3 total')
  })

  it('defaults the title to "Recent runs"', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: '[]' })
    await flush()
    expect(shadow(el).querySelector('[part="header"] span')?.textContent).toBe('Recent runs')
  })

  it('renders one row per run, preserving input order', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: runsAttr(FIXTURE) })
    await flush()
    const rowEls = rows(el).querySelectorAll('.row[data-run-id]')
    expect(rowEls.length).toBe(3)
    expect([...rowEls].map((r) => r.getAttribute('data-run-id'))).toEqual([
      'r-103', 'r-102', 'r-101',
    ])
  })

  it('maps run status to a per-status dot class', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: runsAttr(FIXTURE) })
    await flush()
    const dots = [...rows(el).querySelectorAll('.row .dot')]
    expect(dots[0].classList.contains('s-running')).toBe(true)
    expect(dots[1].classList.contains('s-success')).toBe(true)
    expect(dots[2].classList.contains('s-failed')).toBe(true)
  })

  it('renders the duration value and clamps the bar width', async () => {
    const { el } = mount(reg, 'ship-run-history', {
      runs: runsAttr([{ id: 'r-1', duration: '48s', durationPct: 140 }]),
    })
    await flush()
    const row = rows(el).querySelector('.row')!
    expect(row.querySelector('.dur .n')?.textContent).toBe('48s')
    expect((row.querySelector('.dur .b > i') as HTMLElement).style.width).toBe('100%')
  })

  it('exposes each row as a focusable button', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: runsAttr(FIXTURE) })
    await flush()
    const row = rows(el).querySelector('.row')!
    expect(row.getAttribute('role')).toBe('button')
    expect(row.getAttribute('tabindex')).toBe('0')
    expect(row.getAttribute('aria-label')).toContain('r-103')
  })

  it('emits ship-select with the run id on click', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: runsAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    ;(rows(el).querySelectorAll('.row')[1] as HTMLElement).click()
    expect(onSelect).toHaveBeenCalledTimes(1)
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ run: 'r-102' })
  })

  it('emits ship-select on Enter keydown', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: runsAttr(FIXTURE) })
    await flush()
    const onSelect = vi.fn()
    el.addEventListener('ship-select', onSelect)
    const row = rows(el).querySelector('.row') as HTMLElement
    row.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    expect((onSelect.mock.calls[0][0] as CustomEvent).detail).toEqual({ run: 'r-103' })
  })

  it('renders the empty placeholder for no runs', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: '[]' })
    await flush()
    expect(rows(el).querySelector('.empty')?.textContent).toBe('no runs')
  })

  it('falls back to the empty placeholder on invalid JSON', async () => {
    const { el } = mount(reg, 'ship-run-history', { runs: 'xx' })
    await flush()
    expect(rows(el).querySelector('.empty')?.textContent).toBe('no runs')
  })
})
