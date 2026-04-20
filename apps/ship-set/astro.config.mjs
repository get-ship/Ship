import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://set.getship.dev',
  output: 'static',
  vite: {
    ssr: { noExternal: ['@ship/ui'] },
    resolve: { conditions: ['browser', 'import', 'module', 'default'] },
  },
});
