// Starlight's docs collection. Uses the default `docsLoader()` which
// is hard-coded to read src/content/docs/ — the directory is fully
// gitignored and the contents are emitted by scripts/collect-skill-docs.ts.
//
// Sources of truth:
//   - docs/<skill>/references/docs/  (skill-derived pages)
//   - src/content-source/            (hand-authored pages — today: index.md)
import { defineCollection } from "astro:content";
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
};
