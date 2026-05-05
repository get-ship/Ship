import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'

export default defineConfig({
  resolve: {
    alias: [
      // Subpaths first — vite alias matches longest-prefix.
      { find: '@ship/ui-runtime/patch', replacement: resolve(__dirname, '../ui-runtime/src/patch.ts') },
      { find: '@ship/ui-runtime/route', replacement: resolve(__dirname, '../ui-runtime/src/route/index.ts') },
      { find: '@ship/ui-runtime', replacement: resolve(__dirname, '../ui-runtime/src/index.ts') },
    ],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
  },
})
