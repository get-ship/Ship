import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";
const OUT = process.argv[2];
const SELECTOR = process.argv[3];
const NAME = process.argv[4];
await mkdir(OUT, { recursive: true });
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await ctx.newPage();
await page.goto("http://127.0.0.1:3100/devtools.html", { waitUntil: "networkidle" });
await page.waitForTimeout(1000);
const card = page.locator(SELECTOR).first();
await card.scrollIntoViewIfNeeded();
await page.waitForTimeout(500);
const box = await card.boundingBox();
if (box) {
  await page.screenshot({
    path: join(OUT, NAME + ".png"),
    clip: { x: box.x, y: box.y, width: box.width, height: Math.min(box.height, 1200) },
  });
  console.log("OK", NAME, Math.round(box.height));
} else {
  console.log("MISSING", NAME);
}
await browser.close();
