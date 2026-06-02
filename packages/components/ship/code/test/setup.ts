// JSDOM polyfills for the code pack.
//
// ResizeObserver is probed by CodeMirror's view layer and @ship/sui-runtime;
// jsdom doesn't ship it.

if (typeof ResizeObserver === 'undefined') {
  // @ts-expect-error — JSDOM stub only
  global.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  }
}
