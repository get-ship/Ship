import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['test/**/*.test.ts'],
    // jsdom for bind/on/for/slot tests that need a DOM. Signal tests
    // don't care; jsdom adds <2ms overhead per run.
    environment: 'jsdom',
  },
})
