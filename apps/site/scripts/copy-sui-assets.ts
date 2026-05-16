/**
 * Copy SUI runtime + component bundles into apps/site/public/ so the
 * static Astro build can serve them at the paths the importmap (in
 * src/layouts/BaseLayout.astro) expects.
 *
 * Source of truth: crates/sui/ship-sui-server/assets/ — the same bundles
 * the daemon serves at /api/assets/. This mirrors apps/docs's
 * collect-skill-docs.ts SUI copy step (without the markdoc collection).
 */

import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, '../../..');
const PUBLIC_DIR = join(SCRIPT_DIR, '../public');
const SUI_ASSETS_DIR = join(REPO_ROOT, 'crates/sui/ship-sui-server/assets');

const SUI_BUNDLES = [
  'sui-runtime.js',
  'sui-dom.js',
  'sui-sdk-client.js',
  'sui-components.js',
] as const;

mkdirSync(PUBLIC_DIR, { recursive: true });

let copied = 0;
for (const name of SUI_BUNDLES) {
  const src = join(SUI_ASSETS_DIR, name);
  if (!existsSync(src)) {
    console.log(
      `[copy-sui-assets] ${name} not found at ${src}. ` +
      `Run 'cd packages/ui-components && bun run build' first.`,
    );
    continue;
  }
  copyFileSync(src, join(PUBLIC_DIR, name));
  copied++;
}

console.log(`[copy-sui-assets] Copied ${copied}/${SUI_BUNDLES.length} SUI bundles to public/.`);
