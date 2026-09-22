/**
 * Copy Sail runtime + component bundles into apps/site/public/ so the
 * static Astro build can serve them at the paths the importmap (in
 * src/layouts/BaseLayout.astro) expects.
 *
 * Source of truth: the Sail tip checkout — by default
 * `~/dev/Ship/worktrees/orch/crates/deck/deck-core/assets/`, override with
 * `SAIL_REPO` (or legacy `SUI_REPO`). The daemon there serves the same
 * bundles at /api/assets/.
 */

import { cpSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(SCRIPT_DIR, '../../..');
const PUBLIC_DIR = join(SCRIPT_DIR, '../public');
const SITE_STYLES = join(SCRIPT_DIR, '../src/styles');
// Sail tip (orch worktree by default). SAIL_REPO preferred; SUI_REPO still
// accepted so existing operator env keeps working during the cutover.
const SAIL_REPO =
  process.env.SAIL_REPO ??
  process.env.SUI_REPO ??
  `${process.env.HOME}/dev/Ship/worktrees/orch`;
// Daemon-emulation source roots for the static-host shims below (per-icon SVGs,
// view-tokens.css, syntax tokens). These read deck's COMMITTED crate assets and
// the canonical token sheets — not deck build output. The bundle copy itself is
// delegated to `deck emit`, which owns deck's output layout + chunk closure.
const DECK_ASSETS_DIR = join(SAIL_REPO, 'crates/deck/deck-core/assets');

let copied = 0;

mkdirSync(PUBLIC_DIR, { recursive: true });

// ── Sail bundles + lazy-chunk closure → deck emit ────────────────────
// Deck owns where each baked target lands and how its code-split chunks are
// named; a consumer names targets and deck copies the closure.
//
// Framework singletons AND consumer bundles are both built by `deck
// build-assets` run from ship (site app) (the root `build:bundles` script): the
// framework is resolved from this workspace's `node_modules` (file: → orch),
// so there is no separate framework-repo bake. One `deck emit` from
// ship (site app) copies every target's output + lazy-chunk closure into
// `public/`.
{
  const targets = [
    // Framework singletons (node-resolved + baked under .deck/framework).
    'sail-runtime', 'sail-dom', 'sail-sdk-client',
    // Consumer: marketing only. sail-arcade needs ship deck bake first
    // (target not in known list until build-assets discovers @ship/arcade).
    'sail-marketing',
  ];
  const args = ['run', '-q', '--manifest-path', join(SAIL_REPO, 'Cargo.toml'), '-p', 'deck-cli', '--', 'emit', '--out', PUBLIC_DIR];
  for (const t of targets) args.push('--target', t);
  const r = spawnSync('cargo', args, {
    cwd: REPO_ROOT,
    stdio: 'inherit',
    env: { ...process.env },
  });
  if (r.status !== 0) {
    console.error(
      `[copy-sail-assets] 'deck emit' failed (status ${r.status}). ` +
      `Run 'deck build-assets' from ship (site app) first (the 'build:bundles' script).`,
    );
    process.exit(1);
  }
  copied += targets.length;
}
// Icon glyphs: pre-generate each icon as a standalone SVG using the SAME
// wrapper the daemon emits (viewBox 0 0 24 24, stroke=currentColor).
{
  const src = join(DECK_ASSETS_DIR, 'lucide-icons.json');
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
    console.log(`[copy-sail-assets] wrote ${n} per-icon SVGs to public/_deck/icons/`);
  } else {
    console.log(`[copy-sail-assets] ${src} not found; icon SVGs skipped.`);
  }
}

// Theme-picker / preview token sheet (static host has no daemon).
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
    console.log(`[copy-sail-assets] ${missing} not found; skipping view-tokens.css.`);
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
{
  const CANON = join(SAIL_REPO, 'packages/sail/tokens/src');
  const blocks: string[] = [];
  for (const name of ['tokens.css', 'themes.css']) {
    const src = join(CANON, name);
    if (!existsSync(src)) continue;
    const css = readFileSync(src, 'utf8').replace(/\/\*[\s\S]*?\*\//g, '');
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
      '/* Generated by copy-sail-assets.ts — `--syntax-*` tokens for <ship-code-editor>.\n' +
      ' * Source of truth: $SAIL_REPO/packages/sail/tokens/src/ (default ~/dev/Ship/worktrees/orch). Do not edit by hand. */\n' +
      blocks.join('\n\n') + '\n',
    );
    copied++;
  } else {
    console.log('[copy-sail-assets] no --syntax-* tokens in canonical sheets; editor highlighting will be monochrome.');
  }
}

// ── @ship/sail-compiler-wasm pkg/ artifacts ──────────────────────────
// The registry package for @ship/sail-compiler-wasm may only contain package.json
// (the pkg/ build output is not always published). Copy the wasm binary and JS
// wrapper from the sail tip into the installed node_modules package so Vite can
// resolve the exports (./pkg/sail_compiler.js, ./pkg/sail_compiler_bg.wasm).
{
  const SAIL_WASM_SRC = join(SAIL_REPO, 'packages/sail/compiler-wasm/pkg');
  const wasmNm = join(SCRIPT_DIR, '../node_modules/@ship/sail-compiler-wasm/pkg');
  if (existsSync(SAIL_WASM_SRC) && !existsSync(join(wasmNm, 'sail_compiler.js'))) {
    mkdirSync(wasmNm, { recursive: true });
    cpSync(SAIL_WASM_SRC, wasmNm, { recursive: true });
    copied++;
    console.log(`[copy-sail-assets] copied @ship/sail-compiler-wasm/pkg from sail tip`);
  }
}

console.log(`[copy-sail-assets] Copied ${copied} assets to public/.`);
