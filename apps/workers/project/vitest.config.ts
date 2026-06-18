// P1.8: same fix as apps/workers/platform/vitest.config.ts — the
// legacy `./config` subpath export is gone; the v4 API is
// `cloudflareTest(options)` from the main entry, composed via
// Vitest's defineConfig({ plugins: [...] }). See the platform
// config for the full explanation.
import { defineConfig } from 'vitest/config'
import { cloudflareTest } from '@cloudflare/vitest-pool-workers'
export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: './wrangler.jsonc' },
    }),
  ],
  assetsInclude: ['**/*.sql'],
  test: {},
})
