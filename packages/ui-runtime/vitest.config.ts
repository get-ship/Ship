import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      // The integration test compiles real .set files; the emitted
      // JS imports `@ship/ui-runtime`. Point it at the local source so
      // dynamic-imported compiled modules can find the runtime.
      '@ship/ui-runtime': resolve(__dirname, 'src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
