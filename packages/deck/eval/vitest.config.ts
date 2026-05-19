import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: {
      '@ship/sui-runtime/route': resolve(__dirname, '../../sui/runtime/src/route/index.ts'),
      '@ship/sui-runtime/template-hooks': resolve(__dirname, '../../sui/runtime/src/template-hooks/index.ts'),
      '@ship/sui-runtime': resolve(__dirname, '../../sui/runtime/src/index.ts'),
      '@ship/sui-dom/legacy': resolve(__dirname, '../../sui/dom/src/legacy/index.ts'),
      '@ship/sui-dom': resolve(__dirname, '../../sui/dom/src/index.ts'),
    },
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
