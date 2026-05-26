/**
 * Copy SUI runtime + component bundles into apps/site/public/ so the
 * static Astro build can serve them at the paths the importmap (in
 * src/layouts/BaseLayout.astro) expects.
 *
 * Source of truth: crates/deck/deck-core/assets/ — the same bundles
 * the daemon serves at /api/assets/. This mirrors apps/docs's
 * collect-skill-docs.ts SUI copy step (without the markdoc collection).
 */

import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, '../../..');
const PUBLIC_DIR = join(SCRIPT_DIR, '../public');
// Two source roots, by layer:
//   * deck's OWN runtime bundles live in deck's crate (baked product).
//   * the app-focused `sui-marketing.js` is deck *build output* and lives in
//     the project's `.deck/` (deck's `.next`-equivalent).
const SUI_ASSETS_DIR = join(REPO_ROOT, 'crates/deck/deck-core/assets');
const DECK_OUT_DIR = join(REPO_ROOT, '.deck');
const SHIPD_ASSETS_DIR = join(REPO_ROOT, 'apps/shipd/assets');

const SUI_BUNDLES: ReadonlyArray<{ name: string; from: string }> = [
  { name: 'sui-runtime.js', from: SUI_ASSETS_DIR },
  { name: 'sui-dom.js', from: SUI_ASSETS_DIR },
  { name: 'sui-sdk-client.js', from: SUI_ASSETS_DIR },
  // `sui-marketing.js` is the focused bundle for apps/site — registers
  // only the ~12 `<sui-*>` tags marketing actually mounts (~140 KB).
  // The full `sui-components.js` umbrella (~740 KB) was 5× larger and
  // contained 80+ tags the marketing pages never instantiate. If a new
  // tag is added to apps/site/src, append it to the `sui-marketing`
  // bundle target in `packages/deck/bundler/index.ts`. It builds to `.deck/`.
  { name: 'sui-marketing.js', from: DECK_OUT_DIR },
] as const;

// <sui-theme-picker> injects a <link rel="stylesheet" href="/api/assets/view-tokens.css">
// into its shadow root. Mirror that path so the request resolves on a
// static host (we have no daemon serving /api/assets/*).
const DAEMON_API_ASSETS: ReadonlyArray<{ src: string; dst: string }> = [
  { src: join(SHIPD_ASSETS_DIR, 'view-tokens.css'), dst: join(PUBLIC_DIR, 'api/assets/view-tokens.css') },
];

mkdirSync(PUBLIC_DIR, { recursive: true });

let copied = 0;
for (const { name, from } of SUI_BUNDLES) {
  const src = join(from, name);
  if (!existsSync(src)) {
    console.log(
      `[copy-sui-assets] ${name} not found at ${src}. ` +
      `Run 'bun run packages/deck/bundler/index.ts --all' first.`,
    );
    continue;
  }
  copyFileSync(src, join(PUBLIC_DIR, name));
  copied++;
}

for (const { src, dst } of DAEMON_API_ASSETS) {
  if (!existsSync(src)) {
    console.log(`[copy-sui-assets] ${src} not found; skipping.`);
    continue;
  }
  mkdirSync(dirname(dst), { recursive: true });
  copyFileSync(src, dst);
  copied++;
}

console.log(`[copy-sui-assets] Copied ${copied} assets to public/.`);
