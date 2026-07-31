/*
 * Content collections — Astro picks up `.md` files under
 * `src/content/<collection>` and validates them against this schema.
 *
 * `docs` is the marketing-site's curated guide surface. The long-tail
 * reference lives in apps/docs (starlight). Source of truth for both
 * is `.ship/doc-skills/`; the generator will route articles tagged
 * `audience: 'user'` here.
 *
 * Astro 6 uses the `loader` API for collections — `glob()` walks the
 * directory + parses frontmatter.
 */
import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const docs = defineCollection({
  loader: glob({
    pattern: '**/*.md',
    base: './src/content/docs',
    /* Flatten the slug: file at `getting-started/quick-start.md`
     * gets id `quick-start`, not `getting-started/quick-start`.
     * Section comes from frontmatter; folder structure is just human
     * organisation. */
    generateId: ({ entry }) => entry.split('/').pop()!.replace(/\.md$/, ''),
  }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    section: z.string(),
    order: z.number().default(100),
    kicker: z.string().optional(),
    readTime: z.number().optional(),
    updated: z.string().optional(),
    related: z.array(z.string()).default([]),
    tags: z.array(z.string()).default([]),
    status: z.enum(['draft', 'ready']).default('ready'),
  }),
});

export const collections = { docs };
