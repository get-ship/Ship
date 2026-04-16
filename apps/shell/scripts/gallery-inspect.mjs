#!/usr/bin/env node
/**
 * Take per-section screenshots so we can inspect each group in isolation.
 */
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const OUT = process.env.OUT ?? "/tmp/gallery-inspect";
const URL = process.env.URL ?? "http://127.0.0.1:3100/devtools.html";

await mkdir(OUT, { recursive: true });
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await ctx.newPage();
await page.goto(URL, { waitUntil: "networkidle" });
await page.waitForTimeout(1600);
await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));

const sections = page.locator("section.group");
const n = await sections.count();
for (let i = 0; i < n; i++) {
  const sec = sections.nth(i);
  const h2 = (await sec.locator("h2").textContent()) ?? `sec-${i}`;
  const slug = h2.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
  await sec.scrollIntoViewIfNeeded();
  await page.waitForTimeout(400);
  await sec.screenshot({ path: join(OUT, `${String(i).padStart(2, "0")}-${slug}.png`) });
  console.log(`${i}: ${h2}`);
}

await browser.close();
