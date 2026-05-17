import { cloudflareTest } from '@cloudflare/vitest-pool-workers';
import { defineConfig } from 'vitest/config';

// `@cloudflare/vitest-pool-workers` 0.16.x dropped the old
// `defineWorkersConfig` helper. Tests now opt in via a `cloudflareTest`
// vitest plugin alongside the standard `defineConfig` from `vitest/config`.
// Bindings (D1, KV, secrets) come from wrangler.test.jsonc.

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: './wrangler.test.jsonc' },
    }),
  ],
});
