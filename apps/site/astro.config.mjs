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
  },
});
