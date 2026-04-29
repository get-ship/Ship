import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      '@ship-set/runtime': resolve(__dirname, '../ship-set-runtime/src/index.ts'),
      '@ship-set/dom': resolve(__dirname, '../ship-set-dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
