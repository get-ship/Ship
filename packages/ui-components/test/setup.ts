// Polyfill browser APIs missing from JSDOM that some components reference.
// These stubs are intentionally minimal — they suppress uncaught errors
// without simulating real behavior (which requires a browser).

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  globalThis.ResizeObserver = class {
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

// xterm.js's WebGL addon throws "WebGL2 not supported null" inside rAF callbacks in
// JSDOM. Patch rAF to swallow errors from third-party library initialization; the
// terminal test guards against xterm availability with explicit `if (el.write !== undefined)`
// checks, so swallowing these throws doesn't mask real assertion failures.
if (typeof requestAnimationFrame !== 'undefined') {
  const _origRaf = globalThis.requestAnimationFrame.bind(globalThis)
  globalThis.requestAnimationFrame = (cb: FrameRequestCallback): number =>
    _origRaf((...args: Parameters<FrameRequestCallback>) => {
      try { cb(...args) } catch { /* suppress third-party init errors in JSDOM */ }
    })
}
