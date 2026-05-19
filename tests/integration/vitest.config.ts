// Standalone vitest config for the integration suite.
//
// The root `vitest.config.ts` runs in node mode and doesn't see the
// bun workspace `@ship/*` packages (they're symlinked under each
// workspace member's `node_modules`, not at the root). This config
// aliases the runtime + ui-dom + ui-sdk-client packages directly to
// their TS sources so the integration tests can import them, and
// switches to jsdom because both suites mount real DOM.

import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '../../vitest.shared'

const repoRoot = resolve(__dirname, '..', '..')

export default defineConfig({
  resolve: {
    alias: {
      '@ship/sui-runtime/template-hooks': resolve(
        repoRoot,
        'packages/sui/runtime/src/template-hooks/index.ts',
      ),
      '@ship/sui-runtime/dev': resolve(
        repoRoot,
        'packages/sui/runtime/src/dev.ts',
      ),
      '@ship/sui-runtime': resolve(repoRoot, 'packages/sui/runtime/src/index.ts'),
      '@ship/sui-dom/legacy': resolve(
        repoRoot,
        'packages/sui/dom/src/legacy/index.ts',
      ),
      '@ship/sui-dom': resolve(repoRoot, 'packages/sui/dom/src/index.ts'),
      '@ship/sdk-client': resolve(
        repoRoot,
        'packages/deck/sdk-client/src/index.ts',
      ),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    include: [resolve(__dirname, '**/*.test.ts')],
    ...SHIP_VITEST_DEFAULTS,
  },
})
