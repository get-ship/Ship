import { vi } from 'vitest'

// Expose a minimal `jest` global so @testing-library/dom can detect fake
// timers and auto-advance them inside waitFor(). Vitest does not provide
// this global unless `globals: true` is set in the config.
const jestCompat = {
  advanceTimersByTime: (ms: number) => vi.advanceTimersByTime(ms),
  useRealTimers: () => vi.useRealTimers(),
  useFakeTimers: () => vi.useFakeTimers(),
}

// @ts-expect-error -- intentional global shim for testing-library compat
globalThis.jest = jestCompat
