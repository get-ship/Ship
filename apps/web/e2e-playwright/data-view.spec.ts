import { test, expect } from '@playwright/test';
import { readFileSync } from 'fs';
import { join } from 'path';

const TOKEN = readFileSync(join(process.env.HOME!, '.ship/daemon.token'), 'utf8').trim();
const BASE = 'http://127.0.0.1:9315';

test.describe('data browser view (server-rendered)', () => {
  test.beforeAll(async ({ request }) => {
    const headers = { 'Authorization': `Bearer ${TOKEN}`, 'Content-Type': 'application/json' };
    // Seed items — ignore conflicts if they already exist.
    for (const item of [
      { id: 'pw1', data: { title: 'Playwright task 1', status: 'todo', priority: 1 } },
      { id: 'pw2', data: { title: 'Playwright task 2', status: 'doing', priority: 3, tags: ['test'] } },
      { id: 'pw3', data: { title: 'Playwright task 3', status: 'done', priority: 5, done: true } },
    ]) {
      await request.post(`${BASE}/api/collections/task/items`, { headers, data: item }).catch(() => {});
    }
  });

  test('server-renders the page with no JS data fetching', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(1000);

    // Title rendered server-side
    const title = await page.textContent('h2');
    expect(title).toContain('Data');

    // Items rendered server-side in a table (no JS needed)
    const html = await page.content();
    expect(html).toContain('<table');
    expect(html).toContain('Playwright task');
    // No fetch() calls in the page
    expect(html).not.toContain('fetch(');
  });

  test('renders ship-elements in the HTML', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });

    const html = await page.content();
    expect(html).toContain('<ship-select');
    expect(html).toContain('<ship-button');
    expect(html).toContain('<ship-chip');
  });

  test('has create form with typed inputs', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });

    const html = await page.content();
    // Create form exists (hidden by default)
    expect(html).toContain('id="create-form"');
    // Typed inputs for different field types
    expect(html).toContain('<ship-input');
    expect(html).toContain('<ship-switch');
  });

  test('has delete buttons with Datastar actions', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });

    const html = await page.content();
    expect(html).toContain("@delete(");
    expect(html).toContain('/api/collections/');
  });

  test('has SSE subscription for live updates', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });

    const html = await page.content();
    expect(html).toContain('EventSource');
    expect(html).toContain('/api/runtime/events');
  });

  test('screenshot', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(2000);
    await page.screenshot({ path: 'test-results/data-view-final.png', fullPage: true });
  });
});
