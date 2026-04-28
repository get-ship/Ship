import { describe, expect, it, beforeAll, vi } from 'vitest'
import { loadSailComponent } from './helpers'

type ToastHost = HTMLElement & {
  dismiss?: (reason?: string) => void
  close?: (reason?: string) => Promise<void> | void
}

describe('ship-toast (sail)', () => {
  let tag: string

  beforeAll(async () => {
    tag = await loadSailComponent('ship-toast.set')
  }, 120_000)

  function mount(attrs = '', children = 'hello'): ToastHost {
    const wrap = document.createElement('div')
    wrap.innerHTML = `<ship-toast ${attrs}>${children}</ship-toast>`
    const el = wrap.firstElementChild as ToastHost
    document.body.appendChild(el)
    return el
  }

  it('renders a .toast surface with icon/content and a default close button', () => {
    const el = mount('duration="0"', 'Saved')
    expect(el.querySelector('.toast')).not.toBeNull()
    expect(el.querySelector('.icon')).not.toBeNull()
    expect(el.querySelector('.content')).not.toBeNull()
    expect(el.querySelector('.close')).not.toBeNull()
    expect(el.textContent).toContain('Saved')
    el.remove()
  })

  it('default tone is non-semantic — no auto-icon SVG, data-no-icon set', () => {
    const el = mount('duration="0"', 'plain')
    const auto = el.querySelector('.icon svg')
    expect(auto).toBeNull()
    expect(el.hasAttribute('data-no-icon')).toBe(true)
    el.remove()
  })

  it('semantic tones auto-fill the icon SVG and clear data-no-icon', () => {
    const el = mount('tone="success" duration="0"', 'done')
    const auto = el.querySelector('.icon svg')
    expect(auto).not.toBeNull()
    expect(el.hasAttribute('data-no-icon')).toBe(false)
    el.remove()
  })

  it('dismissible="false" hides the close button via CSS toggle (selector absent)', () => {
    const el = mount('duration="0" dismissible="false"', 'x')
    expect(el.querySelector('.close')).toBeNull()
    el.remove()
  })

  it('clicking the close button dispatches ship-dismiss(reason=manual) and detaches', () => {
    const el = mount('duration="0"', 'x')
    const events: any[] = []
    el.addEventListener('ship-dismiss', (e: any) => events.push(e.detail))
    ;(el.querySelector('.close') as HTMLButtonElement).click()
    expect(events.length).toBe(1)
    expect(events[0]).toEqual({ reason: 'manual' })
    expect(el.isConnected).toBe(false)
  })

  it('auto-dismisses after duration with reason=timeout', () => {
    vi.useFakeTimers()
    try {
      const el = mount('duration="1000"', 'tick')
      const spy = vi.fn()
      el.addEventListener('ship-dismiss', spy)
      vi.advanceTimersByTime(1000)
      expect(spy).toHaveBeenCalledTimes(1)
      const detail = (spy.mock.calls[0][0] as CustomEvent).detail
      expect(detail).toEqual({ reason: 'timeout' })
    } finally {
      vi.useRealTimers()
    }
  })

  it('duration=0 disables the auto-dismiss timer', () => {
    vi.useFakeTimers()
    try {
      const el = mount('duration="0"', 'persist')
      const spy = vi.fn()
      el.addEventListener('ship-dismiss', spy)
      vi.advanceTimersByTime(60_000)
      expect(spy).not.toHaveBeenCalled()
      el.remove()
    } finally {
      vi.useRealTimers()
    }
  })

  it('sets tabindex=0 by default so the toast is focusable', () => {
    const el = mount('duration="0"', 'x')
    expect(el.getAttribute('tabindex')).toBe('0')
    el.remove()
  })

  it('preserves an author-supplied tabindex', () => {
    const el = mount('duration="0" tabindex="-1"', 'x')
    expect(el.getAttribute('tabindex')).toBe('-1')
    el.remove()
  })

  it('Escape key dismisses when dismissible (default)', () => {
    const el = mount('duration="0"', 'x')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(spy).toHaveBeenCalledTimes(1)
  })

  it('Escape is inert when dismissible=false', () => {
    const el = mount('duration="0" dismissible="false"', 'x')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(spy).not.toHaveBeenCalled()
    el.remove()
  })

  it('non-semantic tone renders role=status / aria-live=polite', () => {
    const el = mount('duration="0"', 'x')
    const toast = el.querySelector('.toast')!
    expect(toast.getAttribute('role')).toBe('status')
    expect(toast.getAttribute('aria-live')).toBe('polite')
    expect(toast.getAttribute('aria-atomic')).toBe('true')
    el.remove()
  })

  it('destructive/warning tones escalate to role=alert / aria-live=assertive', () => {
    for (const tone of ['destructive', 'warning']) {
      const el = mount(`tone="${tone}" duration="0"`, 'x')
      const toast = el.querySelector('.toast')!
      expect(toast.getAttribute('role')).toBe('alert')
      expect(toast.getAttribute('aria-live')).toBe('assertive')
      el.remove()
    }
  })

  it('show-progress + duration>0 renders the progress bar and exposes --toast-duration', () => {
    const el = mount('duration="2000" show-progress', 'x')
    expect(el.querySelector('.progress')).not.toBeNull()
    expect(el.style.getPropertyValue('--toast-duration')).toBe('2000ms')
    el.remove()
  })

  it('show-progress with duration=0 does not render the progress bar', () => {
    const el = mount('duration="0" show-progress', 'x')
    expect(el.querySelector('.progress')).toBeNull()
    el.remove()
  })

  it('hover pause + leave resume drives data-paused', () => {
    vi.useFakeTimers()
    try {
      const el = mount('duration="1000"', 'x')
      el.dispatchEvent(new Event('mouseenter'))
      expect(el.hasAttribute('data-paused')).toBe(true)
      const spy = vi.fn()
      el.addEventListener('ship-dismiss', spy)
      // Time advances while paused — no fire.
      vi.advanceTimersByTime(2000)
      expect(spy).not.toHaveBeenCalled()
      el.dispatchEvent(new Event('mouseleave'))
      expect(el.hasAttribute('data-paused')).toBe(false)
      vi.advanceTimersByTime(1000)
      expect(spy).toHaveBeenCalledTimes(1)
    } finally {
      vi.useRealTimers()
    }
  })

  it('focusin/focusout pause and resume the timer like hover', () => {
    vi.useFakeTimers()
    try {
      const el = mount('duration="1000"', 'x')
      el.dispatchEvent(new Event('focusin'))
      expect(el.hasAttribute('data-paused')).toBe(true)
      el.dispatchEvent(new Event('focusout'))
      expect(el.hasAttribute('data-paused')).toBe(false)
    } finally {
      vi.useRealTimers()
    }
  })

  it('slots: title / default / actions distribute, and absent slots set data-no-* flags', () => {
    const el = mount(
      'duration="0"',
      `<span slot="title">T</span>body text<button slot="actions">A</button>`,
    )
    expect(el.querySelector('.title')!.textContent).toContain('T')
    expect(el.querySelector('.body')!.textContent).toContain('body')
    expect(el.querySelector('.actions')!.textContent).toContain('A')
    expect(el.hasAttribute('data-no-title')).toBe(false)
    expect(el.hasAttribute('data-no-actions')).toBe(false)
    el.remove()
  })

  it('omitted title/actions slots set data-no-title and data-no-actions', () => {
    const el = mount('duration="0"', 'just body')
    expect(el.hasAttribute('data-no-title')).toBe(true)
    expect(el.hasAttribute('data-no-actions')).toBe(true)
    el.remove()
  })

  it('exposes el.dismiss() and el.close() imperatively', () => {
    const el = mount('duration="0"', 'x')
    expect(typeof el.dismiss).toBe('function')
    expect(typeof el.close).toBe('function')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    el.dismiss!('programmatic')
    expect(spy).toHaveBeenCalledTimes(1)
    const detail = (spy.mock.calls[0][0] as CustomEvent).detail
    expect(detail).toEqual({ reason: 'programmatic' })
    expect(el.isConnected).toBe(false)
  })

  it('repeated dismiss() calls are idempotent (single ship-dismiss)', () => {
    const el = mount('duration="0"', 'x')
    const spy = vi.fn()
    el.addEventListener('ship-dismiss', spy)
    el.dismiss!('manual')
    el.dismiss!('manual')
    expect(spy).toHaveBeenCalledTimes(1)
  })

  it('tone × size matrix renders without error', () => {
    const tones = ['default', 'info', 'success', 'warning', 'destructive', 'accent', 'muted']
    const sizes = ['sm', 'md', 'lg']
    let n = 0
    for (const tone of tones) {
      for (const size of sizes) {
        const el = mount(`tone="${tone}" size="${size}" duration="0"`, 'x')
        expect(el.querySelector('.toast')).not.toBeNull()
        el.remove()
        n++
      }
    }
    expect(n).toBe(tones.length * sizes.length)
  })
})
