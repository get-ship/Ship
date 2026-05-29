import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

export default defineConfig({
  site: 'https://getship.dev',
  output: 'static',
  // (Left CSS handling at Astro's 'auto' default: small sheets inline, the big
  //  shared Desktop sheet stays a cacheable <link>. Forcing 'always' inlined
  //  ~30KB into every page's HTML and killed cross-page CSS caching — a net
  //  loss for multi-surface browsing, and unmeasurable on this box.)
  // One nav: the desktop is the whole site. /features no longer exists
  // (its content lives in windows); auth bounces to the login window.
  redirects: {
    '/features': '/',
    '/sign-in': '/login',
    '/sign-up': '/login',
    // Account, billing, and the store live in the app, not the marketing site.
    '/account': '/',
    '/billing': '/',
    '/store': '/',
  },
  integrations: [
    sitemap({
      // Routes that are dev-only (gated by import.meta.env.DEV in their
      // frontmatter and redirect to / in prod). Excluded from the sitemap
      // so crawlers don't index a redirect.
      filter: (page) => !page.includes('/coming-soon'),
    }),
  ],
  vite: {
    resolve: {
      conditions: ['browser', 'import', 'module', 'default'],
    },
    // SUI runtime + dom are loaded by importmap from /sui-runtime.js +
    // /sui-dom.js (see BaseLayout.astro); keeping them external in the
    // page bundles prevents Astro from duplicating ~140 KB of runtime
    // into every script chunk that touches a SUI component. Arcade also
    // ships its own pack via `@ship/arcade/bundle` — leave that for the
    // browser to fetch, not Vite to inline.
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
