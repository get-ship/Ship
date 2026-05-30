/*
 * `/llms.txt` — index for LLM crawlers + agents.
 *
 * Pattern: a single plain-text file at the site root that lists every
 * indexable page with a one-line description. Agents pick this up
 * before crawling, follow the `.md` mirror for each link, and skip
 * the rendered HTML. See llmstxt.org for the convention.
 */
import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { marketingPages } from '../data/marketing-pages';

const SITE = 'https://getship.dev';

export const GET: APIRoute = async () => {
  const docs = (await getCollection('docs')).sort((a, b) =>
    a.data.section.localeCompare(b.data.section) ||
    (a.data.order ?? 100) - (b.data.order ?? 100),
  );
  const docsBySection: Record<string, typeof docs> = {};
  for (const d of docs) (docsBySection[d.data.section] ??= [] as unknown as typeof docs).push(d);

  const lines: string[] = [];

  lines.push('# Ship');
  lines.push('');
  lines.push(
    '> A personal agentic runtime. Compose agents, tools, and your own approval into workflows the SaaS you use will never ship — and run them on your machine. Free for personal use during public beta.',
  );
  lines.push('');
  lines.push('Every page below has a `.md` companion that serves the same content as plain markdown for agents.');
  lines.push('');

  lines.push('## Surfaces');
  lines.push('');
  for (const p of marketingPages) {
    lines.push(`- [${p.title}](${SITE}/${p.slug}.md): ${p.description}`);
  }
  lines.push('');

  if (docs.length > 0) {
    lines.push('## Docs');
    lines.push('');
    for (const section of Object.keys(docsBySection)) {
      lines.push(`### ${section}`);
      lines.push('');
      for (const d of docsBySection[section]) {
        lines.push(`- [${d.data.title}](${SITE}/docs/${d.id}.md): ${d.data.description}`);
      }
      lines.push('');
    }
  }

  lines.push('## Other');
  lines.push('');
  lines.push(`- [Waitlist](${SITE}/waitlist): Join the pre-launch list — three optional survey questions help us shape the beta.`);
  lines.push(`- [Legal & privacy](${SITE}/legal): Terms and privacy posture during public beta.`);
  lines.push('');

  lines.push('## About this file');
  lines.push('');
  lines.push('- See https://llmstxt.org for the convention.');
  lines.push('- Markdown mirrors follow the developers.cloudflare.com/fundamentals/reference/markdown-for-agents pattern.');
  lines.push('');

  return new Response(lines.join('\n'), {
    status: 200,
    headers: {
      'content-type': 'text/plain; charset=utf-8',
      'cache-control': 'public, max-age=300',
    },
  });
};
