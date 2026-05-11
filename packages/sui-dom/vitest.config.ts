import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '../../vitest.shared'

export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@ship/sui-runtime/template-hooks',
        replacement: resolve(__dirname, '../sui-runtime/src/template-hooks/index.ts'),
      },
      {
        find: '@ship/sui-runtime/route',
        replacement: resolve(__dirname, '../sui-runtime/src/route/index.ts'),
      },
      {
        find: '@ship/sui-runtime',
        replacement: resolve(__dirname, '../sui-runtime/src/index.ts'),
      },
      { find: '@ship/sui-dom', replacement: resolve(__dirname, 'src/index.ts') },
    ],
    // sui-compiler emits `view.js` next to `view.sui` and the paired `view.ts`
    // (see scripts/build-bundles.mjs::compileSui). Tests target the controller
    // source — prefer `.ts` over the generated `.js` shadow.
    extensions: ['.mts', '.ts', '.tsx', '.mjs', '.js', '.jsx', '.json'],
  },
  test: {
    include: ['test/**/*.test.ts'],
    environment: 'jsdom',
    ...SHIP_VITEST_DEFAULTS,
  },
})
