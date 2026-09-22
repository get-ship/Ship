/**
 * Spot-check the marketing site against the local preview server.
 * Loads each page, asserts no console errors, and verifies a sample
 * element is present (native L1 or remaining sail-* CE).
 *
 * Usage:
 *   bun astro preview --port 4321 --host 127.0.0.1 &   # in another shell
 *   bun scripts/playwright-spot-check.mjs
 */

import { chromium } from 'playwright';

const BASE = process.env.SITE_BASE || 'http://127.0.0.1:4321';
const PAGES = [
  { path: '/', sample: 'button[variant], a[variant], button[tone], a[tone]', kind: 'selector' },
  { path: '/features', sample: '.m-card, sail-sparkline, sail-bar-chart', kind: 'selector' },
  { path: '/store', sample: '.m-badge, button, a', kind: 'selector' },
  { path: '/download', sample: 'button, a[href]', kind: 'selector' },
  { path: '/download/coming-soon', sample: 'form, button', kind: 'selector' },
  { path: '/sign-in', sample: 'form, button', kind: 'selector' },
  { path: '/sign-up', sample: 'form, button', kind: 'selector' },
];

const browser = await chromium.launch();
const ctx = await browser.newContext();
const failures = [];

for (const { path, sample, kind } of PAGES) {
  const page = await ctx.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  // Theme picker (if present) may fetch view-tokens.css against the
  // marketing host — 404 is expected without a daemon.
  const IGNORED_URL_FRAGMENT = '/api/assets/view-tokens.css';
  page.on('console', (msg) => {
    if (msg.type() !== 'error') return;
    const url = msg.location()?.url ?? '';
    if (url.includes(IGNORED_URL_FRAGMENT)) return;
    if (msg.text().includes(IGNORED_URL_FRAGMENT)) return;
    consoleErrors.push(`${msg.text()}  (at ${url || '<inline>'})`);
  });
  page.on('pageerror', (err) => {
    const text = String(err);
    if (text.includes(IGNORED_URL_FRAGMENT)) return;
    pageErrors.push(text);
  });
  page.on('response', (resp) => {
    if (resp.status() < 400) return;
    if (resp.url().includes(IGNORED_URL_FRAGMENT)) return;
    consoleErrors.push(`HTTP ${resp.status()} ${resp.url()}`);
  });

  const url = BASE + path;
  await page.goto(url, { waitUntil: 'networkidle' });

  const found = await page.evaluate((sel) => {
    try {
      return !!document.querySelector(sel);
    } catch {
      return false;
    }
  }, sample);

  const title = await page.title();
  const status = consoleErrors.length === 0 && pageErrors.length === 0 && found
    ? 'OK'
    : 'FAIL';

  console.log(`[${status}] ${path}  title="${title}"  sample=${sample} found=${found}`);
  if (consoleErrors.length) console.log(`  console errors: ${consoleErrors.join(' | ')}`);
  if (pageErrors.length) console.log(`  page errors:    ${pageErrors.join(' | ')}`);

  if (status === 'FAIL') failures.push({ path, consoleErrors, pageErrors, found });
  await page.close();
}

await browser.close();

if (failures.length) {
  console.error(`\n${failures.length}/${PAGES.length} pages failed.`);
  process.exit(1);
}
console.log(`\nAll ${PAGES.length} pages clean.`);
