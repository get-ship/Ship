import { test, expect } from '@playwright/test';
import { readFileSync } from 'fs';
import { join } from 'path';

const TOKEN = readFileSync(join(process.env.HOME!, '.ship/daemon.token'), 'utf8').trim();
const BASE = 'http://127.0.0.1:9315';

test.describe('data browser view', () => {
  test.beforeAll(async ({ request }) => {
    // Seed test data
    const headers = { 'Authorization': `Bearer ${TOKEN}`, 'Content-Type': 'application/json' };
    await request.post(`${BASE}/api/collections/task/items`, {
      headers,
      data: { id: 'pw1', data: { title: 'Playwright task 1', status: 'todo', priority: 1 } },
    });
    await request.post(`${BASE}/api/collections/task/items`, {
      headers,
      data: { id: 'pw2', data: { title: 'Playwright task 2', status: 'doing', priority: 3, tags: ['test'] } },
    });
    await request.post(`${BASE}/api/collections/task/items`, {
      headers,
      data: { id: 'pw3', data: { title: 'Playwright task 3', status: 'done', priority: 5, done: true } },
    });
  });

  test('renders view with data', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });

    // Wait for JS to load and fetch data
    await page.waitForTimeout(3000);

    // Screenshot
    await page.screenshot({ path: 'test-results/data-view.png', fullPage: true });

    // Title should render
    const title = await page.textContent('.data-title');
    expect(title).toBe('Data');

    // Subtitle should show item count
    const subtitle = await page.textContent('#data-subtitle');
    expect(subtitle).toContain('item');
  });

  test('shows collection in toolbar', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(3000);

    const toolbar = await page.innerHTML('#data-toolbar');
    expect(toolbar).toContain('ship-select');
    expect(toolbar.toLowerCase()).toContain('task');
  });

  test('renders table with items', async ({ page }) => {
    await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
    await page.goto(`${BASE}/api/runtime/view/data/raw`, { waitUntil: 'domcontentloaded' });
    await page.waitForTimeout(3000);

    await page.screenshot({ path: 'test-results/data-view-table.png', fullPage: true });

    const content = await page.innerHTML('#data-content');
    expect(content).toContain('data-table');
    expect(content).toContain('Playwright task');
  });
});
