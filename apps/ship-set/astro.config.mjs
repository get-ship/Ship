import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://set.getship.dev',
  output: 'static',
  vite: {
    ssr: { noExternal: ['@ship/set'] },
    resolve: { conditions: ['browser', 'import', 'module', 'default'] },
  },
});
