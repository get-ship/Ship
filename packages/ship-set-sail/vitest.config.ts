import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      '@sail/runtime': resolve(__dirname, '../sail-runtime/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    setupFiles: ['./test/setup.ts'],
  },
})
