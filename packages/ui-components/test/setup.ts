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
