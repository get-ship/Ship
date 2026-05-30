import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '@ship/test-setup/vitest'

export default defineConfig({
  test: {
    environment: 'node',
    include: ['test/**/*.test.ts'],
    ...SHIP_VITEST_DEFAULTS,
  },
  resolve: {
    alias: {
      '@ship/sui-build': resolve(__dirname, 'src/index.ts'),
    },
  },
})
