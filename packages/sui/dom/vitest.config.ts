import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '../../../vitest.shared'

export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@ship/sui-runtime/template-hooks',
        replacement: resolve(__dirname, '../runtime/src/template-hooks/index.ts'),
      },
      {
        find: '@ship/sui-runtime/route',
        replacement: resolve(__dirname, '../runtime/src/route/index.ts'),
      },
      {
        find: '@ship/sui-runtime',
        replacement: resolve(__dirname, '../runtime/src/index.ts'),
      },
      { find: '@ship/sui-dom', replacement: resolve(__dirname, 'src/index.ts') },
      // The wasm-backed test helper (src/test-utils/compile-sui.ts) imports
      // the compiler-wasm package. Its `pkg/` is gitignored build output, so
      // bun's workspace linker can't symlink it reliably across worktrees.
      // Alias straight at the built artifact; `just sui-wasm` produces it.
      {
        find: '@ship/sui-compiler-wasm',
        replacement: resolve(__dirname, '../compiler-wasm/pkg/sui_compiler.js'),
      },
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
