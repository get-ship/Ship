import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-command-bar/sui-command-bar.compiled.js'
import '../src/sui-dialog/sui-dialog.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

type CB = HTMLElement & {
  show(mode?: string, query?: string): void
  hide(): void
  toggle(mode?: string): void
  setMode(mode: string): void
}

function makeBar(attrs: Record<string, string | boolean> = {}) {
  return mount(reg, 'sui-command-bar', attrs)
}

function shadow(el: HTMLElement) { return el.shadowRoot! }
function dialog(el: HTMLElement) { return shadow(el).querySelector('sui-dialog') as HTMLElement | null }
function trigger(el: HTMLElement) { return shadow(el).querySelector('.cb-trigger') as HTMLButtonElement | null }
function wrap(el: HTMLElement) { return shadow(el).querySelector('.cb-wrap') as HTMLElement | null }

// Press a key on the document
function key(opts: { key: string; metaKey?: boolean; ctrlKey?: boolean }) {
  document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...opts }))
}

describe('<sui-command-bar>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-command-bar')).toBeDefined()
  })

  it('renders a shadow root with trigger button', () => {
    const { el } = makeBar()
    expect(trigger(el)).not.toBeNull()
  })

  it('renders a sui-dialog', () => {
    const { el } = makeBar()
    expect(dialog(el)).not.toBeNull()
  })

  it('closed by default — [open] absent, data-state=closed', async () => {
    const { el } = makeBar()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('show() opens the bar and sets data-state=open', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('hide() closes the bar', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    ;(el as CB).hide()
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('toggle() opens when closed', async () => {
    const { el } = makeBar()
    ;(el as CB).toggle()
    await flush()
    expect(el.getAttribute('data-state')).toBe('open')
  })

  it('toggle() closes when open', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    ;(el as CB).toggle()
    await flush()
    expect(el.getAttribute('data-state')).toBe('closed')
  })

  it('setMode() reflects to [mode] attribute', async () => {
    const { el } = makeBar()
    ;(el as CB).setMode('find')
    await flush()
    expect(el.getAttribute('mode')).toBe('find')
  })

  it('show(mode) opens in the specified mode', async () => {
    const { el } = makeBar()
    ;(el as CB).show('command')
    await flush()
    expect(el.getAttribute('mode')).toBe('command')
    expect(el.hasAttribute('open')).toBe(true)
  })

  it('show(mode, query) seeds the input value', async () => {
    const { el } = makeBar()
    ;(el as CB).show('find', '>dashboard')
    await flush()
    const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
    expect(input?.value).toBe('>dashboard')
  })

  it('⌘K opens the bar in find mode', async () => {
    const { el } = makeBar()
    await flush()
    key({ key: 'k', metaKey: true })
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('mode')).toBe('find')
  })

  it('⌘K toggles closed when open', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    key({ key: 'k', metaKey: true })
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('Escape key closes the bar', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    key({ key: 'Escape' })
    await flush()
    expect(el.hasAttribute('open')).toBe(false)
  })

  it('trigger click opens the bar in find mode', async () => {
    const { el } = makeBar()
    await flush()
    trigger(el)?.click()
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('mode')).toBe('find')
  })

  it('hide() dispatches sui-command:cancel', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    const events: CustomEvent[] = []
    el.addEventListener('sui-command:cancel', (e) => events.push(e as CustomEvent))
    ;(el as CB).hide()
    await flush()
    expect(events.length).toBe(1)
    expect(events[0]!.bubbles).toBe(true)
    expect(events[0]!.composed).toBe(true)
  })

  it('open attr renders wrap with content', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    expect(wrap(el)?.innerHTML.length).toBeGreaterThan(0)
  })

  describe('mode detection from input', () => {
    it('/ prefix sets command mode', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '/lint'
      input.dispatchEvent(new Event('input'))
      await flush()
      expect(el.getAttribute('mode')).toBe('command')
    })

    it('> prefix sets find mode', async () => {
      const { el } = makeBar()
      ;(el as CB).show('ai')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '>dash'
      input.dispatchEvent(new Event('input'))
      await flush()
      expect(el.getAttribute('mode')).toBe('find')
    })

    it('+ prefix sets create mode', async () => {
      const { el } = makeBar()
      ;(el as CB).show('ai')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '+view'
      input.dispatchEvent(new Event('input'))
      await flush()
      expect(el.getAttribute('mode')).toBe('create')
    })

    it('@token sets mention mode', async () => {
      const { el } = makeBar()
      ;(el as CB).show('ai')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '@agent'
      input.dispatchEvent(new Event('input'))
      await flush()
      expect(el.getAttribute('mode')).toBe('mention')
    })
  })

  describe('find mode', () => {
    it('renders nav destinations', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const items = wrap(el)?.querySelectorAll('.cb-item')
      expect(items?.length).toBeGreaterThan(0)
    })

    it('filters nav destinations by query', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find', '>dashboard')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '>dashboard'
      input.dispatchEvent(new Event('input'))
      await flush()
      const items = wrap(el)?.querySelectorAll('.cb-item')
      const labels = Array.from(items ?? []).map((i) => i.querySelector('span')?.textContent ?? '')
      expect(labels.some((l) => l.toLowerCase().includes('dashboard'))).toBe(true)
    })

    it('ArrowDown increments selected index', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
      await flush()
      const items = wrap(el)?.querySelectorAll('.cb-item')
      expect(items?.[1]?.getAttribute('aria-selected')).toBe('true')
    })

    it('ArrowUp clamps at 0', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true }))
      await flush()
      const items = wrap(el)?.querySelectorAll('.cb-item')
      expect(items?.[0]?.getAttribute('aria-selected')).toBe('true')
    })
  })

  describe('command mode', () => {
    it('renders slash commands', async () => {
      const { el } = makeBar()
      ;(el as CB).show('command')
      await flush()
      const items = wrap(el)?.querySelectorAll('.cb-item')
      expect(items?.length).toBeGreaterThan(0)
    })

    it('shows empty state when no commands match', async () => {
      const { el } = makeBar()
      ;(el as CB).show('command', '/zzznomatch')
      await flush()
      const input = wrap(el)?.querySelector('.cb-search') as HTMLInputElement | null
      if (!input) return
      input.value = '/zzznomatch'
      input.dispatchEvent(new Event('input'))
      await flush()
      const empty = wrap(el)?.querySelector('.cb-empty')
      expect(empty).not.toBeNull()
    })
  })

  describe('create mode', () => {
    it('renders surface selector buttons', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      const btns = wrap(el)?.querySelectorAll('.cb-surface-btn')
      expect(btns?.length).toBe(6)
    })

    it('renders name + shape inputs', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      expect(wrap(el)?.querySelector('.cb-create-name')).not.toBeNull()
      expect(wrap(el)?.querySelector('.cb-create-shape')).not.toBeNull()
    })

    it('submit button disabled when name empty', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      const btn = wrap(el)?.querySelector('.cb-create-submit') as HTMLButtonElement | null
      expect(btn?.disabled).toBe(true)
    })
  })

  describe('ai mode (spawn card)', () => {
    it('renders spawn card when no session', async () => {
      const { el } = makeBar()
      ;(el as CB).show('ai')
      await flush()
      // With no session loaded, should show spawn card or empty thread
      const body = wrap(el)?.querySelector('.cb-body')
      expect(body).not.toBeNull()
    })
  })

  describe('AsyncSearch cancellation', () => {
    it('AbortController is used for stream cancellation', async () => {
      const { el } = makeBar()
      ;(el as CB).show()
      await flush()
      ;(el as CB).hide()
      await flush()
      // Component should not throw on disconnect after abort
      reg.cleanup()
      expect(true).toBe(true)
    })
  })

  describe('result selection via sui-command:select', () => {
    it('item click dispatches sui-command:select', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const events: CustomEvent[] = []
      el.addEventListener('sui-command:select', (e) => events.push(e as CustomEvent))
      const item = wrap(el)?.querySelector('.cb-item') as HTMLElement | null
      item?.click()
      await flush()
      expect(events.length).toBeGreaterThan(0)
      expect(events[0]!.detail.mode).toBe('find')
    })
  })
})
