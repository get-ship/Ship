import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      // The integration test compiles real .sail files; the emitted
      // JS imports `@sail/runtime`. Point it at the local source so
      // dynamic-imported compiled modules can find the runtime.
      '@sail/runtime': resolve(__dirname, 'src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
