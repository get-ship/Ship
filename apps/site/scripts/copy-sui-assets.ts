/**
 * Copy SUI runtime + component bundles into apps/site/public/ so the
 * static Astro build can serve them at the paths the importmap (in
 * src/layouts/BaseLayout.astro) expects.
 *
 * Source of truth: crates/deck/deck-core/assets/ — the same bundles
 * the daemon serves at /api/assets/. This mirrors apps/docs's
 * collect-skill-docs.ts SUI copy step (without the markdoc collection).
 */

import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, '../../..');
const PUBLIC_DIR = join(SCRIPT_DIR, '../public');
const SITE_STYLES = join(SCRIPT_DIR, '../src/styles');
// Two source roots, by layer:
//   * deck's OWN runtime bundles (sui-runtime/dom/sdk-client) now live in
//     .deck/framework/ — the canonical bake target (Recommendation B1).
//   * the app-focused `sui-marketing.js` is deck *build output* and lives in
//     the project's `.deck/` (deck's `.next`-equivalent).
const SUI_ASSETS_DIR = join(REPO_ROOT, '.deck', 'framework');
const DECK_OUT_DIR = join(REPO_ROOT, '.deck');
const DECK_CHUNKS_DIR = join(DECK_OUT_DIR, 'sui-components');
const SHIPD_ASSETS_DIR = join(REPO_ROOT, 'apps/shipd/assets');

const SUI_BUNDLES: ReadonlyArray<{ name: string; from: string }> = [
  { name: 'sui-runtime.js', from: SUI_ASSETS_DIR },
  { name: 'sui-dom.js', from: SUI_ASSETS_DIR },
  { name: 'sui-sdk-client.js', from: SUI_ASSETS_DIR },
  // `sui-marketing.js` is the focused bundle for apps/site — registers
  // only the ~12 `<sui-*>` tags marketing actually mounts (~140 KB).
  // The full `sui-components.js` umbrella (~740 KB) was 5× larger and
  // contained 80+ tags the marketing pages never instantiate. If a new
  // tag is added to apps/site/src, add a side-effect import for it to
  // the traced entry `apps/site/src/marketing/marketing-bundle.ts` —
  // rolldown follows that import graph. It builds to `.deck/`.
  { name: 'sui-marketing.js', from: DECK_OUT_DIR },
  // `sui-arcade.js` is the heavy lazy chunk (pixi + arcade-engine, ~800 KB)
  // for the landing page's `<sui-stage>` games. ArcadeWindow.astro dynamic-
  // imports `/sui-arcade.js` the first time the Arcade window opens; the
  // runtime singletons stay external (page importmap). Deck builds it to
  // `.deck/` as the `sui-arcade` target.
  { name: 'sui-arcade.js', from: DECK_OUT_DIR },
] as const;

mkdirSync(PUBLIC_DIR, { recursive: true });

let copied = 0;
for (const { name, from } of SUI_BUNDLES) {
  const src = join(from, name);
  if (!existsSync(src)) {
    console.log(
      `[copy-sui-assets] ${name} not found at ${src}. ` +
      `Run 'deck build-assets' first.`,
    );
    continue;
  }
  copyFileSync(src, join(PUBLIC_DIR, name));
  copied++;
}

// <sui-icon> fetches ONE icon at a time from `/_sui/icons/<name>.svg`
// (deck-core::serve_sui_icon). The daemon serves those; on a static host
// there is no daemon, so we pre-generate each icon as a standalone SVG using
// the SAME wrapper the daemon emits (viewBox 0 0 24 24, stroke=currentColor).
// Without this every <sui-icon name="…"> renders empty — which is what was
// breaking <sui-theme-picker>'s palette trigger. (The old monolithic
// lucide-icons.json copy is dead — nothing fetches it — so we drop it.)
{
  // lucide-icons.json is committed source — it stays in the crate's assets/.
  const src = join(REPO_ROOT, 'crates/deck/deck-core/assets', 'lucide-icons.json');
  if (existsSync(src)) {
    const catalog = JSON.parse(readFileSync(src, 'utf8')) as Record<string, string>;
    const iconsDir = join(PUBLIC_DIR, '_sui/icons');
    mkdirSync(iconsDir, { recursive: true });
    let n = 0;
    for (const [name, inner] of Object.entries(catalog)) {
      if (!/^[a-z0-9-]+$/.test(name)) continue;
      const svg =
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" ' +
        'stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
        inner + '</svg>';
      writeFileSync(join(iconsDir, `${name}.svg`), svg);
      n++;
    }
    copied++;
    console.log(`[copy-sui-assets] wrote ${n} per-icon SVGs to public/_sui/icons/`);
  } else {
    console.log(`[copy-sui-assets] ${src} not found; <sui-icon> will render empty.`);
  }
}

// <sui-theme-picker> injects <link href="/api/assets/view-tokens.css"> into
// its shadow root and expects the per-theme `[data-theme-preview="<id>"]`
// rules to be present so its swatch cards each render in their own theme.
// The daemon concatenates tokens.css + themes.css in front of the base
// view-tokens.css at runtime (apps/shipd/src/views/assets.rs). On a static
// host there is no daemon, so we reproduce that concatenation here from the
// site's own canonical token/theme sheets — otherwise every swatch falls
// back to the active theme.
{
  const dst = join(PUBLIC_DIR, 'api/assets/view-tokens.css');
  const base = join(SHIPD_ASSETS_DIR, 'view-tokens.css');
  const parts = [
    join(SITE_STYLES, 'tokens.css'),
    join(SITE_STYLES, 'themes.css'),
    base,
  ];
  const missing = parts.find((p) => !existsSync(p));
  if (missing) {
    console.log(`[copy-sui-assets] ${missing} not found; skipping view-tokens.css.`);
  } else {
    mkdirSync(dirname(dst), { recursive: true });
    const bundled = parts.map((p) => readFileSync(p, 'utf8')).join('\n');
    writeFileSync(dst, bundled);
    copied++;
  }
}

// ── Tutorial playground editor ───────────────────────────────────────
// The SUI tutorial (getship.dev/tutorial) is a live editor + sandboxed
// preview. The editor element — `ship-code-editor.js` — is a CodeMirror-
// based lazy chunk in deck's build output (`.deck/sui-components/`), NOT
// in the lean marketing bundle. The tutorial side-loads it on first use,
// so we mirror it into public/ here.
//
// The sandbox host itself (public/tutorial/{host.html,sw.js,scenarios})
// is committed static source under this app — not copied here.
{
  const editorSrc = join(DECK_CHUNKS_DIR, 'ship-code-editor.js');
  if (existsSync(editorSrc)) {
    const editorDstDir = join(PUBLIC_DIR, 'sui-components');
    mkdirSync(editorDstDir, { recursive: true });
    copyFileSync(editorSrc, join(editorDstDir, 'ship-code-editor.js'));
    copied++;
  } else {
    console.log(
      `[copy-sui-assets] ${editorSrc} not found; tutorial editor will not load. ` +
      `Build the deck sui-components chunks first.`,
    );
  }
}

// ── Editor syntax-highlight tokens ───────────────────────────────────
// <ship-code-editor> colours tokens via `--syntax-*` CSS custom properties
// (keyword/string/comment/tag/…), per theme. They're defined in the
// canonical SUI token sheets but NOT in the site's curated tokens — so
// without them the editor parses tokens but renders every one in the
// foreground colour (no visible highlighting). Extract just the
// `--syntax-*` rules (per theme) and emit a small sheet the tutorial
// loads. Sourced from canonical so it can't drift.
{
  const CANON = join(REPO_ROOT, 'packages/sui/tokens/src');
  const blocks: string[] = [];
  for (const name of ['tokens.css', 'themes.css']) {
    const src = join(CANON, name);
    if (!existsSync(src)) continue;
    // Strip comments first — a comment between `{` and a declaration
    // otherwise swallows that declaration (e.g. --syntax-keyword) and
    // leaks into the captured selector.
    const css = readFileSync(src, 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
    // Flat `selector { decls }` blocks — token rules aren't nested.
    const re = /([^{}]+)\{([^{}]*)\}/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(css)) !== null) {
      const selector = m[1].trim();
      const decls = m[2].split(';').map((d) => d.trim()).filter((d) => d.startsWith('--syntax-'));
      if (decls.length) blocks.push(`${selector} {\n  ${decls.join(';\n  ')};\n}`);
    }
  }
  if (blocks.length) {
    const dst = join(PUBLIC_DIR, '_sui/syntax-tokens.css');
    mkdirSync(dirname(dst), { recursive: true });
    writeFileSync(
      dst,
      '/* Generated by copy-sui-assets.ts — `--syntax-*` tokens for <ship-code-editor>.\n' +
      ' * Source of truth: packages/sui/tokens/src/. Do not edit by hand. */\n' +
      blocks.join('\n\n') + '\n',
    );
    copied++;
  } else {
    console.log('[copy-sui-assets] no --syntax-* tokens in canonical sheets; editor highlighting will be monochrome.');
  }
}

console.log(`[copy-sui-assets] Copied ${copied} assets to public/.`);
