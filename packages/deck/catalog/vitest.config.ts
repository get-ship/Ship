import { defineConfig } from 'vitest/config'
import { resolve } from 'node:path'
import { SHIP_VITEST_DEFAULTS } from '../../../vitest.shared'

export default defineConfig({
  test: {
    environment: 'node',
    include: ['test/**/*.test.ts'],
    ...SHIP_VITEST_DEFAULTS,
  },
  resolve: {
    alias: {
      '@ship/sui-catalog': resolve(__dirname, 'src/index.ts'),
    },
  },
})
