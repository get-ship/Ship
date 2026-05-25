import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-theme-toggle/sui-theme-toggle.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()

beforeEach(() => {
  try {
    localStorage.removeItem('ship-theme')
  } catch {
    /* no-op */
  }
  document.documentElement.classList.remove('dark')
})

afterEach(() => {
  reg.cleanup()
  document.documentElement.classList.remove('dark')
})

const shadow = (el: HTMLElement) => el.shadowRoot!
// The toggle now delegates to <sui-button> inside its shadow root, which
// owns its own shadow tree. ARIA + click both live on the outer
// <sui-button>; tests target it as the affordance.
const btnHost = (el: HTMLElement) => shadow(el).querySelector('sui-button')! as HTMLElement

describe('<sui-theme-toggle>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-theme-toggle')).toBeDefined()
  })

  it('renders a button with aria-label and aria-pressed', async () => {
    const { el } = mount(reg, 'sui-theme-toggle')
    await flush()
    const btn = btnHost(el)
    expect(btn.getAttribute('aria-label')).toBe('Toggle theme')
    expect(btn.getAttribute('aria-pressed')).toMatch(/^(true|false)$/)
  })

  it('resolves auto to light when no storage and no prefers-dark', async () => {
    const original = window.matchMedia
    window.matchMedia = ((q: string) => ({
      matches: false,
      media: q,
      addEventListener() {},
      removeEventListener() {},
      addListener() {},
      removeListener() {},
      dispatchEvent: () => false,
      onchange: null,
    })) as unknown as typeof window.matchMedia

    const { el } = mount(reg, 'sui-theme-toggle')
    await flush()
    expect(el.getAttribute('theme')).toBe('light')
    expect(document.documentElement.classList.contains('dark')).toBe(false)

    window.matchMedia = original
  })

  it('resolves auto from localStorage when present', async () => {
    localStorage.setItem('ship-theme', 'dark')
    const { el } = mount(reg, 'sui-theme-toggle')
    await flush()
    expect(el.getAttribute('theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('flips theme on click + persists to localStorage', async () => {
    const { el } = mount(reg, 'sui-theme-toggle', { theme: 'light' })
    await flush()
    expect(el.getAttribute('theme')).toBe('light')
    btnHost(el).click()
    await flush()
    expect(el.getAttribute('theme')).toBe('dark')
    expect(localStorage.getItem('ship-theme')).toBe('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('emits sui-theme-change on toggle with the new theme', async () => {
    const { el } = mount(reg, 'sui-theme-toggle', { theme: 'light' })
    await flush()
    const handler = vi.fn()
    el.addEventListener('sui-theme-change', handler)
    btnHost(el).click()
    expect(handler).toHaveBeenCalledTimes(1)
    const detail = (handler.mock.calls[0][0] as CustomEvent).detail
    expect(detail).toEqual({ theme: 'dark' })
  })

  it('flips aria-pressed when the theme changes', async () => {
    const { el } = mount(reg, 'sui-theme-toggle', { theme: 'light' })
    await flush()
    expect(btnHost(el).getAttribute('aria-pressed')).toBe('false')
    btnHost(el).click()
    await flush()
    expect(btnHost(el).getAttribute('aria-pressed')).toBe('true')
  })
})
