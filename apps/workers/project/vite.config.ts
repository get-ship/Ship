import { defineConfig } from 'vite';
import { cloudflare } from '@cloudflare/vite-plugin';
import viteTsConfigPaths from 'vite-tsconfig-paths';

export default defineConfig(({ mode }) => ({
  envDir: '../../../',
  plugins: [
    viteTsConfigPaths(),
    {
      name: 'sql-loader',
      transform(code, id) {
        if (/[.]sql/i.test(id)) {
          return `export default ${JSON.stringify(code)}`;
        }
        return { code, map: null };
      },
    },
    mode === 'test'
      ? null
      : cloudflare({
          persistState: { path: '../../../.wrangler/state' },
        }),
  ],
  resolve: {
    conditions: ['development', 'import', 'module', 'browser', 'default'],
  },
  optimizeDeps: {
    exclude: ['cloudflare:*', '@ship-cloud/*'],
    include: ['drizzle-orm'],
  },
}));
