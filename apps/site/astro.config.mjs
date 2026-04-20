import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://getship.dev',
  output: 'static',
  vite: {
    ssr: {
      noExternal: ['@ship/ui'],
    },
    resolve: {
      conditions: ['browser', 'import', 'module', 'default'],
    },
  },
});
