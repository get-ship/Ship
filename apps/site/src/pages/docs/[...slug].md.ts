/*
 * `/docs/<slug>.md` — raw-source mirror for the docs collection.
 *
 * Sibling of `[surface].md.ts` at the project root. We split it out
 * because Astro routes `[surface].md.ts` against single-segment URLs
 * only; the docs paths can be nested via the rest-param `[...slug]`.
 */
import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

const docs = await getCollection('docs');
const docBySlug = Object.fromEntries(docs.map((d) => [d.id, d]));

export function getStaticPaths() {
  return docs.map((d) => ({ params: { slug: d.id } }));
}

export const GET: APIRoute = ({ params }) => {
  const slug = params.slug ?? '';
  const doc = docBySlug[slug];
  if (!doc) {
    return new Response('not found\n', {
      status: 404,
      headers: { 'content-type': 'text/plain; charset=utf-8' },
    });
  }
  const body = ((doc as unknown) as { body?: string }).body ?? '';
  const banner = [
    `---`,
    `title: ${JSON.stringify(doc.data.title)}`,
    `description: ${JSON.stringify(doc.data.description)}`,
    `section: ${JSON.stringify(doc.data.section)}`,
    `canonical: https://getship.dev/docs/${doc.id}`,
    `tags: ${JSON.stringify(doc.data.tags ?? [])}`,
    `updated: ${JSON.stringify(doc.data.updated ?? '')}`,
    `---`,
    ``,
  ].join('\n');
  return new Response(banner + body.trim() + '\n', {
    status: 200,
    headers: {
      'content-type': 'text/markdown; charset=utf-8',
      'cache-control': 'public, max-age=300',
    },
  });
};
