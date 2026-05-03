// JSDOM polyfills for Web APIs used by components but absent in jsdom.

if (typeof ResizeObserver === 'undefined') {
  // Minimal no-op stub. xterm's fit addon calls it but jsdom has no layout
  // engine, so the callback never fires — tests only care about DOM structure.
  // @ts-expect-error global augmentation
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
