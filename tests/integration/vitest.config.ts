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
import { SHIP_VITEST_DEFAULTS } from '../../packages/test-setup/src/vitest.ts'

const repoRoot = resolve(__dirname, '..', '..')

export default defineConfig({
  resolve: {
    alias: {
      // `@ship/sui-runtime*` aliases now point into the external sui repo.
      // See the comment on the `@ship/sui-dom` block below.
      '@ship/sui-runtime/template-hooks': resolve(
        '/home/madvib/dev/sui/packages/sui/runtime/src/template-hooks/index.ts',
      ),
      '@ship/sui-runtime/dev': resolve(
        '/home/madvib/dev/sui/packages/sui/runtime/src/dev.ts',
      ),
      '@ship/sui-runtime': resolve('/home/madvib/dev/sui/packages/sui/runtime/src/index.ts'),
      // `@ship/sui-dom` + `@ship/sui-dom/test-utils` now live in the external
      // sui repo (post-carve). Point the alias at the same source root
      // (`/home/madvib/dev/sui/packages/sui/dom/src/...`) so this integration
      // suite keeps testing the same module from the same single source. The
      // path is TEMP — gets relativized to `../sui/packages/...` in capstone.
      '@ship/sui-dom/test-utils': resolve(
        '/home/madvib/dev/sui/packages/sui/dom/src/test-utils/compile-sui.ts',
      ),
      '@ship/sui-dom': resolve('/home/madvib/dev/sui/packages/sui/dom/src/index.ts'),
      '@ship/sdk-client': resolve(
        repoRoot,
        'packages/sdk-client/src/index.ts',
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
