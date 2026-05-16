/**
 * Spot-check the marketing site against the local preview server.
 * Loads each page, asserts no console errors, and verifies a sample
 * <sui-*> custom element has upgraded.
 *
 * Usage:
 *   bun astro preview --port 4321 --host 127.0.0.1 &   # in another shell
 *   bun scripts/playwright-spot-check.mjs
 */

import { chromium } from 'playwright';

const BASE = process.env.SITE_BASE || 'http://127.0.0.1:4321';
const PAGES = [
  { path: '/', sample: 'sui-button' },
  { path: '/features', sample: 'sui-card' },
  { path: '/store', sample: 'sui-badge' },
  { path: '/download', sample: 'sui-button' },
  { path: '/download/coming-soon', sample: 'sui-alert' },
  { path: '/sign-in', sample: 'sui-auth-dialog' },
  { path: '/sign-up', sample: 'sui-auth-dialog' },
];

const browser = await chromium.launch();
const ctx = await browser.newContext();
const failures = [];

for (const { path, sample } of PAGES) {
  const page = await ctx.newPage();
  const consoleErrors = [];
  const pageErrors = [];
  // `/api/assets/view-tokens.css` is fetched by <sui-theme-picker> to inject
  // per-theme preview styles into its shadow root. The marketing site has
  // no daemon so the request 404s — picker still works, preview cards just
  // fall back to the default tokens. Filter that one expected 404.
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

  // Give the SUI bundle a beat to upgrade custom elements.
  const upgraded = await page.evaluate(async (tag) => {
    try {
      await customElements.whenDefined(tag);
      return !!document.querySelector(`${tag}:defined`);
    } catch {
      return false;
    }
  }, sample);

  const title = await page.title();
  const status = consoleErrors.length === 0 && pageErrors.length === 0 && upgraded
    ? 'OK'
    : 'FAIL';

  console.log(`[${status}] ${path}  title="${title}"  ${sample}:defined=${upgraded}`);
  if (consoleErrors.length) console.log(`  console errors: ${consoleErrors.join(' | ')}`);
  if (pageErrors.length) console.log(`  page errors:    ${pageErrors.join(' | ')}`);

  if (status === 'FAIL') failures.push({ path, consoleErrors, pageErrors, upgraded });
  await page.close();
}

await browser.close();

if (failures.length) {
  console.error(`\n${failures.length}/${PAGES.length} pages failed.`);
  process.exit(1);
}
console.log(`\nAll ${PAGES.length} pages clean.`);
