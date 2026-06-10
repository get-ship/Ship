// JSDOM polyfills for the arcade pack.
//
// ResizeObserver is probed by code paths in @ship/sui-runtime and the
// composed primitives; jsdom doesn't ship it. (Mirrors @ship/agents.)

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}
