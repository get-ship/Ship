import { afterEach, describe, expect, it, vi } from 'vitest'
import { formatRelative } from '../src/sui-timestamp/sui-timestamp.ts'
import '../src/sui-timestamp/dist/sui-timestamp.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!
const timeEl = (el: HTMLElement) => shadow(el).querySelector('time')

// ── formatRelative helper ─────────────────────────────────────────────────────

describe('formatRelative()', () => {
  const base = new Date('2024-01-01T12:00:00Z').getTime()

  it('returns "just now" for < 1 minute', () => {
    expect(formatRelative(new Date(base - 30_000).toISOString(), base)).toBe('just now')
  })

  it('returns minutes ago', () => {
    expect(formatRelative(new Date(base - 120_000).toISOString(), base)).toBe('2m ago')
  })

  it('returns hours ago', () => {
    expect(formatRelative(new Date(base - 2 * 3_600_000).toISOString(), base)).toBe('2h ago')
  })

  it('returns days ago', () => {
    expect(formatRelative(new Date(base - 3 * 86_400_000).toISOString(), base)).toBe('3d ago')
  })

  it('returns months ago', () => {
    expect(formatRelative(new Date(base - 60 * 86_400_000).toISOString(), base)).toBe('2mo ago')
  })

  it('returns years ago', () => {
    expect(formatRelative(new Date(base - 400 * 86_400_000).toISOString(), base)).toBe('1y ago')
  })

  it('handles future dates with "from now"', () => {
    expect(formatRelative(new Date(base + 120_000).toISOString(), base)).toBe('2m from now')
  })

  it('returns the raw string for invalid ISO', () => {
    expect(formatRelative('not-a-date', base)).toBe('not-a-date')
  })
})

// ── <sui-timestamp> element ───────────────────────────────────────────────────

describe('<sui-timestamp>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-timestamp')).toBeDefined()
  })

  it('renders empty when value is not set', () => {
    const { el } = mount(reg, 'sui-timestamp')
    expect(timeEl(el)).toBeNull()
  })

  it('renders a <time> element with the value set', async () => {
    const iso = new Date(Date.now() - 60_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    await flush()
    const t = timeEl(el)
    expect(t).not.toBeNull()
    expect(t!.getAttribute('datetime')).toBe(iso)
  })

  it('relative mode (default) shows relative text', async () => {
    const iso = new Date(Date.now() - 120_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    await flush()
    expect(timeEl(el)!.textContent).toBe('2m ago')
  })

  it('absolute mode shows the ISO string', async () => {
    const iso = new Date(Date.now() - 60_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso, format: 'absolute' })
    await flush()
    expect(timeEl(el)!.textContent).toBe(iso)
  })

  it('both mode shows relative time and abs span', async () => {
    const iso = new Date(Date.now() - 120_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso, format: 'both' })
    await flush()
    expect(timeEl(el)!.textContent).toBe('2m ago')
    const abs = shadow(el).querySelector('.abs')!
    expect(abs).not.toBeNull()
    expect(abs.textContent).toContain(iso)
  })

  it('host title is set to the absolute ISO string', async () => {
    const iso = new Date(Date.now() - 60_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    await flush()
    expect(el.title).toBe(iso)
  })

  it('value prop reflects', () => {
    const iso = new Date().toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    expect(el.getAttribute('value')).toBe(iso)
  })

  it('format prop reflects', () => {
    const { el } = mount(reg, 'sui-timestamp', { format: 'absolute' })
    expect(el.getAttribute('format')).toBe('absolute')
  })

  it('<time> has part="time"', async () => {
    const iso = new Date().toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    await flush()
    expect(timeEl(el)!.getAttribute('part')).toBe('time')
  })

  it('switching from relative to absolute updates display', async () => {
    const iso = new Date(Date.now() - 120_000).toISOString()
    const { el } = mount(reg, 'sui-timestamp', { value: iso })
    await flush()
    expect(timeEl(el)!.textContent).toBe('2m ago')

    el.setAttribute('format', 'absolute')
    await flush()
    expect(timeEl(el)!.textContent).toBe(iso)
  })

  it('setInterval is cleared on disconnect (no timer leak)', async () => {
    const setIntervalSpy = vi.spyOn(globalThis, 'setInterval')
    const clearIntervalSpy = vi.spyOn(globalThis, 'clearInterval')

    const { el, container } = mount(reg, 'sui-timestamp', {
      value: new Date().toISOString(),
    })
    await flush()

    const timersBefore = setIntervalSpy.mock.calls.length
    expect(timersBefore).toBeGreaterThan(0)

    container.remove()
    await flush()

    expect(clearIntervalSpy.mock.calls.length).toBeGreaterThan(0)

    setIntervalSpy.mockRestore()
    clearIntervalSpy.mockRestore()
  })
})
