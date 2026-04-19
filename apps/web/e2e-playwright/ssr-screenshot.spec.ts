import { test } from '@playwright/test';
import { readFileSync } from 'fs';
import { join } from 'path';

const TOKEN = readFileSync(join(process.env.HOME!, '.ship/daemon.token'), 'utf8').trim();

test('screenshot server-rendered data view', async ({ page }) => {
  await page.setExtraHTTPHeaders({ 'Authorization': `Bearer ${TOKEN}` });
  await page.goto('http://127.0.0.1:9315/api/runtime/view/data/raw', { waitUntil: 'domcontentloaded' });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'test-results/data-view-ssr.png', fullPage: true });
});
