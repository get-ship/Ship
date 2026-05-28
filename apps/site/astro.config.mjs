import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

export default defineConfig({
  site: 'https://getship.dev',
  output: 'static',
  integrations: [
    sitemap({
      // Routes that are dev-only (gated by import.meta.env.DEV in their
      // frontmatter and redirect to / in prod). Excluded from the sitemap
      // so crawlers don't index a redirect.
      filter: (page) =>
        !page.includes('/pricing') &&
        !page.includes('/coming-soon'),
    }),
  ],
  vite: {
    resolve: {
      conditions: ['browser', 'import', 'module', 'default'],
    },
    // SUI runtime + dom are loaded by importmap from /sui-runtime.js +
    // /sui-dom.js (see BaseLayout.astro); keeping them external in the
    // page bundles prevents Astro from duplicating ~140 KB of runtime
    // into every script chunk that touches a SUI component.
    build: {
      rollupOptions: {
        external: [
          '@ship/sui-runtime',
          '@ship/sui-runtime/route',
          '@ship/sui-runtime/template-hooks',
          '@ship/sui-dom',
          '@ship/sdk-client',
        ],
      },
    },
  },
});
