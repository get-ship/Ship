/**
 * Copy SUI runtime + component bundles into apps/site/public/ so the
 * static Astro build can serve them at the paths the importmap (in
 * src/layouts/BaseLayout.astro) expects.
 *
 * Source of truth: crates/deck/deck-core/assets/ — the same bundles
 * the daemon serves at /api/assets/. This mirrors apps/docs's
 * collect-skill-docs.ts SUI copy step (without the markdoc collection).
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, '../../..');
const PUBLIC_DIR = join(SCRIPT_DIR, '../public');
const SITE_STYLES = join(SCRIPT_DIR, '../src/styles');
// Daemon-emulation source roots for the static-host shims below (per-icon SVGs,
// view-tokens.css, syntax tokens). These read deck's COMMITTED crate assets and
// the canonical token sheets — not deck build output. The bundle copy itself is
// delegated to `deck emit`, which owns deck's output layout + chunk closure.
const DECK_ASSETS_DIR = join(REPO_ROOT, 'crates/deck/deck-core/assets');

let copied = 0;

mkdirSync(PUBLIC_DIR, { recursive: true });

// ── SUI bundles + lazy-chunk closure → deck emit ─────────────────────
// Deck owns where each baked target lands and how its code-split chunks are
// named; a consumer names targets and deck copies the closure. This replaces
// the old hardcoded `.deck/` walk + hand-rolled chunk chaser.
//   * sui-runtime/dom/sdk-client — framework singletons (importmap externals).
//   * sui-marketing — the ~140 KB focused `<sui-*>` pack for the landing page.
//   * sui-arcade — heavy lazy pack (pixi) for the <sui-stage> games.
//   * ship-code-editor — CodeMirror pack the tutorial side-loads on first use.
{
  const targets = [
    'sui-runtime',
    'sui-dom',
    'sui-sdk-client',
    'sui-marketing',
    'sui-arcade',
    'ship-code-editor',
  ];
  const args = ['run', '-q', '-p', 'deck-cli', '--', 'emit', '--out', PUBLIC_DIR];
  for (const t of targets) args.push('--target', t);
  const r = spawnSync('cargo', args, { cwd: REPO_ROOT, stdio: 'inherit' });
  if (r.status !== 0) {
    console.error(
      `[copy-sui-assets] 'deck emit' failed (status ${r.status}). ` +
      `Run 'deck build-assets' first to bake the targets.`,
    );
    process.exit(1);
  }
  copied += targets.length;
}

// <sui-icon> fetches ONE icon at a time from `/_deck/icons/<name>.svg`
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
    const iconsDir = join(PUBLIC_DIR, '_deck/icons');
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
    console.log(`[copy-sui-assets] wrote ${n} per-icon SVGs to public/_deck/icons/`);
  } else {
    console.log(`[copy-sui-assets] ${src} not found; <sui-icon> will render empty.`);
  }
}

// <sui-theme-picker> injects <link href="/api/assets/view-tokens.css"> into
// its shadow root and expects the per-theme `[data-theme-preview="<id>"]`
// rules to be present so its swatch cards each render in their own theme.
// The daemon concatenates tokens.css + themes.css in front of the base
// view-tokens.css at runtime (deck-core view_tokens.rs). On a static
// host there is no daemon, so we reproduce that concatenation here from the
// site's own canonical token/theme sheets — otherwise every swatch falls
// back to the active theme.
{
  const dst = join(PUBLIC_DIR, 'api/assets/view-tokens.css');
  const base = join(DECK_ASSETS_DIR, 'view-tokens.css');
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

// The tutorial playground editor (`ship-code-editor.js` + its CodeMirror chunk
// closure) and the sandbox host (public/tutorial/{host.html,sw.js,scenarios},
// committed static source) load from public/ — the editor bundle is emitted by
// the `deck emit` step above as the `ship-code-editor` target.

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
    const dst = join(PUBLIC_DIR, '_deck/syntax-tokens.css');
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
