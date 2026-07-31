import { defineConfig } from 'vitest/config'

// @ship/skills tests cover the pack's pure-derived helpers (scope grouping +
// frontmatter-row projection) plus the controllers' SDK-narrowing. There's no
// `.sui` compile and no shared-harness DOM mounting here, so this stays a plain
// vitest config — happy-dom only for the runtime's DOM probes at import time.
export default defineConfig({
  test: {
    environment: 'happy-dom',
    include: ['test/**/*.test.ts', 'src/**/*.test.ts'],
  },
})
