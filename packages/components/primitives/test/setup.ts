// JSDOM polyfills for Web APIs used by primitive components but absent
// in jsdom. Intentionally minimal — these suppress uncaught errors
// without simulating real behavior. Tests only care about DOM
// structure / ARIA / event flow.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { primeIconCache } from '../src/sui-icon/registry'

// Prime the per-icon cache from the committed lucide JSON. Production
// `<sui-icon>` fetches each icon from `/_sui/icons/<name>.svg`; tests
// don't have that endpoint and assert synchronously, so we pre-seed
// every icon as the daemon would return it. Kept in lockstep with
// `deck-core/src/assets.rs::serve_sui_icon` and `ICON_ALIASES`.
const ICON_JSON_PATH = resolve(
  process.cwd(),
  '../../../crates/deck/deck-core/assets/lucide-icons.json',
)
const ICONS: Record<string, string> = JSON.parse(readFileSync(ICON_JSON_PATH, 'utf8'))
const wrap = (inner: string) =>
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" ` +
  `stroke="currentColor" stroke-width="2" stroke-linecap="round" ` +
  `stroke-linejoin="round">${inner}</svg>`
for (const [name, inner] of Object.entries(ICONS)) primeIconCache(name, wrap(inner))
for (const [alias, real] of [
  ['git', 'git-branch'],
  ['refresh', 'refresh-cw'],
  ['stop', 'stop-circle'],
  ['warning', 'triangle-alert'],
] as const) {
  if (ICONS[real]) primeIconCache(alias, wrap(ICONS[real]))
}

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}

if (typeof window !== 'undefined' && typeof window.matchMedia === 'undefined') {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: (_query: string) => ({
      matches: false,
      media: _query,
      onchange: null,
      addListener: () => {},
      removeListener: () => {},
      addEventListener: () => {},
      removeEventListener: () => {},
      dispatchEvent: () => false,
    }),
  })
}

if (typeof HTMLCanvasElement !== 'undefined') {
  // JSDOM's getContext throws "Not implemented". Override to return null.
  HTMLCanvasElement.prototype.getContext = () => null
}

// ── Overlay component polyfills (jsdom 25 lacks these) ────────────────────

if (typeof ToggleEvent === 'undefined') {
  // @ts-expect-error — JSDOM stub
  globalThis.ToggleEvent = class ToggleEvent extends Event {
    newState: string
    oldState: string
    constructor(type: string, init: EventInit & { newState?: string; oldState?: string } = {}) {
      super(type, init)
      this.newState = init.newState ?? ''
      this.oldState = init.oldState ?? ''
    }
  }
}

if (typeof PointerEvent === 'undefined') {
  // JSDOM 25 ships MouseEvent but not PointerEvent. Anchor-position
  // primitives (sui-hover-card, sui-tooltip) dispatch synthetic
  // pointerenter/leave in tests; subclass MouseEvent so the same
  // event shape (button, clientX, …) is available without a real
  // pointer device emulation.
  // @ts-expect-error — JSDOM stub
  globalThis.PointerEvent = class PointerEvent extends MouseEvent {
    pointerId: number
    pointerType: string
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init)
      this.pointerId = init.pointerId ?? 0
      this.pointerType = init.pointerType ?? 'mouse'
    }
  }
}

if (typeof HTMLElement !== 'undefined' && !('showPopover' in HTMLElement.prototype)) {
  const _openPopovers = new WeakSet<HTMLElement>()

  Object.defineProperty(HTMLElement.prototype, 'showPopover', {
    value(this: HTMLElement) {
      if (_openPopovers.has(this)) return
      _openPopovers.add(this)
      const ev = new (globalThis as Record<string, typeof ToggleEvent>).ToggleEvent(
        'toggle', { newState: 'open', oldState: 'closed', bubbles: false },
      )
      this.dispatchEvent(ev)
    },
    writable: true, configurable: true,
  })

  Object.defineProperty(HTMLElement.prototype, 'hidePopover', {
    value(this: HTMLElement) {
      if (!_openPopovers.has(this)) return
      _openPopovers.delete(this)
      const ev = new (globalThis as Record<string, typeof ToggleEvent>).ToggleEvent(
        'toggle', { newState: 'closed', oldState: 'open', bubbles: false },
      )
      this.dispatchEvent(ev)
    },
    writable: true, configurable: true,
  })

  Object.defineProperty(HTMLElement.prototype, 'togglePopover', {
    value(this: HTMLElement) {
      if (_openPopovers.has(this)) (this as HTMLElement & { hidePopover(): void }).hidePopover()
      else (this as HTMLElement & { showPopover(): void }).showPopover()
    },
    writable: true, configurable: true,
  })
}

if (typeof HTMLElement !== 'undefined') {
  const dlgProto = (typeof HTMLDialogElement !== 'undefined')
    ? HTMLDialogElement.prototype
    : null
  if (dlgProto && typeof dlgProto.showModal !== 'function') {
    Object.defineProperty(dlgProto, 'showModal', {
      value(this: HTMLDialogElement) { this.setAttribute('open', '') },
      writable: true, configurable: true,
    })
    Object.defineProperty(dlgProto, 'show', {
      value(this: HTMLDialogElement) { this.setAttribute('open', '') },
      writable: true, configurable: true,
    })
    Object.defineProperty(dlgProto, 'close', {
      value(this: HTMLDialogElement) {
        this.removeAttribute('open')
        this.dispatchEvent(new Event('close', { bubbles: false }))
      },
      writable: true, configurable: true,
    })
  }
}
