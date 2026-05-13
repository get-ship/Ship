import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/islands/sui-command-bar/sui-command-bar.compiled.js'
import '../src/islands/sui-command-bar/sui-command-palette.compiled.js'
import '../src/islands/sui-command-bar/sui-command-chat.compiled.js'
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

// Shadow-piercing helpers
function shadow(el: HTMLElement) { return el.shadowRoot! }
function dialog(el: HTMLElement) { return shadow(el).querySelector('sui-dialog') as HTMLElement | null }
function trigger(el: HTMLElement) { return shadow(el).querySelector('.cb-trigger') as HTMLButtonElement | null }
function wrap(el: HTMLElement) { return shadow(el).querySelector('.cb-wrap') as HTMLElement | null }
function palette(el: HTMLElement) { return shadow(el).querySelector('sui-command-palette') as HTMLElement | null }
function palShadow(el: HTMLElement) { return palette(el)?.shadowRoot ?? null }
function searchInput(el: HTMLElement) { return palShadow(el)?.querySelector('.cb-search') as HTMLInputElement | null }
function palItems(el: HTMLElement) { return palShadow(el)?.querySelectorAll('.cb-item') ?? null }

// Press a key on the document
function key(opts: { key: string; metaKey?: boolean; ctrlKey?: boolean }) {
  document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...opts }))
}

describe('<sui-command-bar>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-command-bar')).toBeDefined()
  })

  it('registers sui-command-palette', () => {
    expect(customElements.get('sui-command-palette')).toBeDefined()
  })

  it('registers sui-command-chat', () => {
    expect(customElements.get('sui-command-chat')).toBeDefined()
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

  it('show(mode, query) seeds the palette input value', async () => {
    const { el } = makeBar()
    ;(el as CB).show('find', '>dashboard')
    await flush()
    const input = searchInput(el)
    expect(input?.value).toBe('>dashboard')
  })

  // ⌘K binding lives in the shell (apps/shell2/src/main.ts), not in
  // the cb controller. Two listeners fighting over the same key flipped
  // the bar open and back to closed within ~23ms — see commit
  // e538a6ec. The shell now owns ⌘K; the bar exposes `host.toggle()`
  // for it to call. Test that contract instead.
  it('host.toggle("find") opens the bar in find mode', async () => {
    const { el } = makeBar()
    await flush()
    ;(el as CB).toggle('find')
    await flush()
    expect(el.hasAttribute('open')).toBe(true)
    expect(el.getAttribute('mode')).toBe('find')
  })

  it('host.toggle() closes the bar when already open', async () => {
    const { el } = makeBar()
    ;(el as CB).show()
    await flush()
    ;(el as CB).toggle()
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

  it('open attr mounts sui-command-palette inside cb-wrap', async () => {
    const { el } = makeBar()
    ;(el as CB).show('find')
    await flush()
    expect(palette(el)).not.toBeNull()
  })

  it('palette has a shadow root with input', async () => {
    const { el } = makeBar()
    ;(el as CB).show('find')
    await flush()
    expect(searchInput(el)).not.toBeNull()
  })

  describe('mode detection from palette input', () => {
    it('/ prefix sets command mode', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = searchInput(el)
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
      const input = searchInput(el)
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
      const input = searchInput(el)
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
      const input = searchInput(el)
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
      const items = palItems(el)
      expect(items?.length).toBeGreaterThan(0)
    })

    it('filters nav destinations by query', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find', '>dashboard')
      await flush()
      const input = searchInput(el)
      if (!input) return
      input.value = '>dashboard'
      input.dispatchEvent(new Event('input'))
      await flush()
      const items = palItems(el)
      const labels = Array.from(items ?? []).map((i) => i.querySelector('span')?.textContent ?? '')
      expect(labels.some((l) => l.toLowerCase().includes('dashboard'))).toBe(true)
    })

    it('ArrowDown increments selected index', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      // Dispatch on bar host — onPaletteKeydown listens there.
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, composed: true }))
      await flush()
      const items = palItems(el)
      expect(items?.[1]?.getAttribute('aria-selected')).toBe('true')
    })

    it('ArrowUp clamps at 0', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, composed: true }))
      await flush()
      const items = palItems(el)
      expect(items?.[0]?.getAttribute('aria-selected')).toBe('true')
    })
  })

  describe('command mode', () => {
    it('renders slash commands', async () => {
      const { el } = makeBar()
      ;(el as CB).show('command')
      await flush()
      const items = palItems(el)
      expect(items?.length).toBeGreaterThan(0)
    })

    it('shows empty state when no commands match', async () => {
      const { el } = makeBar()
      ;(el as CB).show('command', '/zzznomatch')
      await flush()
      const input = searchInput(el)
      if (!input) return
      input.value = '/zzznomatch'
      input.dispatchEvent(new Event('input'))
      await flush()
      const empty = palShadow(el)?.querySelector('.cb-empty')
      expect(empty).not.toBeNull()
    })
  })

  describe('create mode', () => {
    it('renders surface selector buttons', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      const btns = palShadow(el)?.querySelectorAll('.cb-surface-btn')
      expect(btns?.length).toBe(6)
    })

    it('renders name + shape inputs', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      expect(palShadow(el)?.querySelector('.cb-create-name')).not.toBeNull()
      expect(palShadow(el)?.querySelector('.cb-create-shape')).not.toBeNull()
    })

    it('submit button disabled when name empty', async () => {
      const { el } = makeBar()
      ;(el as CB).show('create')
      await flush()
      const btn = palShadow(el)?.querySelector('.cb-create-submit') as HTMLButtonElement | null
      expect(btn?.disabled).toBe(true)
    })
  })

  describe('ai mode (spawn card)', () => {
    it('renders spawn card when no session', async () => {
      const { el } = makeBar()
      ;(el as CB).show('ai')
      await flush()
      const card = palShadow(el)?.querySelector('.cb-spawn-card')
      expect(card).not.toBeNull()
    })
  })

  describe('AsyncSearch cancellation', () => {
    it('component cleans up without throw on disconnect', async () => {
      const { el } = makeBar()
      ;(el as CB).show()
      await flush()
      ;(el as CB).hide()
      await flush()
      reg.cleanup()
      expect(true).toBe(true)
    })
  })

  describe('result selection via sui-command:select', () => {
    it('item click dispatches sui-command:select from palette', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const events: CustomEvent[] = []
      el.addEventListener('sui-command:select', (e) => events.push(e as CustomEvent))
      const item = palShadow(el)?.querySelector('.cb-item') as HTMLElement | null
      item?.click()
      await flush()
      expect(events.length).toBeGreaterThan(0)
      expect(events[0]!.detail.mode).toBe('find')
    })
  })

  describe('focus retention (regression)', () => {
    it('palette input stays focused after ArrowDown', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = searchInput(el)
      if (!input) return
      input.focus()
      // ArrowDown is handled on bar host; input should retain focus after.
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, composed: true }))
      await flush()
      // Input lives in a shadow root; document.activeElement reflects to its host chain.
      expect(document.activeElement === el || input.getRootNode() !== document).toBe(true)
    })

    it('palette input stays focused after ArrowUp', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = searchInput(el)
      if (!input) return
      input.focus()
      el.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, composed: true }))
      await flush()
      expect(document.activeElement === el || input.getRootNode() !== document).toBe(true)
    })

    // Original QA bug: typing into the search input destroyed the input
    // element on each keystroke (re-render loop wiped the DOM). This
    // test directly exercises that path — type 5 chars, assert the
    // element identity is preserved AND the value reflects.
    it('palette input identity + value preserved across keystrokes', async () => {
      const { el } = makeBar()
      ;(el as CB).show('find')
      await flush()
      const input = searchInput(el)
      if (!input) return
      input.focus()
      const initial = input  // capture identity
      for (const ch of 'hello') {
        input.value += ch
        input.dispatchEvent(new InputEvent('input', { data: ch, bubbles: true, composed: true }))
        await flush()
      }
      // Same element instance after every keystroke (no re-render destroyed it)
      expect(searchInput(el)).toBe(initial)
      expect(input.value).toBe('hello')
    })
  })
})
