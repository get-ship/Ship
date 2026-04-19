import { defineConfig } from 'vite';
import { cloudflare } from '@cloudflare/vite-plugin';

export default defineConfig({
  envDir: '../../',
  server: { cors: false, port: 8787 },
  resolve: {
    conditions: ['development', 'import', 'module', 'browser', 'default'],
  },
  plugins: [
    {
      name: 'sql-loader',
      transform(code, id) {
        if (/[.]sql/i.test(id)) {
          return `export default ${JSON.stringify(code)}`;
        }
        return { code, map: null };
      },
    },
    cloudflare({
      auxiliaryWorkers: [
        { configPath: '../../cloud/apps/actors/platform/wrangler.jsonc' },
        { configPath: '../../cloud/apps/actors/workspace/wrangler.jsonc' },
      ],
      persistState: { path: '../../.wrangler/state' },
    }),
  ],
});
