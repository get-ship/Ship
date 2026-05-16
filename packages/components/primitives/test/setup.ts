// JSDOM polyfills for Web APIs used by primitive components but absent
// in jsdom. Intentionally minimal — these suppress uncaught errors
// without simulating real behavior. Tests only care about DOM
// structure / ARIA / event flow.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { setIconRegistry } from '../src/sui-icon/registry'

// Prime the lucide icon registry from the on-disk asset. Production
// `<sui-icon>` lazy-fetches the registry from `/_sui/lucide-icons.json`;
// tests don't have that endpoint and assert synchronously, so we inject
// the same map at startup so resolution is sync.
const ICON_REGISTRY_PATH = resolve(
  process.cwd(),
  '../../../crates/sui/ship-sui-server/assets/lucide-icons.json',
)
setIconRegistry(JSON.parse(readFileSync(ICON_REGISTRY_PATH, 'utf8')))

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
