// Standalone vitest config for the integration suite.
//
// The root `vitest.config.ts` runs in node mode and doesn't see the
// pnpm-workspace `@ship/*` packages (they're symlinked under each
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
      '@ship/ui-runtime/template-hooks': resolve(
        repoRoot,
        'packages/ui-runtime/src/template-hooks/index.ts',
      ),
      '@ship/ui-runtime/dev': resolve(
        repoRoot,
        'packages/ui-runtime/src/dev.ts',
      ),
      '@ship/ui-runtime': resolve(repoRoot, 'packages/ui-runtime/src/index.ts'),
      '@ship/ui-dom/legacy': resolve(
        repoRoot,
        'packages/ui-dom/src/legacy/index.ts',
      ),
      '@ship/ui-dom': resolve(repoRoot, 'packages/ui-dom/src/index.ts'),
      '@ship/ui-sdk-client': resolve(
        repoRoot,
        'packages/ui-sdk-client/src/index.ts',
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
