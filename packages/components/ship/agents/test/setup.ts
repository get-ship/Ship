// JSDOM polyfills for the agents pack.
//
// ResizeObserver is probed by code paths in @ship/sail-runtime and the
// composed primitives; jsdom doesn't ship it.

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}
