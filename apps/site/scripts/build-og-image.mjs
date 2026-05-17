/**
 * Render scripts/og-template.html → public/og-image.png at 1200x630.
 *
 * Idempotent: skips if the PNG already exists, unless `--force` is passed
 * or `FORCE_OG=1` is set. This keeps CI/Cloudflare-build environments —
 * which may not have playwright + chromium — from failing the build.
 *
 * Run locally with `--force` to regenerate after editing the template.
 *
 * Run automatically as part of `prebuild`. Requires playwright (already
 * a dev dep in the repo root). Headless Chromium opens the template via
 * file:// (so the Google Fonts @import works) and screenshots the body.
 */

import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const TEMPLATE = join(SCRIPT_DIR, 'og-template.html');
const OUT = join(SCRIPT_DIR, '../public/og-image.png');

const force = process.argv.includes('--force') || process.env.FORCE_OG === '1';
if (existsSync(OUT) && !force) {
  console.log(`[build-og-image] ${OUT} exists; skipping (pass --force to regenerate).`);
  process.exit(0);
}

let chromium;
try {
  ({ chromium } = await import('playwright'));
} catch (err) {
  console.log(`[build-og-image] playwright unavailable (${err.message}); leaving existing PNG in place.`);
  process.exit(0);
}

const browser = await chromium.launch();
const ctx = await browser.newContext({
  viewport: { width: 1200, height: 630 },
  deviceScaleFactor: 1,
});
const page = await ctx.newPage();
await page.goto(pathToFileURL(TEMPLATE).href, { waitUntil: 'networkidle' });
// Give web fonts a moment to settle once networkidle fires.
await page.waitForTimeout(300);
await page.screenshot({ path: OUT, type: 'png', clip: { x: 0, y: 0, width: 1200, height: 630 } });
await browser.close();

console.log(`[build-og-image] wrote ${OUT}`);
