// Authority pack test setup. Runs under happy-dom (see vitest.config.ts),
// which provides ResizeObserver / matchMedia / PointerEvent natively and a CSS
// parser that tolerates the modern scoped styles these components inject — so
// no jsdom polyfills are needed here.
export {}
