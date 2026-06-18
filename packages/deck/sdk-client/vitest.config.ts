import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: [
      // Subpaths first — vite alias matches longest-prefix.
      { find: '@ship/sui-runtime/patch', replacement: resolve(__dirname, '../../sui/runtime/src/patch.ts') },
      { find: '@ship/deck/route', replacement: resolve(__dirname, '../../deck/route/src/index.ts') },
      { find: '@ship/sui-runtime/route', replacement: resolve(__dirname, '../../sui/runtime/src/route/index.ts') },
      { find: '@ship/sui-runtime', replacement: resolve(__dirname, '../../sui/runtime/src/index.ts') },
    ],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
