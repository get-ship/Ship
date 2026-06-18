// P1.8: @cloudflare/vitest-pool-workers@0.15.2 dropped the legacy
// `defineWorkersConfig` / `defineWorkersProject` from the
// `./config` subpath. The v4 API is `cloudflareTest(options)` from
// the main entry, which returns a Vite plugin. Compose it via
// Vitest's `defineConfig({ plugins: [...] })`. The option shape
// stays the same — `poolOptions.workers.wrangler.configPath` for
// the wrangler config; `assetsInclude` is a top-level Vite option
// and is forwarded through the plugin.
import { defineConfig } from 'vitest/config'
import { cloudflareTest } from '@cloudflare/vitest-pool-workers'
export default defineConfig({
  // P1.8: @cloudflare/vitest-pool-workers@0.15.2 dropped the legacy
  // `defineWorkersConfig` / `defineWorkersProject` from the `./config`
  // subpath. The v4 API is `cloudflareTest(options)` from the main
  // entry, which returns a Vite plugin and registers the workers
  // pool. The test.poolOptions block is dropped because the plugin
  // configures the pool itself.
  plugins: [
    cloudflareTest({
      wrangler: { configPath: './wrangler.jsonc' },
    }),
  ],
  assetsInclude: ['**/*.sql'],
  test: {},
})
