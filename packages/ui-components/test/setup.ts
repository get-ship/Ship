// JSDOM polyfills for Web APIs used by components but absent in jsdom.
// Intentionally minimal — these suppress uncaught errors without simulating
// real behavior. Tests only care about DOM structure / ARIA / event flow.

if (typeof ResizeObserver === 'undefined') {
  // xterm's fit addon calls it but jsdom has no layout engine, so the
  // callback never fires.
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}

if (typeof window !== 'undefined' && typeof window.matchMedia === 'undefined') {
  // xterm.js calls matchMedia for DPR detection; stub returns a minimal object.
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
  // JSDOM's getContext throws "Not implemented" which propagates as an uncaught
  // exception through xterm.js's error handler. Override to return null so
  // xterm gracefully skips canvas/WebGL rendering in the test environment.
  HTMLCanvasElement.prototype.getContext = () => null
}

// xterm.js's WebGL addon throws "WebGL2 not supported null" inside rAF callbacks
// in JSDOM. Patch rAF to swallow errors from third-party library init; the
// terminal test guards xterm availability with explicit `if (el.write !== undefined)`
// checks, so swallowing these throws doesn't mask real assertion failures.
if (typeof requestAnimationFrame !== 'undefined') {
  const _origRaf = globalThis.requestAnimationFrame.bind(globalThis)
  globalThis.requestAnimationFrame = (cb: FrameRequestCallback): number =>
    _origRaf((...args: Parameters<FrameRequestCallback>) => {
      try { cb(...args) } catch { /* suppress third-party init errors in JSDOM */ }
    })
}

// ── Overlay component polyfills (jsdom 25 lacks these) ────────────────────

// ToggleEvent — fired by dialog and popover on open/close transitions.
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

// Popover API — showPopover / hidePopover / togglePopover on HTMLElement.
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

// HTMLDialogElement.showModal / close — jsdom 25 does not implement them.
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
