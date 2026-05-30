/*
 * `[surface].md.ts` — markdown mirror for every marketing surface.
 *
 * Pattern from developers.cloudflare.com/fundamentals/reference/
 * markdown-for-agents — every doc URL exposes a `.md` companion that
 * serves chrome-free markdown for crawlers and LLMs. We do the same
 * for the OS desktop's surface windows: `/agents` is the rendered
 * Astro page; `/agents.md` is the same content as plain markdown.
 *
 * Source of truth lives in `src/data/marketing-pages.ts`. This route
 * just looks the slug up and serves the body.
 */
import type { APIRoute } from 'astro';
import { marketingPages, pageBySlug } from '../data/marketing-pages';

/* Docs have their own mirror route at pages/docs/[...slug].md.ts — this
 * root mirror handles only the top-level surface summaries. */
export function getStaticPaths() {
  return marketingPages.map((p) => ({ params: { surface: p.slug } }));
}

export const GET: APIRoute = ({ params }) => {
  const slug = params.surface ?? '';
  const page = pageBySlug[slug];
  if (!page) {
    return new Response('not found\n', {
      status: 404,
      headers: { 'content-type': 'text/plain; charset=utf-8' },
    });
  }
  const body = [
    `---`,
    `title: ${JSON.stringify(page.title)}`,
    `description: ${JSON.stringify(page.description)}`,
    `canonical: https://getship.dev/${page.slug}`,
    `---`,
    ``,
    page.body.trim(),
    ``,
  ].join('\n');
  return new Response(body, {
    status: 200,
    headers: {
      'content-type': 'text/markdown; charset=utf-8',
      'cache-control': 'public, max-age=300',
    },
  });
};
