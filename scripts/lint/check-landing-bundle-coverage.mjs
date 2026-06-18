#!/usr/bin/env node
/**
 * Landing custom-element coverage guard.
 *
 * Every `<sui-*>` / `<ship-*>` tag the marketing landing renders resolves to a
 * real Web Component ONLY if some bundle the page loads registers it. The eager
 * bundle is `sui-marketing.js`, whose membership is the side-effect import set
 * in `apps/site/src/marketing/marketing-bundle.ts` (the single source of truth).
 * An un-registered tag does NOT error — it renders as an inert
 * HTMLUnknownElement, silently. Task #225 shipped exactly that.
 *
 * This guard parses the landing sources for custom-element tags and fails when a
 * used tag is neither (a) imported in marketing-bundle.ts, nor (b) provided by a
 * documented lazily side-loaded bundle. There are no native `sui-`/`ship-`
 * elements, so any such tag MUST come from a bundle — the check is total.
 *
 * Run from anywhere; resolves the repo root via this file's location.
 */
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const LANDING_DIR = join(ROOT, 'apps/site/src/components/landing');
const BUNDLE = join(ROOT, 'apps/site/src/marketing/marketing-bundle.ts');

// (b) Tags registered by bundles the landing side-loads on demand rather than
// eagerly. Each is a documented exception with the bundle that defines it; the
// page injects `<script src="/<bundle>">` only when the owning window opens.
// Add here ONLY when the side-load is real and load-bearing.
const LAZY = {
  'sui-stage': '/sui-arcade.js', // ArcadeWindow injects the pixi pack on open
  'sui-hud': '/sui-arcade.js', //   "  — arcade HUD ships in the same pack
};

// (a) Eager membership: basename of each `.compiled.js` side-effect import.
const bundled = new Set(
  [...readFileSync(BUNDLE, 'utf8').matchAll(/([a-z0-9-]+)\.compiled\.js/g)].map((m) => m[1]),
);

// Blank out comments while preserving newlines so reported line numbers stay
// accurate. Covers JS `//` + `/* */`, Astro `{/* */}`, and HTML `<!-- -->`.
function stripComments(src) {
  const blank = (m) => m.replace(/[^\n]/g, ' ');
  return src
    .replace(/\/\*[\s\S]*?\*\//g, blank)
    .replace(/<!--[\s\S]*?-->/g, blank)
    .replace(/(^|[^:])\/\/[^\n]*/g, (m, p1) => p1 + ' '.repeat(m.length - p1.length));
}

function walk(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else if (/\.(astro|ts)$/.test(p)) out.push(p);
  }
  return out;
}

const violations = [];
for (const file of walk(LANDING_DIR)) {
  const lines = stripComments(readFileSync(file, 'utf8')).split('\n');
  lines.forEach((line, i) => {
    for (const m of line.matchAll(/<((?:sui|ship)-[a-z0-9-]+)/g)) {
      const tag = m[1];
      if (bundled.has(tag) || tag in LAZY) continue;
      violations.push({ file: relative(ROOT, file), line: i + 1, tag });
    }
  });
}

if (violations.length) {
  console.error('FAIL: landing renders custom-element tags no bundle registers.\n');
  for (const v of violations) {
    console.error(`  ${v.file}:${v.line}  <${v.tag}>  — inert HTMLUnknownElement on a fresh deploy`);
  }
  console.error(
    '\nFix one of:\n' +
    '  • add a side-effect import for the tag in apps/site/src/marketing/marketing-bundle.ts, or\n' +
    '  • remove the usage from the window, or\n' +
    '  • if it loads via a lazy bundle, register it in the LAZY map in this script.\n',
  );
  process.exit(1);
}

console.log(`landing bundle coverage OK — every <sui-*>/<ship-*> tag resolves (${bundled.size} eager + ${Object.keys(LAZY).length} lazy).`);
